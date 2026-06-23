pub use rig_wallet_macro::{ChainConfig, TokenConfig};

pub trait ChainConfig: Send + Sync {
    const RPC: &'static [&'static str];

    const CHAIN_ID: Option<u64>;
}
