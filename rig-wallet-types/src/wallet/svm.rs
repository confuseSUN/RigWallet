//! SVM wallet implementation for Solana.
//!
//! This module provides the functionality to derive and use Solana wallets
//! from BIP-39 mnemonics following BIP-44 derivation paths.

use rig_wallet_account::{Curve, DerivationPath, ExtendedPrivKey, Mnemonic};
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};
use zeroize::Zeroize;

use crate::{
    errors::Result,
    wallet::{ProviderWallet, Wallet},
};

pub type SVMWallet = Wallet<Keypair>;

impl ProviderWallet for SVMWallet {
    fn from_env() -> Result<Self> {
        let mnemonic = std::env::var("MNEMONIC")?;
        derive_svm_wallet(&mnemonic, "m/44'/501'/0'/0'")
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

/// Derive an SVM wallet from a BIP-39 mnemonic and BIP-44 derivation path.
///
/// # Security
///
/// The extended private key's secret bytes are zeroized in memory after use
/// to minimize exposure of sensitive cryptographic material.
pub fn derive_svm_wallet(phrase: &str, path: &str) -> Result<SVMWallet> {
    let mnemonic = Mnemonic::from_phrase(phrase)?;
    let seed = mnemonic.to_seed("");
    let path: DerivationPath = path.parse()?;
    let mut extended_key = ExtendedPrivKey::derive(&seed, path, Curve::Ed25519)?;

    // Create keypair from the secret key bytes
    let keypair = Keypair::new_from_array(extended_key.secret_key);
    let address = keypair.pubkey().to_string();

    // Zero out sensitive data in memory
    extended_key.secret_key.zeroize();

    Ok(Wallet::new(keypair, address))
}
