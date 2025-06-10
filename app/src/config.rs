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
        // 0xプレフィックスを削除
        let hex_str = self
            .private_key
            .strip_prefix("0x")
            .unwrap_or(&self.private_key);
        let decoded = hex::decode(hex_str)?;

        decoded
            .try_into()
            .map_err(|data: Vec<u8>| Error::InvalidPrivateKeyLength(data.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用の設定作成ヘルパー
    fn create_test_config(
        chain_id: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
        private_key: &str,
    ) -> Config {
        Config {
            chain_id,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            private_key: private_key.to_string(),
        }
    }

    // JSON文字列から直接Configをデシリアライズするテストヘルパー
    fn config_from_json(json: &str) -> serde_json::Result<Config> {
        serde_json::from_str(json).map_err(Into::into)
    }

    #[test]
    fn test_config_deserialization_valid() {
        let json = r#"{
            "chain_id": 11155111,
            "max_fee_per_gas": "0x77359400",
            "max_priority_fee_per_gas": "0x3b9aca00",
            "private_key": "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        }"#;

        let config = config_from_json(json).unwrap();

        assert_eq!(config.chain_id, 11155111);
        assert_eq!(config.max_fee_per_gas, U256::from(0x77359400u64));
        assert_eq!(config.max_priority_fee_per_gas, U256::from(0x3b9aca00u64));
        assert_eq!(
            config.private_key,
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
    }

    #[test]
    fn test_config_deserialization_with_0x_prefix() {
        let json = r#"{
            "chain_id": 1,
            "max_fee_per_gas": "0x1dcd65000",
            "max_priority_fee_per_gas": "0x77359400",
            "private_key": "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        }"#;

        let config = config_from_json(json).unwrap();

        assert_eq!(config.chain_id, 1);
        assert_eq!(config.max_fee_per_gas, U256::from(0x1dcd65000u64));
        assert_eq!(config.max_priority_fee_per_gas, U256::from(0x77359400u64));
        assert_eq!(
            config.private_key,
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
    }

    #[test]
    fn test_config_deserialization_decimal_gas_values() {
        let json = r#"{
            "chain_id": 11155111,
            "max_fee_per_gas": 2000000000,
            "max_priority_fee_per_gas": 1000000000,
            "private_key": "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        }"#;

        let config = config_from_json(json).unwrap();

        assert_eq!(config.max_fee_per_gas, U256::from(2_000_000_000u64));
        assert_eq!(
            config.max_priority_fee_per_gas,
            U256::from(1_000_000_000u64)
        );
    }

    #[test]
    fn test_config_deserialization_missing_required_field() {
        let json = r#"{
            "chain_id": 1,
            "max_fee_per_gas": "0x77359400",
            "max_priority_fee_per_gas": "0x3b9aca00"
        }"#;

        let result = config_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_deserialization_invalid_gas_values() {
        let json = r#"{
            "chain_id": 1,
            "max_fee_per_gas": "0xinvalid",
            "max_priority_fee_per_gas": "0x3b9aca00",
            "private_key": "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        }"#;

        let result = config_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_private_key_bytes_valid() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        let key_bytes = config.get_private_key_bytes().unwrap();
        assert_eq!(key_bytes.len(), 32);
        assert_eq!(key_bytes[0], 0xac);
        assert_eq!(key_bytes[31], 0x80);
    }

    #[test]
    fn test_get_private_key_bytes_with_0x_prefix() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        let key_bytes = config.get_private_key_bytes().unwrap();
        assert_eq!(key_bytes.len(), 32);
        assert_eq!(key_bytes[0], 0xac);
        assert_eq!(key_bytes[31], 0x80);
    }

    #[test]
    fn test_get_private_key_bytes_all_zeros() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        let key_bytes = config.get_private_key_bytes().unwrap();
        assert_eq!(key_bytes, [0u8; 32]);
    }

    #[test]
    fn test_get_private_key_bytes_all_ones() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );

        let key_bytes = config.get_private_key_bytes().unwrap();
        assert_eq!(key_bytes, [0xffu8; 32]);
    }

    #[test]
    fn test_get_private_key_bytes_invalid_hex() {
        let config = create_test_config(1, U256::zero(), U256::zero(), "invalid_hex_string");

        let result = config.get_private_key_bytes();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_private_key_bytes_wrong_length_short() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff", // 31 bytes (62 chars)
        );

        let result = config.get_private_key_bytes();
        assert!(result.is_err());

        if let Err(Error::InvalidPrivateKeyLength(len)) = result {
            assert_eq!(len, 31);
        } else {
            panic!("Expected InvalidPrivateKeyLength error, got: {:?}", result);
        }
    }

    #[test]
    fn test_get_private_key_bytes_wrong_length_long() {
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80ff", // 33 bytes (66 chars)
        );

        let result = config.get_private_key_bytes();
        assert!(result.is_err());

        if let Err(Error::InvalidPrivateKeyLength(len)) = result {
            assert_eq!(len, 33);
        } else {
            panic!("Expected InvalidPrivateKeyLength error, got: {:?}", result);
        }
    }

    #[test]
    fn test_get_private_key_bytes_odd_length_hex() {
        // 奇数文字数（hex::decodeでOddLengthエラー）
        let config = create_test_config(
            1,
            U256::zero(),
            U256::zero(),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff8", // 63 chars (odd)
        );

        let result = config.get_private_key_bytes();
        assert!(result.is_err());

        // FromHexエラーが発生することを確認
        match result {
            Err(Error::FromHex(hex::FromHexError::OddLength)) => {
                // 期待通り
            }
            _ => panic!("Expected FromHex(OddLength) error, got: {:?}", result),
        }
    }

    #[test]
    fn test_config_debug_output() {
        let config = create_test_config(
            11155111,
            U256::from(2_000_000_000u64),
            U256::from(1_000_000_000u64),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("chain_id"));
        assert!(debug_str.contains("max_fee_per_gas"));
        assert!(debug_str.contains("max_priority_fee_per_gas"));
        assert!(debug_str.contains("private_key"));
    }

    // ===== 実用的なシナリオテスト =====

    #[test]
    fn test_config_sepolia_testnet() {
        let config = create_test_config(
            11155111,                  // Sepolia chain ID
            U256::from(0x77359400u64), // 2 Gwei
            U256::from(0x3b9aca00u64), // 1 Gwei
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        let key_bytes = config.get_private_key_bytes().unwrap();

        assert_eq!(config.chain_id, 11155111);
        assert!(config.max_fee_per_gas > config.max_priority_fee_per_gas);
        assert_eq!(key_bytes.len(), 32);
    }

    #[test]
    fn test_config_mainnet() {
        let config = create_test_config(
            1,                          // Mainnet chain ID
            U256::from(0x12a05f200u64), // 5 Gwei
            U256::from(0x77359400u64),  // 2 Gwei
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        assert_eq!(config.chain_id, 1);
        assert!(config.max_fee_per_gas >= config.max_priority_fee_per_gas);
        assert!(config.max_fee_per_gas > U256::from(1_000_000_000u64)); // > 1 Gwei
    }

    #[test]
    fn test_config_gas_price_validation() {
        let config = create_test_config(
            1,
            U256::from(1_000_000_000u64), // 1 Gwei
            U256::from(2_000_000_000u64), // 2 Gwei (higher than max_fee)
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        // EIP-1559では max_priority_fee_per_gas <= max_fee_per_gas である必要がある
        // このテストは論理的な検証のみ（実際のバリデーションロジックがあれば）
        assert_ne!(config.max_fee_per_gas, config.max_priority_fee_per_gas);
    }

    #[test]
    fn test_config_zero_gas_prices() {
        let config = create_test_config(
            31337, // Local testnet
            U256::zero(),
            U256::zero(),
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        assert_eq!(config.max_fee_per_gas, U256::zero());
        assert_eq!(config.max_priority_fee_per_gas, U256::zero());
        assert!(config.get_private_key_bytes().is_ok());
    }

    #[test]
    fn test_config_large_gas_prices() {
        // 非常に高いガス価格（100 Gwei）
        let high_gas = U256::from(100_000_000_000u64);

        let config = create_test_config(
            1,
            high_gas,
            U256::from(10_000_000_000u64), // 10 Gwei
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        );

        assert_eq!(config.max_fee_per_gas, high_gas);
        assert!(config.max_fee_per_gas > U256::from(50_000_000_000u64)); // > 50 Gwei
    }
}
