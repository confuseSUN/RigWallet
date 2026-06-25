//! Ethereum transfer types and [`create_ethereum_agent`].

use rig_core::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    providers::openai,
};
use rig_wallet_types::errors::{Error, Result};

use crate::ethereum::{dai::ETHDAI, eth::ETH, usdc::ETHUSDC, usdt::ETHUSDT};

pub mod dai;
pub mod eth;
pub mod usdc;
pub mod usdt;

/// Builds a rig agent with ETH + ERC-20 transfer tools.
///
/// Requires `OPENAI_API_KEY` (or compatible provider env) and an EVM wallet in
/// [`WalletContext`](rig_wallet_types::wallet::WalletContext).
pub fn create_ethereum_agent(
    preamble: Option<&str>,
) -> Result<Agent<<openai::Client as CompletionClient>::CompletionModel>> {
    let preamble = preamble.unwrap_or("You are an Ethereum wallet agent.");

    let agent = openai::Client::from_env()
        .map_err(|e| Error::Config(e.to_string()))?
        .agent("qwen3.6-plus")
        .preamble(preamble)
        .tool(ETH::new())
        .tool(ETHUSDC::new())
        .tool(ETHUSDT::new())
        .tool(ETHDAI::new())
        .max_tokens(1024)
        .default_max_turns(10)
        .build();

    Ok(agent)
}
