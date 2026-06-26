use core::array::TryFromSliceError;
use core::fmt;

use hmac::digest::InvalidLength;
use k256::elliptic_curve::Error as K256Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Mnemonic only supports 12/15/18/21/24 words.
    BadWordCount(usize),
    /// Entropy was not a multiple of 32 bits or between 128-256 bits in length.
    BadEntropyBitCount(usize),
    /// Mnemonic contains an unknown word.
    UnknownWord(String),
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// The mnemonic has an invalid child number.
    InvalidChildNum,
    /// The path must start with "m/".
    BadDerivationPath,
    /// Invalid secret key.
    InvalidSecretKey,
    /// Invalid HMAC key length.
    InvalidHmacKeyLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadWordCount(count) => {
                write!(
                    f,
                    "BIP-39 mnemonic only supports 12/15/18/21/24 words: {count}"
                )
            }
            Error::BadEntropyBitCount(count) => write!(
                f,
                "entropy was not between 128-256 bits or not a multiple of 32 bits: {count} bits"
            ),
            Error::UnknownWord(word) => write!(f, "mnemonic contains an unknown word: {word}"),
            Error::InvalidChecksum => write!(f, "mnemonic has an invalid checksum"),
            Error::InvalidChildNum => write!(f, "invalid child number in derivation path"),
            Error::BadDerivationPath => {
                write!(f, "derivation path must start with \"m/\"")
            }
            Error::InvalidSecretKey => write!(f, "invalid secret key"),
            Error::InvalidHmacKeyLength => write!(f, "invalid HMAC key length"),
        }
    }
}

impl std::error::Error for Error {}

impl From<K256Error> for Error {
    fn from(_: K256Error) -> Self {
        Error::InvalidSecretKey
    }
}

impl From<InvalidLength> for Error {
    fn from(_: InvalidLength) -> Self {
        Error::InvalidHmacKeyLength
    }
}

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Self {
        Error::InvalidSecretKey
    }
}

pub type Result<T> = core::result::Result<T, Error>;
