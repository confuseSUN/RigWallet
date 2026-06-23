use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Mnemonic only support 12/15/18/21/24 words.
    BadWordCount(usize),
    /// Entropy was not a multiple of 32 bits or between 128-256n bits in length.
    BadEntropyBitCount(usize),
    /// Mnemonic contains an unknown word.
    UnknownWord(String),
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// The mnemonic has an invalid child number.
    InvalidChildNum,
    /// The path must start with "m/".
    BadDrivePath,
    /// Invalid secret key.
    InvalidSecretKey,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadWordCount(count) => {
                write!(
                    f,
                    "BIP-0039 mnemonic only supports 12/15/18/21/24 words: {count}"
                )
            }
            Error::BadEntropyBitCount(count) => write!(
                f,
                "entropy was not between 128-256 bits or not a multiple of 32 bits: {count} bits"
            ),
            Error::UnknownWord(word) => write!(f, "mnemonic contains an unknown word: {word}"),
            Error::InvalidChecksum => write!(f, "mnemonic has an invalid checksum"),
            Error::InvalidChildNum => write!(f, "Childnum is invalid"),
            Error::BadDrivePath => write!(f, "The driver path likes m/.."),
            Error::InvalidSecretKey => write!(f, "Invalid secret key"),
        }
    }
}

impl std::error::Error for Error {}
