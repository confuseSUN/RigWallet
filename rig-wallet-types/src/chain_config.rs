pub use rig_wallet_macro::{ChainConfig, TokenConfig};

use crate::{errors::Error, errors::Result};

pub trait ChainConfig: Send + Sync {
    const RPC: &'static str;
    const CHAIN_ID: Option<u64>;

    fn chain_id() -> Result<u64> {
        Self::CHAIN_ID
            .ok_or_else(|| Error::Config("no chain ID configured".into()))
    }
}

pub trait TokenConfig: Send + Sync {
    const TOKEN: &'static str;
    const DECIMAL: Option<u8>;
}
