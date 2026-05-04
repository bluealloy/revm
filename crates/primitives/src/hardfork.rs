//! Ethereum hardfork specification IDs.

#![expect(non_camel_case_types)]

use core::str::FromStr;
pub use std::string::{String, ToString};
pub use SpecId::*;

/// Specification IDs and their activation points.
///
/// Information was obtained from the [Ethereum Execution Specifications](https://github.com/ethereum/execution-specs).
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecId {
    /// Frontier
    ///
    /// Activated at block 1
    FRONTIER = 0,
    /// Homestead
    ///
    /// Activated at block 1150000
    HOMESTEAD,
    /// Tangerine Whistle
    ///
    /// Activated at block 2463000
    TANGERINE,
    /// Spurious Dragon
    ///
    /// Activated at block 2675000
    SPURIOUS_DRAGON,
    /// Byzantium
    ///
    /// Activated at block 4370000
    BYZANTIUM,
    /// Constantinople
    ///
    /// Activated at block 7280000
    CONSTANTINOPLE,
    /// Petersburg
    ///
    /// Activated at block 7280000
    PETERSBURG,
    /// Istanbul
    ///
    /// Activated at block 9069000
    ISTANBUL,
    /// Berlin
    ///
    /// Activated at block 12244000
    BERLIN,
    /// London
    ///
    /// Activated at block 12965000
    LONDON,
    /// Paris/Merge
    ///
    /// Activated at block 15537394
    MERGE,
    /// Shanghai
    ///
    /// Activated at block 17034870 (timestamp 1681338455)
    SHANGHAI,
    /// Cancun
    ///
    /// Activated at block 19426587 (timestamp 1710338135)
    CANCUN,
    /// Prague
    ///
    /// Activated at block 22431084
    PRAGUE,
    /// Osaka
    ///
    /// Activated at block 23935694
    #[default]
    OSAKA,
    /// Amsterdam
    ///
    /// Activated at block TBD
    AMSTERDAM,
}

impl SpecId {
    /// The latest known spec. This may refer to a highly experimental hard fork
    /// that is not yet finalized or deployed on any network.
    ///
    /// **Warning**: This value will change between minor versions as new hard forks are added.
    /// Do not rely on it for stable behavior.
    #[doc(alias = "MAX")]
    pub const NEXT: Self = Self::AMSTERDAM;

    /// Returns the [`SpecId`] for the given [`u8`].
    #[inline]
    pub const fn try_from_u8(spec_id: u8) -> Option<Self> {
        if spec_id <= Self::NEXT as u8 {
            // SAFETY: `spec_id` is within the valid range.
            Some(unsafe { core::mem::transmute::<u8, Self>(spec_id) })
        } else {
            None
        }
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        self as u8 >= other as u8
    }
}

impl From<SpecId> for u8 {
    #[inline]
    fn from(spec_id: SpecId) -> Self {
        spec_id as u8
    }
}

impl TryFrom<u8> for SpecId {
    type Error = u8;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from_u8(value).ok_or(value)
    }
}

/// String identifiers for hardforks.
pub mod name {
    /// String identifier for the Frontier hardfork
    pub const FRONTIER: &str = "Frontier";
    /// String identifier for the Homestead hardfork
    pub const HOMESTEAD: &str = "Homestead";
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
    /// String identifier for the Berlin hardfork
    pub const BERLIN: &str = "Berlin";
    /// String identifier for the London hardfork
    pub const LONDON: &str = "London";
    /// String identifier for the Paris/Merge hardfork
    pub const MERGE: &str = "Merge";
    /// String identifier for the Shanghai hardfork
    pub const SHANGHAI: &str = "Shanghai";
    /// String identifier for the Cancun hardfork
    pub const CANCUN: &str = "Cancun";
    /// String identifier for the Prague hardfork
    pub const PRAGUE: &str = "Prague";
    /// String identifier for the Osaka hardfork
    pub const OSAKA: &str = "Osaka";
    /// String identifier for the Amsterdam hardfork
    pub const AMSTERDAM: &str = "Amsterdam";
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
            name::HOMESTEAD => Ok(Self::HOMESTEAD),
            name::TANGERINE => Ok(Self::TANGERINE),
            name::SPURIOUS_DRAGON => Ok(Self::SPURIOUS_DRAGON),
            name::BYZANTIUM => Ok(Self::BYZANTIUM),
            name::CONSTANTINOPLE => Ok(Self::CONSTANTINOPLE),
            name::PETERSBURG => Ok(Self::PETERSBURG),
            name::ISTANBUL => Ok(Self::ISTANBUL),
            name::BERLIN => Ok(Self::BERLIN),
            name::LONDON => Ok(Self::LONDON),
            name::MERGE => Ok(Self::MERGE),
            name::SHANGHAI => Ok(Self::SHANGHAI),
            name::CANCUN => Ok(Self::CANCUN),
            name::PRAGUE => Ok(Self::PRAGUE),
            name::OSAKA => Ok(Self::OSAKA),
            name::AMSTERDAM => Ok(Self::AMSTERDAM),
            _ => Err(UnknownHardfork),
        }
    }
}

impl From<SpecId> for &'static str {
    fn from(spec_id: SpecId) -> Self {
        match spec_id {
            SpecId::FRONTIER => name::FRONTIER,
            SpecId::HOMESTEAD => name::HOMESTEAD,
            SpecId::TANGERINE => name::TANGERINE,
            SpecId::SPURIOUS_DRAGON => name::SPURIOUS_DRAGON,
            SpecId::BYZANTIUM => name::BYZANTIUM,
            SpecId::CONSTANTINOPLE => name::CONSTANTINOPLE,
            SpecId::PETERSBURG => name::PETERSBURG,
            SpecId::ISTANBUL => name::ISTANBUL,
            SpecId::BERLIN => name::BERLIN,
            SpecId::LONDON => name::LONDON,
            SpecId::MERGE => name::MERGE,
            SpecId::SHANGHAI => name::SHANGHAI,
            SpecId::CANCUN => name::CANCUN,
            SpecId::PRAGUE => name::PRAGUE,
            SpecId::OSAKA => name::OSAKA,
            SpecId::AMSTERDAM => name::AMSTERDAM,
        }
    }
}

impl core::fmt::Display for SpecId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}
