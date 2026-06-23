use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxRequest {
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
}

impl TxRequest {
    pub fn new(to: String, value: u128) -> Self {
        Self {
            to,
            value,
            data: vec![],
        }
    }
}
