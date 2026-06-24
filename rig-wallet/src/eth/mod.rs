use rig_wallet_types::chain_config::ChainConfig;
use rig_wallet_executor::evm::eip1559::Eip1559Transfer;

#[derive(ChainConfig)]
#[mainnet_rpc = "https://mainnet.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[testnet_rpc = "https://sepolia.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[mainnet_chainid = "1"]
#[testnet_chainid = "11155111"]
pub struct ETHConfig;
pub type ETH = Eip1559Transfer<ETHConfig>;

#[cfg(test)]
mod tests {
    use rig_core::{
        client::{CompletionClient, ProviderClient},
        completion::Prompt,
        providers::openai,
    };
    use rig_wallet_types::{request::TxRequest, transaction::Transaction, derive_evm_wallet};

    use crate::eth::ETH;

    const DEMO_MNEMONIC: &str =
        "critic ability galaxy moon miracle film domain ritual wasp coconut torch anxiety";
    const DEMO_PATH: &str = "m/44'/60'/0'/0/0";

    #[tokio::test]
    async fn transfer_eth() {
        let wallet = derive_evm_wallet(DEMO_MNEMONIC, DEMO_PATH).unwrap();
        let request = TxRequest::new("0x3677572639a6b17725f04946b45A12E6443344F5", 10);
        let mut eth = ETH::new(request, wallet.clone());
        let tx_hash = eth.send(&wallet).await.unwrap();
        println!("tx_hash: {tx_hash}");
    }

    #[tokio::test]
    async fn transfer_with_agent() {
        dotenvy::dotenv().ok();

        let wallet = derive_evm_wallet(DEMO_MNEMONIC, DEMO_PATH).unwrap();
        let eth = ETH::from_wallet(wallet);

        let agent = openai::Client::from_env()
            .unwrap()
            .agent("qwen3.6-plus")
            .preamble("You are an Ethereum wallet agent.")
            .tool(eth)
            .max_tokens(1024)
            .build();

        let response = agent
            .prompt("send 10 wei to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
            .unwrap();
        println!("{response}");
    }
}
