use async_trait::async_trait;

use crate::{errors::Result, signer::Signer, wallet::Wallet};

pub struct TxData {
    pub payload: Vec<u8>,
    pub address: String,
}

impl TxData {
    pub fn new(payload: Vec<u8>, address: String) -> Self {
        Self { payload, address }
    }
}

#[async_trait]
pub trait Transaction<S: Signer + Send + Sync> {
    async fn build(&mut self) -> Result<Vec<TxData>>;

    async fn build_and_sign(&mut self, wallet: &[Wallet<S>]) -> Result<Vec<S::Signature>> {
        let msgs = self.build().await?;
        Ok(msgs
            .iter()
            .map(|m| {
                let current_w = wallet
                    .iter()
                    .find(|w| w.address == m.address || w.address == format!("0x{}", m.address))
                    .unwrap();
                current_w.priv_key.sign(&m.payload)
            })
            .collect())
    }

    async fn send(&self, signatures: Vec<S::Signature>) -> Result<String>;
}
