use alloy::{
    consensus::{EthereumTxEnvelope, SignableTransaction, TxEip4844Variant, TypedTransaction},
    eips::Encodable2718,
    providers::{Provider, ProviderBuilder},
};
use rig_wallet_types::{
    errors::{Error, Result},
    wallet::evm::EvmSignature,
};

/// Broadcast a signed EIP-1559 transaction over HTTP JSON-RPC.
pub async fn broadcast_signed_tx(
    tx_unsigned: Option<&TypedTransaction>,
    signatures: &[EvmSignature],
    rpc: &str,
) -> Result<String> {
    let signature = signatures
        .first()
        .copied()
        .ok_or(Error::InvalidSignature)?;
    let tx_unsigned = tx_unsigned.ok_or(Error::NotBuilt)?;

    let tx_envelope: EthereumTxEnvelope<TxEip4844Variant> =
        tx_unsigned.clone().into_signed(signature).into();
    let tx_encoded = tx_envelope.encoded_2718();

    let url = rpc.parse()?;
    let provider = ProviderBuilder::new().connect_http(url);

    let pending = provider.send_raw_transaction(&tx_encoded).await?;

    Ok(pending.tx_hash().to_string())
}
