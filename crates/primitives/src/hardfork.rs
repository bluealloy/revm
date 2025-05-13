#![allow(non_camel_case_types)]
// enumn has missing docs. Should be replaced in the future https://github.com/bluealloy/revm/issues/2402
#![allow(missing_docs)]

use core::str::FromStr;
pub use num_enum::TryFromPrimitive;
pub use std::string::{String, ToString};
pub use SpecId::*;

/// Specification IDs and their activation block
///
/// Information was obtained from the [Ethereum Execution Specifications](https://github.com/ethereum/execution-specs).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecId {
    /// Frontier hard fork
    /// Activated at block 0
    FRONTIER = 0,
    /// Frontier Thawing hard fork
    /// Activated at block 200000
    FRONTIER_THAWING,
    /// Homestead hard fork
    /// Activated at block 1150000
    HOMESTEAD,
    /// DAO Fork hard fork
    /// Activated at block 1920000
    DAO_FORK,
    /// Tangerine Whistle hard fork
    /// Activated at block 2463000
    TANGERINE,
    /// Spurious Dragon hard fork
    /// Activated at block 2675000
    SPURIOUS_DRAGON,
    /// Byzantium hard fork
    /// Activated at block 4370000
    BYZANTIUM,
    /// Constantinople hard fork
    /// Activated at block 7280000 is overwritten with PETERSBURG
    CONSTANTINOPLE,
    /// Petersburg hard fork
    /// Activated at block 7280000
    PETERSBURG,
    /// Istanbul hard fork
    /// Activated at block 9069000
    ISTANBUL,
    /// Muir Glacier hard fork
    /// Activated at block 9200000
    MUIR_GLACIER,
    /// Berlin hard fork
    /// Activated at block 12244000
    BERLIN,
    /// London hard fork
    /// Activated at block 12965000
    LONDON,
    /// Arrow Glacier hard fork
    /// Activated at block 13773000
    ARROW_GLACIER,
    /// Gray Glacier hard fork
    /// Activated at block 15050000
    GRAY_GLACIER,
    /// Paris/Merge hard fork
    /// Activated at block 15537394 (TTD: 58750000000000000000000)
    MERGE,
    /// Shanghai hard fork
    /// Activated at block 17034870 (Timestamp: 1681338455)
    SHANGHAI,
    /// Cancun hard fork
    /// Activated at block 19426587 (Timestamp: 1710338135)
    CANCUN,
    /// Prague hard fork
    /// Activated at block 22431086 (Timestamp: 1746612311)
    #[default]
    PRAGUE,
    /// Osaka hard fork
    /// Activated at block TBD
    OSAKA,
}

impl SpecId {
    /// Returns the [`SpecId`] for the given [`u8`].
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::try_from(spec_id).ok()
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        self as u8 >= other as u8
    }
}

/// String identifiers for hardforks.
pub mod name {
    /// String identifier for the Frontier hardfork
    pub const FRONTIER: &str = "Frontier";
    /// String identifier for the Frontier Thawing hardfork
    pub const FRONTIER_THAWING: &str = "Frontier Thawing";
    /// String identifier for the Homestead hardfork
    pub const HOMESTEAD: &str = "Homestead";
    /// String identifier for the DAO Fork hardfork
    pub const DAO_FORK: &str = "DAO Fork";
    /// String identifier for the Tangerine Whistle hardfork
    pub const TANGERINE: &str = "Tangerine";
    /// String identifier for the Spurious Dragon hardfork
    pub const SPURIOUS_DRAGON: &str = "Spurious";
    /// String identifier for the Byzantium hardfork
    pub const BYZANTIUM: &str = "Byzantium";
    /// String identifier for the Constantinople hardfork
    pub const CONSTANTINOPLE: &str = "Constantinople";
    /// String identifier for the Petersburg hardfork
    pub const PETERSBURG: &str = "Petersburg";
    /// String identifier for the Istanbul hardfork
    pub const ISTANBUL: &str = "Istanbul";
    /// String identifier for the Muir Glacier hardfork
    pub const MUIR_GLACIER: &str = "MuirGlacier";
    /// String identifier for the Berlin hardfork
    pub const BERLIN: &str = "Berlin";
    /// String identifier for the London hardfork
    pub const LONDON: &str = "London";
    /// String identifier for the Arrow Glacier hardfork
    pub const ARROW_GLACIER: &str = "Arrow Glacier";
    /// String identifier for the Gray Glacier hardfork
    pub const GRAY_GLACIER: &str = "Gray Glacier";
    /// String identifier for the Paris/Merge hardfork
    pub const MERGE: &str = "Merge";
    /// String identifier for the Shanghai hardfork
    pub const SHANGHAI: &str = "Shanghai";
    /// String identifier for the Cancun hardfork
    pub const CANCUN: &str = "Cancun";
    /// String identifier for the Prague hardfork
    pub const PRAGUE: &str = "Prague";
    /// String identifier for the Osaka hardfork (Prague with EOF)
    pub const OSAKA: &str = "Osaka";
    /// String identifier for the latest hardfork
    pub const LATEST: &str = "Latest";
}

/// Error type for unknown hardfork names. Returned by [`SpecId::from_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownHardfork;

impl FromStr for SpecId {
    type Err = UnknownHardfork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            name::FRONTIER => Ok(Self::FRONTIER),
            name::FRONTIER_THAWING => Ok(Self::FRONTIER_THAWING),
            name::HOMESTEAD => Ok(Self::HOMESTEAD),
            name::DAO_FORK => Ok(Self::DAO_FORK),
            name::TANGERINE => Ok(Self::TANGERINE),
            name::SPURIOUS_DRAGON => Ok(Self::SPURIOUS_DRAGON),
            name::BYZANTIUM => Ok(Self::BYZANTIUM),
            name::CONSTANTINOPLE => Ok(Self::CONSTANTINOPLE),
            name::PETERSBURG => Ok(Self::PETERSBURG),
            name::ISTANBUL => Ok(Self::ISTANBUL),
            name::MUIR_GLACIER => Ok(Self::MUIR_GLACIER),
            name::BERLIN => Ok(Self::BERLIN),
            name::LONDON => Ok(Self::LONDON),
            name::ARROW_GLACIER => Ok(Self::ARROW_GLACIER),
            name::GRAY_GLACIER => Ok(Self::GRAY_GLACIER),
            name::MERGE => Ok(Self::MERGE),
            name::SHANGHAI => Ok(Self::SHANGHAI),
            name::CANCUN => Ok(Self::CANCUN),
            name::PRAGUE => Ok(Self::PRAGUE),
            name::OSAKA => Ok(Self::OSAKA),
            _ => Err(UnknownHardfork),
        }
    }
}

impl From<SpecId> for &'static str {
    fn from(spec_id: SpecId) -> Self {
        match spec_id {
            SpecId::FRONTIER => name::FRONTIER,
            SpecId::FRONTIER_THAWING => name::FRONTIER_THAWING,
            SpecId::HOMESTEAD => name::HOMESTEAD,
            SpecId::DAO_FORK => name::DAO_FORK,
            SpecId::TANGERINE => name::TANGERINE,
            SpecId::SPURIOUS_DRAGON => name::SPURIOUS_DRAGON,
            SpecId::BYZANTIUM => name::BYZANTIUM,
            SpecId::CONSTANTINOPLE => name::CONSTANTINOPLE,
            SpecId::PETERSBURG => name::PETERSBURG,
            SpecId::ISTANBUL => name::ISTANBUL,
            SpecId::MUIR_GLACIER => name::MUIR_GLACIER,
            SpecId::BERLIN => name::BERLIN,
            SpecId::LONDON => name::LONDON,
            SpecId::ARROW_GLACIER => name::ARROW_GLACIER,
            SpecId::GRAY_GLACIER => name::GRAY_GLACIER,
            SpecId::MERGE => name::MERGE,
            SpecId::SHANGHAI => name::SHANGHAI,
            SpecId::CANCUN => name::CANCUN,
            SpecId::PRAGUE => name::PRAGUE,
            SpecId::OSAKA => name::OSAKA,
        }
    }
}

impl core::fmt::Display for SpecId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}
