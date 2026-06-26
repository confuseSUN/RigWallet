use std::str::FromStr;

use hmac::{Hmac, Mac};
use k256::{SecretKey, elliptic_curve::sec1::ToEncodedPoint};
use sha2::Sha512;

use super::types::Curve;
use crate::errors::Error;

const HARDENED_BIT: u32 = 1 << 31;

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub struct ChildNumber(u32);

impl ChildNumber {
    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn is_hardened(&self) -> bool {
        self.0 & HARDENED_BIT == HARDENED_BIT
    }

    pub fn is_normal(&self) -> bool {
        self.0 & HARDENED_BIT == 0
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub fn hardened_from_u32(index: u32) -> Self {
        ChildNumber(index | HARDENED_BIT)
    }

    pub fn non_hardened_from_u32(index: u32) -> Self {
        ChildNumber(index)
    }
}

impl FromStr for ChildNumber {
    type Err = Error;

    fn from_str(input: &str) -> Result<ChildNumber, Error> {
        let (num_str, hardened) = match input.strip_suffix('\'') {
            Some(n) => (n, HARDENED_BIT),
            None => (input, 0),
        };

        let index: u32 = num_str.parse().map_err(|_| Error::InvalidChildNum)?;
        if index >= HARDENED_BIT {
            return Err(Error::InvalidChildNum);
        }

        // equivalent to addition
        Ok(ChildNumber(index | hardened))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct DerivationPath {
    path: Vec<ChildNumber>,
}

impl DerivationPath {
    pub fn as_ref(&self) -> &[ChildNumber] {
        &self.path
    }
}

impl FromStr for DerivationPath {
    type Err = Error;

    fn from_str(path: &str) -> Result<DerivationPath, Error> {
        let mut parts = path.split('/');

        if parts.next() != Some("m") {
            return Err(Error::BadDerivationPath);
        }

        let mut path_vec = Vec::new();
        for part in parts {
            path_vec.push(part.parse()?);
        }

        Ok(DerivationPath { path: path_vec })
    }
}

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct ExtendedPrivKey {
    pub curve: Curve,
    pub secret_key: [u8; 32],
    pub chain_code: [u8; 32],
}

impl ExtendedPrivKey {
    /// Derives an extended private key from a seed and derivation path.
    ///
    /// # Security
    ///
    /// The returned `ExtendedPrivKey` contains sensitive cryptographic material.
    /// Callers should zeroize `secret_key` after use.
    pub fn derive(
        seed: &[u8],
        path: DerivationPath,
        curve: Curve,
    ) -> Result<ExtendedPrivKey, Error> {
        let (secret_key, chain_code) = Self::derive_master_key(seed, curve)?;

        let mut sk = ExtendedPrivKey {
            curve,
            secret_key,
            chain_code,
        };

        for child in path.as_ref().iter() {
            sk = sk.child(child)?;
        }

        Ok(sk)
    }

    /// Derives the master key from a seed using HMAC-SHA512.
    fn derive_master_key(seed: &[u8], curve: Curve) -> Result<([u8; 32], [u8; 32]), Error> {
        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(curve.seed_key())?;
        hmac.update(seed);
        let hash = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = hash.split_at(32);

        let sk: [u8; 32] = secret_key.try_into()?;
        let cc: [u8; 32] = chain_code.try_into()?;

        Ok((sk, cc))
    }

    pub fn child(&self, child: &ChildNumber) -> Result<ExtendedPrivKey, Error> {
        let (secret_key, chain_code) = match self.curve {
            Curve::K256 => self.child_k256(child)?,
            Curve::Ed25519 => self.child_ed25519(child)?,
        };

        Ok(ExtendedPrivKey {
            curve: self.curve,
            secret_key,
            chain_code,
        })
    }

    fn child_k256(&self, child: &ChildNumber) -> Result<([u8; 32], [u8; 32]), Error> {
        let mut bytes = Vec::new();
        if child.is_normal() {
            let sk = SecretKey::from_slice(&self.secret_key)?;
            let pk = sk.public_key().to_encoded_point(true);
            bytes.extend(pk.as_bytes());
        } else {
            bytes.push(0);
            bytes.extend(&self.secret_key);
        }
        bytes.extend(&child.to_bytes());

        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.chain_code)?;
        hmac.update(&bytes);
        let i = hmac.finalize().into_bytes();
        let (il, ir) = i.split_at(32);

        let child_sk = tweak_key(&self.secret_key, il)?;

        let cc: [u8; 32] = ir.try_into()?;

        Ok((child_sk, cc))
    }

    fn child_ed25519(&self, child: &ChildNumber) -> Result<([u8; 32], [u8; 32]), Error> {
        let mut bytes = Vec::new();
        bytes.push(0);
        bytes.extend_from_slice(&self.secret_key);
        bytes.extend_from_slice(&child.to_bytes());

        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.chain_code)?;
        hmac.update(&bytes);
        let i = hmac.finalize().into_bytes();
        let (il, ir) = i.split_at(32);

        let sk: [u8; 32] = il.try_into()?;
        let cc: [u8; 32] = ir.try_into()?;

        Ok((sk, cc))
    }
}

pub fn tweak_key(key1: &[u8], key2: &[u8]) -> Result<[u8; 32], Error> {
    let sk1 = SecretKey::from_slice(key1)?;
    let sk2 = SecretKey::from_slice(key2)?;
    let new_secret_key = sk1
        .to_nonzero_scalar()
        .add(&sk2.to_nonzero_scalar())
        .to_bytes();
    Ok(new_secret_key.into())
}
