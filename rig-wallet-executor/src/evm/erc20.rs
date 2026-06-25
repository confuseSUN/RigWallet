use alloy::{
    consensus::SignableTransaction,
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    sol,
};
use async_trait::async_trait;
use rig_wallet_types::{
    chain_config::{ChainConfig, TokenConfig},
    errors::Result,
    evm_transfer_builder,
    transaction::{Payload, Transaction},
    wallet::{EVMWallet, Signer, WalletContext, evm::EvmSignature},
};

use crate::evm::{GAS_LIMIT_DENOMINATOR, GAS_LIMIT_NUMERATOR, broadcast_signed_tx};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    EvmToken,
    "ERC20.json"
);

evm_transfer_builder!(
    tool(description = "Transfer ERC20 tokens to a given address."),
    ERC20Transfer,
    ChainConfig,
    TokenConfig,
);

#[async_trait]
impl<C: ChainConfig, T: TokenConfig> Transaction<EVMWallet> for ERC20Transfer<C, T> {
    async fn build(&mut self) -> Result<Payload> {
        let from: Address = WalletContext::evm()?.address().parse()?;
        let to: Address = self.request.to.parse()?;
        let token: Address = T::TOKEN.parse()?;

        let url = C::RPC.parse()?;
        let provider = ProviderBuilder::new().connect_http(url);
        let nonce = provider.get_transaction_count(from).pending().await?;
        let fees = provider.estimate_eip1559_fees().await?;

        let contract = EvmToken::new(token, provider.clone());
        let data = contract
            .transfer(to, U256::from(self.request.value))
            .calldata()
            .clone();

        let tx = TransactionRequest::default()
            .with_from(from)
            .with_to(token)
            .with_nonce(nonce)
            .with_chain_id(C::chain_id()?)
            .with_input(data)
            .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas)
            .with_max_fee_per_gas(fees.max_fee_per_gas);

        let gas_used = provider.estimate_gas(tx.clone()).await?;
        let gas_limit = gas_used * GAS_LIMIT_NUMERATOR / GAS_LIMIT_DENOMINATOR;
        let tx = tx.with_gas_limit(gas_limit);

        let tx_unsigned = tx.build_unsigned()?;

        let mut payload = Vec::with_capacity(tx_unsigned.payload_len_for_signature());
        tx_unsigned.encode_for_signing(&mut payload);

        self.tx_unsigned = Some(tx_unsigned);

        Ok(payload)
    }

    async fn sign(&mut self) -> Result<EvmSignature> {
        let payload = self.build().await?;
        WalletContext::evm()?.sign(&payload)
    }

    async fn send_signed(&self, signature: EvmSignature) -> Result<String> {
        broadcast_signed_tx(self.tx_unsigned.clone().unwrap(), signature, C::RPC).await
    }
}
