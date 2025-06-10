use ethereum_types::U256;
use serde::{Deserialize, Deserializer};

pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(n) => Ok(U256::from(n.as_u64().unwrap_or(0))),
        serde_json::Value::String(s) => {
            U256::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
        }
        _ => Err(serde::de::Error::custom("Expected number or hex string")),
    }
}

pub fn deserialize_hex_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;

    let trimmed_hex_string = hex_string.strip_prefix("0x").unwrap_or(&hex_string);
    if trimmed_hex_string.is_empty() {
        Ok(vec![])
    } else {
        hex::decode(trimmed_hex_string).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // Test helper: JSON値からデシリアライズするヘルパー関数
    fn test_deserialize_u256_from_json(json_value: &str) -> Result<U256, serde_json::Error> {
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_u256")]
            value: U256,
        }

        let json = format!(r#"{{"value": {}}}"#, json_value);
        let result: TestStruct = serde_json::from_str(&json)?;
        Ok(result.value)
    }

    fn test_deserialize_hex_bytes_from_json(
        json_value: &str,
    ) -> Result<Vec<u8>, serde_json::Error> {
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_hex_bytes")]
            value: Vec<u8>,
        }

        let json = format!(r#"{{"value": {}}}"#, json_value);
        let result: TestStruct = serde_json::from_str(&json)?;
        Ok(result.value)
    }

    // ===== deserialize_u256 のテスト =====

    #[test]
    fn test_deserialize_u256_hex_string() {
        assert_eq!(
            test_deserialize_u256_from_json(r#""0x0""#).unwrap(),
            U256::zero()
        );
        assert_eq!(
            test_deserialize_u256_from_json(r#""0x1""#).unwrap(),
            U256::one()
        );
        assert_eq!(
            test_deserialize_u256_from_json(r#""0xff""#).unwrap(),
            U256::from(255)
        );
        assert_eq!(
            test_deserialize_u256_from_json(r#""0x5208""#).unwrap(),
            U256::from(21000)
        );
    }

    #[test]
    fn test_deserialize_u256_hex_string_large() {
        // 1 ETH in wei (0xde0b6b3a7640000)
        let expected = U256::from_str_radix("de0b6b3a7640000", 16).unwrap();
        assert_eq!(
            test_deserialize_u256_from_json(r#""0xde0b6b3a7640000""#).unwrap(),
            expected
        );

        // Max U256
        let max_hex = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(
            test_deserialize_u256_from_json(&format!(r#""{}""#, max_hex)).unwrap(),
            U256::MAX
        );
    }

    #[test]
    fn test_deserialize_u256_hex_string_without_prefix() {
        // 0xプレフィックスなしでも動作することを確認
        assert_eq!(
            test_deserialize_u256_from_json(r#""ff""#).unwrap(),
            U256::from(255)
        );
        assert_eq!(
            test_deserialize_u256_from_json(r#""5208""#).unwrap(),
            U256::from(21000)
        );
    }

    #[test]
    fn test_deserialize_u256_number() {
        assert_eq!(test_deserialize_u256_from_json("0").unwrap(), U256::zero());
        assert_eq!(test_deserialize_u256_from_json("1").unwrap(), U256::one());
        assert_eq!(
            test_deserialize_u256_from_json("255").unwrap(),
            U256::from(255)
        );
        assert_eq!(
            test_deserialize_u256_from_json("21000").unwrap(),
            U256::from(21000)
        );
    }

    #[test]
    fn test_deserialize_u256_number_large() {
        // JavaScriptの安全な整数の範囲内
        assert_eq!(
            test_deserialize_u256_from_json("9007199254740991").unwrap(),
            U256::from(9007199254740991u64)
        );
    }

    #[test]
    fn test_deserialize_u256_invalid_hex() {
        assert!(test_deserialize_u256_from_json(r#""0xgg""#).is_err());
        assert!(test_deserialize_u256_from_json(r#""invalid""#).is_err());
    }

    #[test]
    fn test_deserialize_u256_invalid_type() {
        assert!(test_deserialize_u256_from_json("true").is_err());
        assert!(test_deserialize_u256_from_json("null").is_err());
        assert!(test_deserialize_u256_from_json("[]").is_err());
        assert!(test_deserialize_u256_from_json("{}").is_err());
    }

    // ===== deserialize_hex_bytes のテスト =====

    #[test]
    fn test_deserialize_hex_bytes_empty() {
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0x""#).unwrap(),
            Vec::<u8>::new()
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""""#).unwrap(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn test_deserialize_hex_bytes_basic() {
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0x00""#).unwrap(),
            vec![0x00]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0xff""#).unwrap(),
            vec![0xff]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0x1234""#).unwrap(),
            vec![0x12, 0x34]
        );
    }

    #[test]
    fn test_deserialize_hex_bytes_without_prefix() {
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""00""#).unwrap(),
            vec![0x00]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""ff""#).unwrap(),
            vec![0xff]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""1234""#).unwrap(),
            vec![0x12, 0x34]
        );
    }

    #[test]
    fn test_deserialize_hex_bytes_erc20_transfer() {
        // ERC20 transfer function call data
        let transfer_data = "0xa9059cbb000000000000000000000000742d35cc6634c0532925a3b8d2f8e0c4ed2d11df0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        let result =
            test_deserialize_hex_bytes_from_json(&format!(r#""{}""#, transfer_data)).unwrap();

        // Function selector should be 0xa9059cbb
        assert_eq!(result[0..4], [0xa9, 0x05, 0x9c, 0xbb]);
        assert_eq!(result.len(), 68); // 4 bytes selector + 32 bytes address + 32 bytes amount
    }

    #[test]
    fn test_deserialize_hex_bytes_long_data() {
        // 長いデータのテスト
        let long_hex = "0x".to_string() + &"ab".repeat(100);
        let result = test_deserialize_hex_bytes_from_json(&format!(r#""{}""#, long_hex)).unwrap();
        assert_eq!(result.len(), 100);
        assert!(result.iter().all(|&b| b == 0xab));
    }

    #[test]
    fn test_deserialize_hex_bytes_odd_length() {
        // 奇数長の16進数文字列（通常は無効）
        assert!(test_deserialize_hex_bytes_from_json(r#""0x123""#).is_err());
        assert!(test_deserialize_hex_bytes_from_json(r#""abc""#).is_err());
    }

    #[test]
    fn test_deserialize_hex_bytes_invalid_hex() {
        assert!(test_deserialize_hex_bytes_from_json(r#""0xgg""#).is_err());
        assert!(test_deserialize_hex_bytes_from_json(r#""0x1g""#).is_err());
        assert!(test_deserialize_hex_bytes_from_json(r#""invalid""#).is_err());
    }

    #[test]
    fn test_deserialize_hex_bytes_case_insensitive() {
        // 大文字小文字の確認
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0xAbCd""#).unwrap(),
            vec![0xab, 0xcd]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0xABCD""#).unwrap(),
            vec![0xab, 0xcd]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""abcd""#).unwrap(),
            vec![0xab, 0xcd]
        );
    }

    // ===== エッジケースとバウンダリテスト =====

    #[test]
    fn test_deserialize_u256_boundary_values() {
        // ゼロ
        assert_eq!(
            test_deserialize_u256_from_json(r#""0x0""#).unwrap(),
            U256::zero()
        );
        assert_eq!(test_deserialize_u256_from_json("0").unwrap(), U256::zero());

        // 1
        assert_eq!(
            test_deserialize_u256_from_json(r#""0x1""#).unwrap(),
            U256::one()
        );
        assert_eq!(test_deserialize_u256_from_json("1").unwrap(), U256::one());
    }

    #[test]
    fn test_deserialize_hex_bytes_boundary_values() {
        // 1バイト
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0x00""#).unwrap(),
            vec![0x00]
        );
        assert_eq!(
            test_deserialize_hex_bytes_from_json(r#""0xff""#).unwrap(),
            vec![0xff]
        );

        // 32バイト（よく使われるサイズ）
        let hash_like = "0x".to_owned() + &"a1".repeat(32);
        let result = test_deserialize_hex_bytes_from_json(&format!(r#""{}""#, hash_like)).unwrap();
        assert_eq!(result.len(), 32);
        assert!(result.iter().all(|&b| b == 0xa1));
    }
}
