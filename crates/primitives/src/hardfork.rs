#![allow(non_camel_case_types)]

use core::str::FromStr;
pub use std::string::{String, ToString};
pub use SpecId::*;

/// Specification IDs and their activation block
///
/// Information was obtained from the [Ethereum Execution Specifications](https://github.com/ethereum/execution-specs).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecId {
    FRONTIER = 0,     // Frontier               0
    FRONTIER_THAWING, // Frontier Thawing       200000
    HOMESTEAD,        // Homestead              1150000
    DAO_FORK,         // DAO Fork               1920000
    TANGERINE,        // Tangerine Whistle      2463000
    SPURIOUS_DRAGON,  // Spurious Dragon        2675000
    BYZANTIUM,        // Byzantium              4370000
    CONSTANTINOPLE,   // Constantinople         7280000 is overwritten with PETERSBURG
    PETERSBURG,       // Petersburg             7280000
    ISTANBUL,         // Istanbul	            9069000
    MUIR_GLACIER,     // Muir Glacier           9200000
    BERLIN,           // Berlin	                12244000
    LONDON,           // London	                12965000
    ARROW_GLACIER,    // Arrow Glacier          13773000
    GRAY_GLACIER,     // Gray Glacier           15050000
    MERGE,            // Paris/Merge            15537394 (TTD: 58750000000000000000000)
    SHANGHAI,         // Shanghai               17034870 (Timestamp: 1681338455)
    CANCUN,           // Cancun                 19426587 (Timestamp: 1710338135)
    PRAGUE,           // Prague                 TBD
    OSAKA,            // Osaka                  TBD
    #[default]
    LATEST = u8::MAX,
}

impl SpecId {
    /// Returns the [`SpecId`] for the given [`u8`].
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        self as u8 >= other as u8
    }
}

/// String identifiers for hardforks.
pub mod name {
    pub const FRONTIER: &str = "Frontier";
    pub const FRONTIER_THAWING: &str = "Frontier Thawing";
    pub const HOMESTEAD: &str = "Homestead";
    pub const DAO_FORK: &str = "DAO Fork";
    pub const TANGERINE: &str = "Tangerine";
    pub const SPURIOUS_DRAGON: &str = "Spurious";
    pub const BYZANTIUM: &str = "Byzantium";
    pub const CONSTANTINOPLE: &str = "Constantinople";
    pub const PETERSBURG: &str = "Petersburg";
    pub const ISTANBUL: &str = "Istanbul";
    pub const MUIR_GLACIER: &str = "MuirGlacier";
    pub const BERLIN: &str = "Berlin";
    pub const LONDON: &str = "London";
    pub const ARROW_GLACIER: &str = "Arrow Glacier";
    pub const GRAY_GLACIER: &str = "Gray Glacier";
    pub const MERGE: &str = "Merge";
    pub const SHANGHAI: &str = "Shanghai";
    pub const CANCUN: &str = "Cancun";
    pub const PRAGUE: &str = "Prague";
    pub const OSAKA: &str = "PragueEOF";
    pub const LATEST: &str = "Latest";
}

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
            name::LATEST => Ok(Self::LATEST),
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
            SpecId::LATEST => name::LATEST,
        }
    }
}

impl core::fmt::Display for SpecId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}
