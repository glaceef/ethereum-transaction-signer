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
            Ok(U256::from_str_radix(&s, 16).map_err(serde::de::Error::custom)?)
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
