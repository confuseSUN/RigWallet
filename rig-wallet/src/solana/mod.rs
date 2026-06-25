//! Solana transfer types and [`create_solana_agent`].

use rig_core::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    providers::openai,
};
use rig_wallet_types::errors::{Error, Result};

use crate::solana::{sol::SOL, usdc::SOLUSDC, usdt::SOLUSDT};

pub mod sol;
pub mod usdc;
pub mod usdt;

/// Builds a rig agent with SOL + SPL transfer tools.
pub fn create_solana_agent(
    preamble: Option<&str>,
) -> Result<Agent<<openai::Client as CompletionClient>::CompletionModel>> {
    let preamble = preamble.unwrap_or("You are a Solana wallet agent.");

    let agent = openai::Client::from_env()
        .map_err(|e| Error::Config(e.to_string()))?
        .agent("qwen3.6-plus")
        .preamble(preamble)
        .tool(SOL::new())
        .tool(SOLUSDC::new())
        .tool(SOLUSDT::new())
        .max_tokens(1024)
        .default_max_turns(10)
        .build();

    Ok(agent)
}
