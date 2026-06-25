use rig_wallet_executor::tvm::spl::SplTransfer;
use rig_wallet_types::chain_config::TokenConfig;

use crate::solana::sol::SOLConfig;

#[derive(Clone, TokenConfig)]
#[mainnet_token = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]
#[testnet_token = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"]
#[decimal = "6"]
#[tool_name = "SOLUSDC"]
pub struct USDCConfig;
pub type SOLUSDC = SplTransfer<SOLConfig, USDCConfig>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rig_wallet_types::{
        request::TxRequest,
        transaction::Transaction,
        wallet::{ProviderWallet, SVMWallet, WalletContext},
    };

    use crate::solana::usdc::SOLUSDC;

    #[tokio::test]
    async fn transfer_usdc() {
        dotenvy::dotenv().ok();

        let request = TxRequest::new("ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc", 10);

        WalletContext::with_svm(Arc::new(SVMWallet::from_env()), async {
            let mut usdc = SOLUSDC::new().with_request(request);
            let tx_hash = usdc.send().await.unwrap();
            println!("tx_hash: {tx_hash}");
        })
        .await;
    }
}
