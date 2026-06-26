use std::sync::Arc;

use crate::errors::Result;

pub mod context;
pub mod evm;
pub mod svm;

pub use context::WalletContext;

pub use evm::{EVMWallet, EvmSignature, derive_evm_wallet};
pub use svm::{SVMWallet, SvmSignature, derive_svm_wallet};

pub trait Signer {
    type Signature;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature>;
}

/// Trait for wallet initialization from environment.
pub trait ProviderWallet: Sized {
    /// Loads the wallet from environment variables.
    fn from_env() -> Result<Self>;
}

#[derive(Debug)]
pub struct Wallet<S: Signer> {
    signer: Arc<S>,
    address: String,
}

impl<S: Signer> Clone for Wallet<S> {
    fn clone(&self) -> Self {
        Self {
            signer: Arc::clone(&self.signer),
            address: self.address.clone(),
        }
    }
}

impl<S: Signer> Wallet<S> {
    pub fn new(signer: S, address: impl Into<String>) -> Self {
        Self {
            signer: Arc::new(signer),
            address: address.into(),
        }
    }

    pub fn signer(&self) -> &S {
        &self.signer
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}

impl<S: Signer> Signer for Wallet<S> {
    type Signature = S::Signature;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature> {
        self.signer.sign(msg)
    }
}

pub type SignatureOf<T> = <T as Signer>::Signature;
