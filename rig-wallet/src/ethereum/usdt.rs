use rig_wallet_executor::evm::erc20::ERC20Transfer;
use rig_wallet_types::chain_config::TokenConfig;

use crate::ethereum::eth::ETHConfig;

#[derive(Clone, TokenConfig)]
#[mainnet_token = "0xdAC17F958D2ee523a2206206994597C13D831ec7"]
#[testnet_token = "0x7169D38820dfd117C3FA1f22a697dBA58d90BA06"]
#[tool_name = "ETHUSDT"]
pub struct USDTConfig;
pub type ETHUSDT = ERC20Transfer<ETHConfig, USDTConfig>;
