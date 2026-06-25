use std::sync::Arc;

use rig_core::completion::Prompt;
use rig_wallet::ethereum::create_ethereum_agent;
use rig_wallet_types::wallet::{EVMWallet, ProviderWallet, WalletContext};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let wallet = Arc::new(EVMWallet::from_env());

    WalletContext::with_evm(wallet, async {
        let agent = create_ethereum_agent(None).unwrap();

        let response = agent
            .prompt("send 10 wei to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
            .unwrap();
        println!("{response}");

        println!();

        let response = agent
            .prompt("send 0.0001 usdc to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
            .unwrap();
        println!("{response}");
    })
    .await;
}
