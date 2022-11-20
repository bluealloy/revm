use derive_more::{AsRef, Deref};
use fixed_hash::{construct_fixed_hash, impl_fixed_hash_conversions};
//use rlp::{RlpDecodable,RlpEncodable};

construct_fixed_hash! {
    /// My 256 bit hash type.
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    //#[derive(RlpDecodable, RlpEncodable)]
    #[derive(AsRef,Deref)]
    pub struct B256(32);
}

construct_fixed_hash! {
    /// My 160 bit hash type.
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(AsRef,Deref)]
    pub struct B160(20);
}

impl From<u64> for B160 {
    fn from(fr: u64) -> Self {
        let x_bytes = fr.to_be_bytes();
        let y_bytes = 0u128.to_be_bytes();
        B160([
            x_bytes[0],
            x_bytes[1],
            x_bytes[2],
            x_bytes[3],
            x_bytes[4],
            x_bytes[5],
            x_bytes[6],
            x_bytes[7],
            x_bytes[4],
            y_bytes[5],
            y_bytes[6],
            y_bytes[7],
            y_bytes[8],
            y_bytes[9],
            y_bytes[10],
            y_bytes[11],
            y_bytes[12],
            y_bytes[13],
            y_bytes[14],
            y_bytes[15],
        ])
    }
}

impl_fixed_hash_conversions!(B256, B160);
