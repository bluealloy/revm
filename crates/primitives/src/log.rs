use crate::{Address, Bytes, B256};
use alloc::vec::Vec;
use alloy_rlp::{RlpDecodable, RlpEncodable};

#[derive(Clone, Debug, Default, PartialEq, Eq, RlpDecodable, RlpEncodable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
}
