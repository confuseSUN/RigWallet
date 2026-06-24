use alloy::{
    hex::FromHexError,
    network::{Ethereum, UnbuiltTransactionError},
    primitives::AddressError,
    signers::{Error as SignerError, local::LocalSignerError},
    transports::RpcError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Account(#[from] rig_wallet_account::Error),

    #[error(transparent)]
    Address(#[from] AddressError),

    #[error(transparent)]
    Hex(#[from] FromHexError),

    #[error(transparent)]
    Signer(#[from] SignerError),

    #[error(transparent)]
    SignerKey(#[from] LocalSignerError),

    #[error("invalid signature")]
    InvalidSignature,

    #[error("transaction not built; call build() first")]
    NotBuilt,

    #[error(transparent)]
    TxBuild(#[from] UnbuiltTransactionError<Ethereum>),

    #[error(transparent)]
    Rpc(#[from] RpcError<alloy::transports::TransportErrorKind>),

    #[error("invalid RPC URL: {0}")]
    RpcUrl(#[from] url::ParseError),

    #[error("misconfigured: {0}")]
    Config(String),
}

pub type Result<T> = core::result::Result<T, Error>;
