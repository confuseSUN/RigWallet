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
    async fn build(&mut self) -> Result<Payload>;

    async fn sign(&mut self) -> Result<SignatureOf<W>>;

    async fn send_signed(&self, signature: SignatureOf<W>) -> Result<String>;

    async fn send(&mut self) -> Result<String> {
        let signature = self.sign().await?;
        self.send_signed(signature).await
    }
}
