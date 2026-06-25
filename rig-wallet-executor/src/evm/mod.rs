use alloy::{
    consensus::{EthereumTxEnvelope, SignableTransaction, TxEip4844Variant, TypedTransaction},
    eips::Encodable2718,
    providers::{Provider, ProviderBuilder},
};
use rig_wallet_types::{errors::Result, wallet::evm::EvmSignature};

pub mod eip1559;
pub mod erc20;

/// Gas limit safety margin applied on top of the RPC estimate (20%).
const GAS_LIMIT_NUMERATOR: u64 = 12;
const GAS_LIMIT_DENOMINATOR: u64 = 10;

/// Broadcast a signed EIP-1559 transaction over HTTP JSON-RPC.
pub async fn broadcast_signed_tx(
    tx_unsigned: TypedTransaction,
    signature: EvmSignature,
    rpc: &str,
) -> Result<String> {
    let tx_envelope: EthereumTxEnvelope<TxEip4844Variant> =
        tx_unsigned.into_signed(signature).into();
    let tx_encoded = tx_envelope.encoded_2718();

    let url = rpc.parse()?;
    let provider = ProviderBuilder::new().connect_http(url);

    let pending = provider.send_raw_transaction(&tx_encoded).await?;

    Ok(pending.tx_hash().to_string())
}
