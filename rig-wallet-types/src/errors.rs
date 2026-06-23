use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    WalletNotFound(String),
    InvalidSignature,
    DecodeFailed,
    RpcFailed(String),
    SendFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::WalletNotFound(address) => write!(f, "wallet not found for address: {address}"),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::DecodeFailed => write!(f, "failed to decode signature bytes"),
            Error::RpcFailed(msg) => write!(f, "rpc request failed: {msg}"),
            Error::SendFailed(msg) => write!(f, "failed to send transaction: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
