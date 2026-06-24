use alloy::{
    hex,
    primitives::keccak256,
    signers::{Signature, Signer as AlloySigner, SignerSync, local::PrivateKeySigner},
};
use rig_wallet_account::{Curve, DerivationPath, ExtendedPrivKey, Mnemonic};

use super::{Signer, Wallet};
use crate::errors::Result;

pub type EVMWallet = Wallet<PrivateKeySigner>;

/// Derive an EVM wallet from a BIP-39 mnemonic and BIP-44 derivation path.
pub fn derive_evm_wallet(phrase: &str, path: &str) -> Result<EVMWallet> {
    let mnemonic = Mnemonic::from_phrase(phrase)?;
    let seed = mnemonic.to_seed("");
    let path: DerivationPath = path.parse()?;
    let extended_key = ExtendedPrivKey::derive(&seed, path, Curve::K256)?;

    let hex_key = hex::encode(extended_key.secret_key);
    let signer: PrivateKeySigner = hex_key.parse()?;
    let address = AlloySigner::address(&signer).to_string();

    Ok(Wallet::new(signer, address))
}

/// EVM ECDSA signature.
pub type EvmSignature = Signature;

impl Signer for PrivateKeySigner {
    type Signature = EvmSignature;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature> {
        Ok(self.sign_hash_sync(&keccak256(msg))?)
    }
}
