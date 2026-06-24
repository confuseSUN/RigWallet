//! Transaction building, signing, and broadcasting.

use async_trait::async_trait;

use crate::{
    errors::Result,
    wallet::{SignatureOf, Signer},
};

/// Bytes to be signed by the wallet.
pub type Payload = Vec<u8>;

#[async_trait]
pub trait Transaction<W: Signer + Send + Sync>: Send {
    async fn build(&mut self) -> Result<Vec<Payload>>;

    async fn sign(&mut self, wallet: &W) -> Result<Vec<SignatureOf<W>>> {
        let payloads = self.build().await?;
        payloads
            .iter()
            .map(|payload| wallet.sign(payload))
            .collect()
    }

    async fn send_signed(&self, signatures: Vec<SignatureOf<W>>) -> Result<String>;

    async fn send(&mut self, wallet: &W) -> Result<String> {
        let signatures = self.sign(wallet).await?;
        self.send_signed(signatures).await
    }
}
