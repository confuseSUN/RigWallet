use async_trait::async_trait;
use rig_wallet_types::{
    chain_config::{ChainConfig, TokenConfig},
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
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::{id as token_program_id, instruction::transfer_checked};

use crate::tvm::broadcast_signed_tx;

tvm_transfer_builder!(
    tool(description = "Transfer SPL tokens to a given address."),
    SplTransfer,
    ChainConfig,
    TokenConfig,
);

#[async_trait]
impl<C: ChainConfig, T: TokenConfig> Transaction<SVMWallet> for SplTransfer<C, T> {
    async fn build(&mut self) -> Result<Payload> {
        let from: Address = WalletContext::svm()?.address().parse()?;
        let to: Address = self.request.to.parse()?;
        let mint: Address = T::TOKEN.parse()?;

        let client =
            RpcClient::new_with_commitment(C::RPC.to_string(), CommitmentConfig::confirmed());
        let recent_blockhash = client.get_latest_blockhash().await?;

        let from_ata = get_associated_token_address(&from, &mint);
        let to_ata = get_associated_token_address(&to, &mint);

        let mut instructions = vec![];

        if client.get_account(&to_ata).await.is_err() {
            instructions.push(create_associated_token_account(
                &from,
                &to,
                &mint,
                &token_program_id(),
            ));
        }

        let transfer_instruction = transfer_checked(
            &token_program_id(),
            &from_ata,
            &mint,
            &to_ata,
            &from,
            &[&from],
            self.request.value as u64,
            T::DECIMAL.unwrap(),
        )
        .unwrap();
        instructions.push(transfer_instruction);

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
