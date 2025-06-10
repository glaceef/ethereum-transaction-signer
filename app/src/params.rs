use crate::de::{deserialize_hex_bytes, deserialize_u256};
use ethereum_types::{H160, U256};
use serde::Deserialize;
use std::path::Path;

// params.json で渡すパラメータ
#[derive(Debug, Deserialize)]
pub struct Params {
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce: U256,
    pub to_address: H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub value: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit: U256,
    #[serde(default, deserialize_with = "deserialize_hex_bytes")]
    pub input: Vec<u8>,
}

impl Params {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let json_content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json_content).unwrap()
    }
}
