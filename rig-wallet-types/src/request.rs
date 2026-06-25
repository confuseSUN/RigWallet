use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TransferToolArgs {
    pub to: String,
    pub value: u128,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxRequest {
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
}

impl TxRequest {
    pub fn new(to: impl Into<String>, value: u128) -> Self {
        Self {
            to: to.into(),
            value,
            data: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = data.into();
        self
    }
}
