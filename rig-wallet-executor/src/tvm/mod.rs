use rig_wallet_types::SvmSignature;
use rig_wallet_types::errors::Result;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_sdk::transaction::Transaction;

pub mod native;
pub mod spl;

pub async fn broadcast_signed_tx(
    mut tx: Transaction,
    signature: SvmSignature,
    rpc: &str,
) -> Result<String> {
    tx.signatures[0] = signature;

    let client = RpcClient::new_with_commitment(rpc.to_string(), CommitmentConfig::confirmed());

    let tx_hash = client.send_and_confirm_transaction(&tx).await?;

    Ok(tx_hash.to_string())
}
