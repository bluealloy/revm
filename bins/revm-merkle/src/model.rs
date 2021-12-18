//use

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use serde::{
    de::{self, Error},
    Deserialize,
};

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct State(pub HashMap<H160, AccountInfo>);

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub balance: U256,
    #[serde(deserialize_with = "deserialize_str_as_bytes")]
    pub code: Bytes,
    //#[serde(deserialize_with = "deserialize_str_as_u64")]
    pub nonce: u64,
    pub storage: HashMap<H256, H256>,
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

// pub fn deserialize_str_as_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
// where
//     D: de::Deserializer<'de>,
// {
//     let string = String::deserialize(deserializer)?;

//     let output = if let Some(stripped) = string.strip_prefix("0x") {
//         u64::from_str_radix(stripped, 16).unwrap()
//     } else {
//         string.parse().unwrap()
//     };

//     Ok(output)
// }
