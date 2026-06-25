use rig_wallet_executor::tvm::spl::SplTransfer;
use rig_wallet_types::chain_config::TokenConfig;

use crate::solana::sol::SOLConfig;

#[derive(Clone, TokenConfig)]
#[mainnet_token = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"]
#[decimal = "6"]
#[tool_name = "SOLUSDT"]
pub struct USDTConfig;
pub type SOLUSDT = SplTransfer<SOLConfig, USDTConfig>;
