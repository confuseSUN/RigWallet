use std::{future::Future, sync::Arc};

use crate::errors::{Error, Result};

use super::{EVMWallet, SVMWallet};

tokio::task_local! {
    static EVM_WALLET: Arc<EVMWallet>;
    static SVM_WALLET: Arc<SVMWallet>;
}

pub struct WalletContext;

impl WalletContext {
    pub async fn with_evm<T>(wallet: Arc<EVMWallet>, f: impl Future<Output = T> + Send) -> T {
        EVM_WALLET.scope(wallet, f).await
    }

    pub async fn with_svm<T>(wallet: Arc<SVMWallet>, f: impl Future<Output = T> + Send) -> T {
        SVM_WALLET.scope(wallet, f).await
    }

    pub fn evm() -> Result<Arc<EVMWallet>> {
        EVM_WALLET
            .try_with(|wallet| wallet.clone())
            .map_err(|_| Error::WalletNotInScope)
    }

    pub fn svm() -> Result<Arc<SVMWallet>> {
        SVM_WALLET
            .try_with(|wallet| wallet.clone())
            .map_err(|_| Error::WalletNotInScope)
    }
}
