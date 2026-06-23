use std::marker::PhantomData;

use alloy::{
    consensus::{EthereumTxEnvelope, SignableTransaction, TxEip4844Variant, TypedTransaction},
    eips::Encodable2718,
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use async_trait::async_trait;
use rig_core::{completion::ToolDefinition, tool::Tool};
use rig_wallet_types::{
    Error,
    chain_config::ChainConfig,
    errors::Result,
    request::TxRequest,
    signer::evm::EVMSignature,
    transaction::{Transaction, TxData},
    wallet::Wallet,
};
use serde::Deserialize;
use serde_json::json;

pub struct EIP1559<C: ChainConfig> {
    pub request: TxRequest,
    tx_unsigned: Option<TypedTransaction>,
    wallet: Wallet<PrivateKeySigner>,
    _c: PhantomData<C>,
}

impl<C: ChainConfig> EIP1559<C> {
    pub fn new(request: TxRequest, wallet: Wallet<PrivateKeySigner>) -> Self {
        Self {
            request,
            tx_unsigned: None,
            wallet,
            _c: PhantomData,
        }
    }

    pub fn from_wallet(wallet: Wallet<PrivateKeySigner>) -> Self {
        Self {
            request: TxRequest::default(),
            tx_unsigned: None,
            wallet,
            _c: PhantomData,
        }
    }
}

#[async_trait]
impl<C: ChainConfig> Transaction<PrivateKeySigner> for EIP1559<C> {
    async fn build(&mut self) -> Result<Vec<TxData>> {
        let from = self.wallet.address.parse::<Address>().unwrap();
        let to = self.request.to.parse::<Address>().unwrap();

        let url = C::RPC.first().unwrap().parse().unwrap();
        let provider = ProviderBuilder::new().connect_http(url);
        let nonce = provider
            .get_transaction_count(from)
            .await
            .map_err(|e| Error::RpcFailed(e.to_string()))?;
        let estimate = provider
            .estimate_eip1559_fees()
            .await
            .map_err(|e| Error::RpcFailed(e.to_string()))?;

        let tx = TransactionRequest::default()
            .with_from(from)
            .with_to(to)
            .with_nonce(nonce)
            .with_chain_id(C::CHAIN_ID.unwrap())
            .with_value(U256::from(self.request.value))
            .with_max_priority_fee_per_gas(estimate.max_priority_fee_per_gas)
            .with_max_fee_per_gas(estimate.max_fee_per_gas);

        let gas_used = provider
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| Error::RpcFailed(e.to_string()))?;

        let tx = tx.with_gas_limit(gas_used * 12 / 10);

        let tx_unsigned = tx.build_unsigned().unwrap();
        let mut buf = Vec::with_capacity(tx_unsigned.payload_len_for_signature());
        tx_unsigned.encode_for_signing(&mut buf);

        self.tx_unsigned = Some(tx_unsigned);

        Ok(vec![TxData::new(buf, self.wallet.address.clone())])
    }

    async fn send(&self, signature: Vec<EVMSignature>) -> Result<String> {
        crate::_evm_send_tx!(self, signature, C::RPC.first().unwrap())
    }
}

#[derive(Debug, Deserialize)]
pub struct TransferEthArgs {
    to: String,
    value: u128,
}

impl<C: ChainConfig> Tool for EIP1559<C> {
    const NAME: &'static str = "transfer_eth";
    type Error = Error;
    type Args = TransferEthArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer ETH to a given address".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient address"
                    },
                    "value": {
                        "type": "integer",
                        "description": "Amount in wei"
                    }
                },
                "required": ["to", "value"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        let request = TxRequest {
            to: args.to,
            value: args.value,
            data: vec![],
        };

        let wallet = self.wallet.clone();
        let mut eth = EIP1559::<C>::new(request, wallet.clone());
        let signatures = eth.build_and_sign(&[wallet]).await?;
        eth.send(signatures).await
    }
}
