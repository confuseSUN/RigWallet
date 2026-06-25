use async_trait::async_trait;
use rig_wallet_types::{
    chain_config::ChainConfig,
    errors::Result,
    transaction::{Payload, Transaction},
    tvm_transfer_builder,
    wallet::{
        Signer, WalletContext,
        svm::{SVMWallet, SvmSignature},
    },
};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig, rpc_request::Address,
};
use solana_sdk::transaction::Transaction as SdkTransaction;
use solana_system_interface::instruction::transfer;

use crate::tvm::broadcast_signed_tx;

tvm_transfer_builder!(
    tool(description = "Transfer SOL to a given address."),
    SolTransfer,
    ChainConfig,
);

#[async_trait]
impl<C: ChainConfig> Transaction<SVMWallet> for SolTransfer<C> {
    async fn build(&mut self) -> Result<Payload> {
        let from: Address = WalletContext::svm()?.address().parse()?;
        let to: Address = self.request.to.parse()?;

        let client =
            RpcClient::new_with_commitment(C::RPC.to_string(), CommitmentConfig::confirmed());
        let recent_blockhash = client.get_latest_blockhash().await?;

        let transfer_instruction = transfer(&from, &to, self.request.value as u64);
        let instructions = vec![transfer_instruction];

        let mut transaction = SdkTransaction::new_with_payer(&instructions, Some(&from));
        transaction.message.recent_blockhash = recent_blockhash;
        let data = transaction.message_data();

        self.tx_unsigned = transaction;

        Ok(data)
    }

    async fn sign(&mut self) -> Result<SvmSignature> {
        let payload = self.build().await?;
        WalletContext::svm()?.sign(&payload)
    }

    async fn send_signed(&self, signature: SvmSignature) -> Result<String> {
        broadcast_signed_tx(self.tx_unsigned.clone(), signature, C::RPC).await
    }
}
