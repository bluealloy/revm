use revm::primitives::hardfork::SpecId;

pub type RwasmSpecId = SpecId;

// #[repr(u8)]
// #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[allow(non_camel_case_types)]
// pub enum RwasmSpecId {
//     #[default]
//     SUPERPOSE = 0x52,
// }
//
// impl RwasmSpecId {
//     /// Converts the [`RwasmSpecId`] into a [`SpecId`].
//     pub const fn into_eth_spec(self) -> SpecId {
//         match self {
//             Self::SUPERPOSE => SpecId::PRAGUE,
//         }
//     }
//
//     pub const fn is_enabled_in(self, other: RwasmSpecId) -> bool {
//         other as u8 <= self as u8
//     }
// }
//
// impl From<RwasmSpecId> for SpecId {
//     fn from(spec: RwasmSpecId) -> Self {
//         spec.into_eth_spec()
//     }
// }
//
// impl FromStr for RwasmSpecId {
//     type Err = UnknownHardfork;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             name::SUPERPOSE => Ok(RwasmSpecId::SUPERPOSE),
//             _ => Err(UnknownHardfork),
//         }
//     }
// }
//
// impl From<RwasmSpecId> for &'static str {
//     fn from(spec_id: RwasmSpecId) -> Self {
//         match spec_id {
//             RwasmSpecId::SUPERPOSE => name::SUPERPOSE,
//         }
//     }
// }
//
// /// String identifiers for Fluent hardforks
// pub mod name {
//     pub const SUPERPOSE: &str = "Superpose";
// }
