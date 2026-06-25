use std::sync::Arc;

use rig_core::completion::Prompt;
use rig_wallet::solana::create_solana_agent;
use rig_wallet_types::wallet::{ProviderWallet, SVMWallet, WalletContext};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let wallet = Arc::new(SVMWallet::from_env());

    WalletContext::with_svm(wallet, async {
        let agent = create_solana_agent(None).unwrap();

        let response = agent
            .prompt("send 0.0001 sol to ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc")
            .await
            .unwrap();
        println!("{response}");

        println!("--------------------------------------------");

        let response = agent
            .prompt("send 0.0001 usdc to ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc")
            .await
            .unwrap();
        println!("{response}");
    })
    .await;
}
