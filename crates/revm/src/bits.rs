use derive_more::{AsRef, Deref};
use fixed_hash::{construct_fixed_hash, impl_fixed_hash_conversions};

construct_fixed_hash! {
    /// My 256 bit hash type.
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(AsRef,Deref)]
    pub struct B256(32);
}

construct_fixed_hash! {
    /// My 256 bit hash type.
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(AsRef,Deref)]
    pub struct B160(20);
}

impl_fixed_hash_conversions!(B256, B160);
