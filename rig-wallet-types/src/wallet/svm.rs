//! SVM wallet implementation for Solana.
//!
//! This module provides the functionality to derive and use Solana wallets
//! from BIP-39 mnemonics following BIP-44 derivation paths.

use rig_wallet_account::{Curve, DerivationPath, ExtendedPrivKey, Mnemonic};
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use crate::{
    errors::Result,
    wallet::{ProviderWallet, Wallet},
};

pub type SVMWallet = Wallet<Keypair>;

impl ProviderWallet for SVMWallet {
    fn from_env() -> Self {
        let mnemonic =
            std::env::var("MNEMONIC").expect("MNEMONIC environment variable must be set");
        derive_svm_wallet(&mnemonic, "m/44'/501'/0'/0'")
            .expect("Failed to derive SVM wallet from mnemonic")
    }
}

/// SVM ECDSA signature.
pub type SvmSignature = Signature;

impl super::Signer for Keypair {
    type Signature = SvmSignature;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature> {
        Ok(self.sign_message(msg))
    }
}

pub fn derive_svm_wallet(phrase: &str, path: &str) -> Result<SVMWallet> {
    let mnemonic = Mnemonic::from_phrase(phrase)?;
    let seed = &mnemonic.to_seed("");
    let path: DerivationPath = path.parse().unwrap();

    let e_priv_key = ExtendedPrivKey::derive(seed, path, Curve::Ed25519)?;
    let keypair = Keypair::new_from_array(e_priv_key.secret_key);
    let address = keypair.pubkey().to_string();
    Ok(Wallet::new(keypair, address))
}