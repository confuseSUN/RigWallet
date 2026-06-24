use std::marker::PhantomData;

use alloy::{
    consensus::{SignableTransaction, TypedTransaction}, network::TransactionBuilder, primitives::{Address, Bytes, U256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest, signers::local::PrivateKeySigner,
};
use async_trait::async_trait;
use rig_core::{completion::ToolDefinition, tool::Tool};
use rig_wallet_types::{
    chain_config::ChainConfig,
    errors::Result,
    request::TxRequest,
    transaction::{Payload, Transaction},
    wallet::{evm::EvmSignature, EVMWallet, Wallet},
};
use serde::Deserialize;
use serde_json::json;

use super::send::broadcast_signed_tx;

/// Gas limit safety margin applied on top of the RPC estimate (20%).
const GAS_LIMIT_NUMERATOR: u64 = 12;
const GAS_LIMIT_DENOMINATOR: u64 = 10;

pub struct Eip1559Transfer<C: ChainConfig> {
    pub request: TxRequest,
    tx_unsigned: Option<TypedTransaction>,
    wallet: Wallet<PrivateKeySigner>,
    _c: PhantomData<C>,
}

impl<C: ChainConfig> Eip1559Transfer<C> {
    pub fn new(request: TxRequest, wallet: Wallet<PrivateKeySigner>) -> Self {
        Self {
            request,
            tx_unsigned: None,
            wallet,
            _c: PhantomData,
        }
    }

    pub fn from_wallet(wallet: Wallet<PrivateKeySigner>) -> Self {
        Self::new(TxRequest::default(), wallet)
    }

    pub fn wallet(&self) -> &Wallet<PrivateKeySigner> {
        &self.wallet
    }
}

impl<C: ChainConfig> Clone for Eip1559Transfer<C> {
    fn clone(&self) -> Self {
        Self {
            request: self.request.clone(),
            tx_unsigned: self.tx_unsigned.clone(),
            wallet: self.wallet.clone(),
            _c: PhantomData,
        }
    }
}

#[async_trait]
impl<C: ChainConfig> Transaction<EVMWallet> for Eip1559Transfer<C> {
    async fn build(&mut self) -> Result<Vec<Payload>> {
        let from: Address = self.wallet.address().parse()?;
        let to: Address = self.request.to.parse()?;
        let url = C::RPC.parse()?;
        let provider = ProviderBuilder::new().connect_http(url);

        let nonce = provider.get_transaction_count(from).await?;
        let fees = provider.estimate_eip1559_fees().await?;

        let mut tx = TransactionRequest::default()
            .with_from(from)
            .with_to(to)
            .with_nonce(nonce)
            .with_chain_id(C::chain_id()?)
            .with_value(U256::from(self.request.value))
            .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas)
            .with_max_fee_per_gas(fees.max_fee_per_gas);

        if !self.request.data.is_empty() {
            tx = tx.with_input(Bytes::copy_from_slice(&self.request.data));
        }

        let gas_used = provider.estimate_gas(tx.clone()).await?;
        let gas_limit = gas_used * GAS_LIMIT_NUMERATOR / GAS_LIMIT_DENOMINATOR;
        let tx = tx.with_gas_limit(gas_limit);

        let tx_unsigned = tx.build_unsigned()?;

        let mut payload = Vec::with_capacity(tx_unsigned.payload_len_for_signature());
        tx_unsigned.encode_for_signing(&mut payload);

        self.tx_unsigned = Some(tx_unsigned);

        Ok(vec![payload])
    }

    async fn send_signed(&self, signatures: Vec<EvmSignature>) -> Result<String> {
        broadcast_signed_tx(self.tx_unsigned.as_ref(), &signatures, C::RPC).await
    }
}

#[derive(Debug, Deserialize)]
pub struct TransferEthArgs {
    to: String,
    value: u128,
}

impl<C: ChainConfig> Tool for Eip1559Transfer<C> {
    const NAME: &'static str = "transfer_eth";
    type Error = rig_wallet_types::errors::Error;
    type Args = TransferEthArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer ETH to a given address. Amount is in wei.".to_string(),
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
        let wallet = self.wallet.clone();
        let mut transfer = self.clone();
        transfer.request = TxRequest::new(args.to, args.value);
        transfer.tx_unsigned = None;
        transfer.send(&wallet).await
    }
}
