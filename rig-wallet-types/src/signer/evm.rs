use alloy::{
    primitives::keccak256,
    signers::{Signature, SignerSync, local::PrivateKeySigner},
};

use crate::signer::Signer;

/// EVM signature
pub type EVMSignature = Signature;

impl Signer for PrivateKeySigner {
    type Signature = EVMSignature;

    fn sign(&self, msg: &[u8]) -> Self::Signature {
        self.sign_hash_sync(&keccak256(msg)).unwrap()
    }
}
