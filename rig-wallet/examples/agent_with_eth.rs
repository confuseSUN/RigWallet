use std::sync::Arc;

use rig_core::completion::Prompt;
use rig_wallet::ethereum::create_ethereum_agent;
use rig_wallet_types::wallet::{EVMWallet, ProviderWallet, WalletContext};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let wallet = match EVMWallet::from_env() {
        Ok(w) => Arc::new(w),
        Err(e) => {
            eprintln!("Failed to load wallet: {}", e);
            std::process::exit(1);
        }
    };

    WalletContext::with_evm(wallet, async {
        let agent = create_ethereum_agent(None).expect("Failed to create agent");

        match agent
            .prompt("send 10 wei to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
        {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("Transaction failed: {}", e),
        }

        match agent
            .prompt("send 0.0001 usdc to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
        {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("Transaction failed: {}", e),
        }
    })
    .await;
}
