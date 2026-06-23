use crate::errors::Result;

pub mod evm;

pub trait SignFromToBytes: Sized + Send + Sync {
    fn from_bytes(bytes: &[u8], addr: Option<String>, tx: Option<&[u8]>) -> Result<Self>;

    fn to_bytes(&self) -> Vec<u8>;
}

pub trait Signer {
    type Signature;

    fn sign(&self, msg: &[u8]) -> Self::Signature;
}
