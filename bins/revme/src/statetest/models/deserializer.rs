use std::str::FromStr;

use bytes::Bytes;
use revm::{B160, U256};
use serde::{
    de::{self, Error},
    Deserialize,
};

pub fn deserialize_str_as_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let output = if let Some(stripped) = string.strip_prefix("0x") {
        u64::from_str_radix(stripped, 16).unwrap()
    } else {
        string.parse().unwrap()
    };

    Ok(output)
}

pub fn deserialize_str_as_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let output = string.parse().unwrap();

    Ok(output)
}

pub fn deserialize_vec_as_vec_bytes<'de, D>(deserializer: D) -> Result<Vec<Bytes>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let strings: Vec<String> = Vec::<String>::deserialize(deserializer)?;

    let mut out = Vec::new();
    for string in strings {
        out.push(
            hex::decode(string.strip_prefix("0x").unwrap_or(&string))
                .map_err(D::Error::custom)?
                .into(),
        )
    }
    Ok(out)
}

pub fn deserialize_maybe_empty<'de, D>(deserializer: D) -> Result<Option<B160>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string: String = String::deserialize(deserializer)?;
    if string.is_empty() {
        return Ok(None);
    }
    Ok(Some(B160::from_str(&string).map_err(D::Error::custom)?))
}

pub fn deserialize_str_as_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(hex::decode(s.strip_prefix("0x").unwrap_or(&s))
        .map_err(D::Error::custom)?
        .into())
}

pub fn deserialize_opt_str_as_bytes<'de, D>(deserializer: D) -> Result<Option<Bytes>, D::Error>
where
    D: de::Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct WrappedValue(#[serde(deserialize_with = "deserialize_str_as_bytes")] Bytes);

    Option::<WrappedValue>::deserialize(deserializer)
        .map(|opt_wrapped: Option<WrappedValue>| opt_wrapped.map(|wrapped: WrappedValue| wrapped.0))
}
