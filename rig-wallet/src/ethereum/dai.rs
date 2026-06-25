use rig_wallet_executor::evm::erc20::ERC20Transfer;
use rig_wallet_types::chain_config::TokenConfig;

use crate::ethereum::eth::ETHConfig;

#[derive(Clone, TokenConfig)]
#[mainnet_token = "0x6B175474E89094C44Da98b954EedeAC495271d0F"]
#[tool_name = "ETHDAI"]
pub struct DAIConfig;
pub type ETHDAI = ERC20Transfer<ETHConfig, DAIConfig>;
