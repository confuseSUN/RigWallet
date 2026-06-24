use crate::errors::Result;

pub mod evm;

pub use evm::{derive_evm_wallet, EVMWallet, EvmSignature};

pub trait Signer {
    type Signature;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature>;
}

#[derive(Debug, Clone)]
pub struct Wallet<S: Signer> {
    signer: S,
    address: String,
}

impl<S: Signer> Wallet<S> {
    pub fn new(signer: S, address: impl Into<String>) -> Self {
        Self {
            signer,
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
