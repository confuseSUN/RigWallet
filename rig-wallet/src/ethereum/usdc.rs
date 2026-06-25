//! USDC on Ethereum — type alias over the shared [`ERC20Transfer`] builder.
//!
//! `ETHUSDC` monomorphizes `ERC20Transfer<ETHConfig, USDCConfig>`; the agent tool name
//! comes from `USDCConfig::TOOL_NAME` (`"ETHUSDC"`), not from the struct ident.

use rig_wallet_executor::evm::erc20::ERC20Transfer;
use rig_wallet_types::chain_config::TokenConfig;

use crate::ethereum::eth::ETHConfig;

#[derive(Clone, TokenConfig)]
#[mainnet_token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"]
#[testnet_token = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"]
#[tool_name = "ETHUSDC"]
pub struct USDCConfig;
pub type ETHUSDC = ERC20Transfer<ETHConfig, USDCConfig>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rig_wallet_types::{
        request::TxRequest,
        transaction::Transaction,
        wallet::{EVMWallet, ProviderWallet, WalletContext},
    };

    use crate::ethereum::usdc::ETHUSDC;

    #[tokio::test]
    async fn transfer_usdc() {
        dotenvy::dotenv().ok();

        let request = TxRequest::new("0x3677572639a6b17725f04946b45A12E6443344F5", 10);

        WalletContext::with_evm(Arc::new(EVMWallet::from_env()), async {
            let mut usdc = ETHUSDC::new().with_request(request);
            let tx_hash = usdc.send().await.unwrap();
            println!("tx_hash: {tx_hash}");
        })
        .await;
    }
}
