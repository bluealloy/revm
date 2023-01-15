use crate::{bytes::Bytes, B160, B256};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: B160,
    pub topics: Vec<B256>,
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub data: Bytes,
}
