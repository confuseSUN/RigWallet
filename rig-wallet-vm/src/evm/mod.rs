pub mod eip1559;

#[macro_export]
macro_rules! _evm_send_tx {
    ($self:expr, $signature:expr, $rpc:expr) => {{
        // for eth only one signature is needed
        let signature = $signature[0];
        let tx_envelope: EthereumTxEnvelope<TxEip4844Variant> = $self
            .tx_unsigned
            .clone()
            .unwrap()
            .into_signed(signature)
            .into();
        let tx_encoded = tx_envelope.encoded_2718();

        let url = $rpc.parse().unwrap();
        let provider = ProviderBuilder::new().connect_http(url);

        let pending = provider
            .send_raw_transaction(&tx_encoded)
            .await
            .map_err(|e| rig_wallet_types::Error::RpcFailed(e.to_string()))?;
        let tx_hash = pending.tx_hash().to_string();

        Ok(tx_hash)
    }};
}
