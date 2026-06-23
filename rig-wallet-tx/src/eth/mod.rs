use rig_wallet_types::chain_config::ChainConfig;
use rig_wallet_vm::evm::eip1559::EIP1559;

#[derive(ChainConfig)]
#[mainnet_rpc = "https://mainnet.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[testnet_rpc = "https://sepolia.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[mainnet_chainid = "1"]
#[testnet_chainid = "11155111"]
pub struct ETHConfig;

pub type TransferEth = EIP1559<ETHConfig>;

#[cfg(test)]
mod tests {
    use rig_core::{
        client::{CompletionClient, ProviderClient},
        completion::Prompt,
        providers::openai,
    };
    use rig_wallet_types::{request::TxRequest, transaction::Transaction, wallet::gen_evm_wallet};

    use crate::eth::TransferEth;

    #[tokio::test]
    async fn test_eth_normal() {
        let request = TxRequest {
            to: "0x3677572639a6b17725f04946b45A12E6443344F5".to_string(),
            value: 10,
            data: vec![],
        };

        let wallet = gen_evm_wallet(
            "critic ability galaxy moon miracle film domain ritual wasp coconut torch anxiety",
            "m/44'/60'/0'/0/0",
        );

        println!("address: {:?}", wallet.address);

        let mut eth = TransferEth::new(request, wallet.clone());
        let signatures = eth.build_and_sign(&[wallet]).await.unwrap();
        let tx_hash = eth.send(signatures).await.unwrap();
        println!("tx_hash: {:?}\n", tx_hash);
    }

    #[tokio::test]
    async fn test_eth_agent() {
        dotenvy::dotenv().ok();

        let wallet = gen_evm_wallet(
            "critic ability galaxy moon miracle film domain ritual wasp coconut torch anxiety",
            "m/44'/60'/0'/0/0",
        );
        let transfer_eth = TransferEth::from_wallet(wallet);

        let agent = openai::Client::from_env()
            .unwrap()
            .agent("qwen3.6-plus")
            .preamble("you are an ethereum trading agent")
            .tool(transfer_eth)
            .max_tokens(1024)
            .build();

        let response = agent
            .prompt("send 10 wei to 0x3677572639a6b17725f04946b45A12E6443344F5")
            .await
            .unwrap();
        println!("{response}");
    }
}
