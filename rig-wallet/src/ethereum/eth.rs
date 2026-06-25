use rig_wallet_executor::evm::eip1559::Eip1559Transfer;
use rig_wallet_types::chain_config::ChainConfig;

#[derive(Clone, ChainConfig)]
#[mainnet_rpc = "https://mainnet.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[testnet_rpc = "https://sepolia.infura.io/v3/a344f746a9854beeb4ee386cafc64cb8"]
#[mainnet_chainid = "1"]
#[testnet_chainid = "11155111"]
pub struct ETHConfig;
pub type ETH = Eip1559Transfer<ETHConfig>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rig_wallet_types::{
        request::TxRequest,
        transaction::Transaction,
        wallet::{EVMWallet, ProviderWallet, WalletContext},
    };

    use crate::ethereum::eth::ETH;

    #[tokio::test]
    async fn transfer_eth() {
        dotenvy::dotenv().ok();

        let request = TxRequest::new("0x3677572639a6b17725f04946b45A12E6443344F5", 10);

        WalletContext::with_evm(Arc::new(EVMWallet::from_env()), async {
            let mut eth = ETH::new().with_request(request);
            let tx_hash = eth.send().await.unwrap();
            println!("tx_hash: {tx_hash}");
        })
        .await;
    }
}
