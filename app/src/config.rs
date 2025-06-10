use crate::{Result, de::deserialize_u256, error::Error};
use ethereum_types::U256;
use serde::Deserialize;

// 環境変数パラメータ
#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain_id: u64,
    #[serde(deserialize_with = "deserialize_u256")]
    pub max_fee_per_gas: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub max_priority_fee_per_gas: U256,
    pub private_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }

    pub fn get_private_key_bytes(&self) -> Result<[u8; 32]> {
        let decoded = hex::decode(&self.private_key)?;

        decoded
            .try_into()
            .map_err(|data: Vec<u8>| Error::InvalidPrivateKeyLength(data.len()))
    }
}
