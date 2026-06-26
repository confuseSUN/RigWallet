use std::sync::Arc;

use rig_core::completion::Prompt;
use rig_wallet::solana::create_solana_agent;
use rig_wallet_types::wallet::{ProviderWallet, SVMWallet, WalletContext};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let wallet = match SVMWallet::from_env() {
        Ok(w) => Arc::new(w),
        Err(e) => {
            eprintln!("Failed to load wallet: {}", e);
            std::process::exit(1);
        }
    };

    WalletContext::with_svm(wallet, async {
        let agent = create_solana_agent(None).expect("Failed to create agent");

        match agent
            .prompt("send 0.0001 sol to ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc")
            .await
        {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("Transaction failed: {}", e),
        }

        match agent
            .prompt("send 0.0001 usdc to ExMUofCj1sqMzPyy6od6Ak6CvnzhqpZQ7aQ9JWxdaiAc")
            .await
        {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("Transaction failed: {}", e),
        }
    })
    .await;
}
