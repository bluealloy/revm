use crate::Address;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaikoFields {
    pub treasury: Address,
    pub is_anchor: bool,
}
