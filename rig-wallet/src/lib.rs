//! RigWallet facade: pre-wired chain/token types and agent helpers.
//!
//! - [`ethereum`] — ETH, USDC, USDT, DAI on Ethereum
//! - [`solana`] — SOL, USDC, USDT on Solana

pub mod ethereum;
pub mod solana;

pub use rig_wallet_types::{derive_evm_wallet, request::TxRequest, transaction::Transaction};
