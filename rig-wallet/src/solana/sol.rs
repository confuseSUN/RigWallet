use rig_wallet_executor::tvm::native::SolTransfer;
use rig_wallet_types::chain_config::ChainConfig;

#[derive(Clone, ChainConfig)]
#[mainnet_rpc = "https://api.mainnet-beta.solana.com"]
#[testnet_rpc = "https://api.devnet.solana.com"]
pub struct SOLConfig;
pub type SOL = SolTransfer<SOLConfig>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rig_wallet_types::{
        request::TxRequest,
        transaction::Transaction,
        wallet::{ProviderWallet, SVMWallet, WalletContext},
    };

    use crate::solana::sol::SOL;

    #[tokio::test]
    async fn transfer_sol() {
        dotenvy::dotenv().ok();

        let request = TxRequest::new("ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc", 10);

        WalletContext::with_svm(Arc::new(SVMWallet::from_env().unwrap()), async {
            let mut sol = SOL::new().with_request(request);
            let tx_hash = sol.send().await.unwrap();
            println!("tx_hash: {tx_hash}");
        })
        .await;
    }
}
