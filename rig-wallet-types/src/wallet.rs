use rig_wallet_account::{
    mnemonic::Mnemonic,
    path::{DerivationPath, ExtendedPrivKey},
    types::Curve,
};

use alloy::{hex, signers::local::PrivateKeySigner};

use crate::signer::Signer;

#[derive(Debug, Clone)]
pub struct Wallet<S: Signer> {
    pub priv_key: S,
    pub address: String,
}

// pub fn gen_btc_wallet(phrase: &str, path: &str, r#type: &str) -> Wallet {
//     let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
//     let seed = &mnemonic.to_seed("");
//     let path: DerivationPath = path.parse().unwrap();

//     let e_priv_key = ExtendedPrivKey::derive(seed, path, Curve::K256).unwrap();
//     let privkey =
//         PrivateKey::from_slice(&e_priv_key.secret_key, bitcoin::Network::Bitcoin).unwrap();
//     let secp = Secp256k1::signing_only();
//     let pubkey = privkey.public_key(&secp);
//     let network = if cfg!(feature = "testnet") {
//         bitcoin::Network::Testnet
//     } else {
//         bitcoin::Network::Bitcoin
//     };
//     let addr = match r#type {
//         "00" => Address::p2pkh(&pubkey, network),
//         "20" => Address::p2wpkh(&CompressedPublicKey::try_from(pubkey).unwrap(), network),
//         _ => unimplemented!(),
//     };

//     Wallet {
//         priv_key: e_priv_key.secret_key.to_vec(),
//         address: addr.to_string(),
//     }
// }

pub fn gen_evm_wallet(phrase: &str, path: &str) -> Wallet<PrivateKeySigner> {
    let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
    let seed = &mnemonic.to_seed("");
    let path: DerivationPath = path.parse().unwrap();

    let e_priv_key = ExtendedPrivKey::derive(seed, path, Curve::K256).unwrap();
    let hex_priv_key = hex::encode(e_priv_key.secret_key);
    let key: PrivateKeySigner = hex_priv_key.parse().unwrap();
    let address = key.address().to_string();
    Wallet {
        priv_key: key,
        address: address,
    }
}
