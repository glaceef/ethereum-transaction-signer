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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_params_deserialization_basic() {
        let json = r#"{
            "nonce": "0x1",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x16345785d8a0000",
            "gas_limit": "0x5208"
        }"#;

        let params: Params = serde_json::from_str(json).unwrap();

        assert_eq!(params.nonce, U256::from(1));
        assert_eq!(
            params.to_address,
            "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df"
                .parse()
                .unwrap()
        );
        assert_eq!(
            params.value,
            U256::from_str_radix("16345785d8a0000", 16).unwrap()
        );
        assert_eq!(params.gas_limit, U256::from(21000)); // 0x5208 = 21000
        assert!(params.input.is_empty()); // default値
    }

    #[test]
    fn test_params_with_input_data() {
        let json = r#"{
            "nonce": "0x0",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x0",
            "gas_limit": "0x5208",
            "input": "0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b8d2f8e0c4ed2d11df0000000000000000000000000000000000000000000000000de0b6b3a7640000"
        }"#;

        let params: Params = serde_json::from_str(json).unwrap();

        assert_eq!(params.nonce, U256::zero());
        assert_eq!(params.value, U256::zero());
        assert!(!params.input.is_empty());
        // ERC20 transfer function selector (0xa9059cbb)
        assert_eq!(params.input[0..4], [0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn test_params_from_path() {
        // 一時ファイルを作成
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_content = r#"{
            "nonce": "0x42",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x1bc16d674ec80000",
            "gas_limit": "0x7530"
        }"#;

        write!(temp_file, "{}", json_content).unwrap();

        // ファイルから読み込み
        let params = Params::from_path(temp_file.path());

        assert_eq!(params.nonce, U256::from(0x42));
        assert_eq!(
            params.value,
            U256::from_str_radix("1bc16d674ec80000", 16).unwrap()
        ); // 2 ETH in wei
        assert_eq!(params.gas_limit, U256::from(30000)); // 0x7530 = 30000
    }

    #[test]
    fn test_params_large_values() {
        let json = r#"{
            "nonce": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "to_address": "0x0000000000000000000000000000000000000000",
            "value": "0xde0b6b3a7640000",
            "gas_limit": "0x1e8480"
        }"#;

        let params: Params = serde_json::from_str(json).unwrap();

        assert_eq!(params.nonce, U256::MAX);
        assert_eq!(params.to_address, H160::zero());
        assert_eq!(params.gas_limit, U256::from(2_000_000));
    }

    #[test]
    fn test_params_minimal_valid() {
        let json = r#"{
            "nonce": "0x0",
            "to_address": "0x0000000000000000000000000000000000000000",
            "value": "0x0",
            "gas_limit": "0x5208"
        }"#;

        let params: Params = serde_json::from_str(json).unwrap();

        assert_eq!(params.nonce, U256::zero());
        assert_eq!(params.to_address, H160::zero());
        assert_eq!(params.value, U256::zero());
        assert_eq!(params.gas_limit, U256::from(21000));
        assert!(params.input.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_params_missing_required_field() {
        let json = r#"{
            "nonce": "0x0",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x0"
        }"#; // gas_limit が missing

        let _: Params = serde_json::from_str(json).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_params_invalid_hex_format() {
        let json = r#"{
            "nonce": "invalid_hex",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x0",
            "gas_limit": "0x5208"
        }"#;

        let _: Params = serde_json::from_str(json).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_params_invalid_address() {
        let json = r#"{
            "nonce": "0x0",
            "to_address": "0xinvalid_address",
            "value": "0x0",
            "gas_limit": "0x5208"
        }"#;

        let _: Params = serde_json::from_str(json).unwrap();
    }

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_params_from_nonexistent_path() {
        Params::from_path("nonexistent_file.json");
    }

    #[test]
    fn test_debug_output() {
        let json = r#"{
            "nonce": "0x1",
            "to_address": "0x742d35Cc6634C0532925a3b8D2f8e0C4eD2d11Df",
            "value": "0x0",
            "gas_limit": "0x5208"
        }"#;

        let params: Params = serde_json::from_str(json).unwrap();
        let debug_str = format!("{:?}", params);

        assert!(debug_str.contains("nonce"));
        assert!(debug_str.contains("to_address"));
        assert!(debug_str.contains("value"));
        assert!(debug_str.contains("gas_limit"));
    }
}
