#![allow(non_camel_case_types)]

pub use SpecId::*;

/// Specification IDs and their activation block.
///
/// Information was obtained from the [Ethereum Execution Specifications](https://github.com/ethereum/execution-specs)
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecId {
    FRONTIER = 0,         // Frontier               0
    FRONTIER_THAWING = 1, // Frontier Thawing       200000
    HOMESTEAD = 2,        // Homestead              1150000
    DAO_FORK = 3,         // DAO Fork               1920000
    TANGERINE = 4,        // Tangerine Whistle      2463000
    SPURIOUS_DRAGON = 5,  // Spurious Dragon        2675000
    BYZANTIUM = 6,        // Byzantium              4370000
    CONSTANTINOPLE = 7,   // Constantinople         7280000 is overwritten with PETERSBURG
    PETERSBURG = 8,       // Petersburg             7280000
    ISTANBUL = 9,         // Istanbul	            9069000
    MUIR_GLACIER = 10,    // Muir Glacier           9200000
    BERLIN = 11,          // Berlin	                12244000
    LONDON = 12,          // London	                12965000
    ARROW_GLACIER = 13,   // Arrow Glacier          13773000
    GRAY_GLACIER = 14,    // Gray Glacier           15050000
    MERGE = 15,           // Paris/Merge            15537394 (TTD: 58750000000000000000000)
    SHANGHAI = 16,        // Shanghai               17034870 (Timestamp: 1681338455)
    CANCUN = 17,          // Cancun                 19426587 (Timestamp: 1710338135)
    PRAGUE = 18,          // Prague                 TBD
    PRAGUE_EOF = 19,      // Prague+EOF             TBD
    #[default]
    LATEST = u8::MAX,
}

impl SpecId {
    /// Returns the `SpecId` for the given `u8`.
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        Self::enabled(self, other)
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn enabled(our: SpecId, other: SpecId) -> bool {
        our as u8 >= other as u8
    }
}

/// String identifiers for hardforks.
pub mod id {
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
    pub const PRAGUE_EOF: &str = "PragueEOF";
    pub const LATEST: &str = "Latest";
}

impl From<&str> for SpecId {
    fn from(name: &str) -> Self {
        match name {
            id::FRONTIER => Self::FRONTIER,
            id::FRONTIER_THAWING => Self::FRONTIER_THAWING,
            id::HOMESTEAD => Self::HOMESTEAD,
            id::DAO_FORK => Self::DAO_FORK,
            id::TANGERINE => Self::TANGERINE,
            id::SPURIOUS_DRAGON => Self::SPURIOUS_DRAGON,
            id::BYZANTIUM => Self::BYZANTIUM,
            id::CONSTANTINOPLE => Self::CONSTANTINOPLE,
            id::PETERSBURG => Self::PETERSBURG,
            id::ISTANBUL => Self::ISTANBUL,
            id::MUIR_GLACIER => Self::MUIR_GLACIER,
            id::BERLIN => Self::BERLIN,
            id::LONDON => Self::LONDON,
            id::ARROW_GLACIER => Self::ARROW_GLACIER,
            id::GRAY_GLACIER => Self::GRAY_GLACIER,
            id::MERGE => Self::MERGE,
            id::SHANGHAI => Self::SHANGHAI,
            id::CANCUN => Self::CANCUN,
            id::PRAGUE => Self::PRAGUE,
            id::PRAGUE_EOF => Self::PRAGUE_EOF,
            id::LATEST => Self::LATEST,
            _ => Self::LATEST,
        }
    }
}

impl From<SpecId> for &'static str {
    fn from(spec_id: SpecId) -> Self {
        match spec_id {
            SpecId::FRONTIER => id::FRONTIER,
            SpecId::FRONTIER_THAWING => id::FRONTIER_THAWING,
            SpecId::HOMESTEAD => id::HOMESTEAD,
            SpecId::DAO_FORK => id::DAO_FORK,
            SpecId::TANGERINE => id::TANGERINE,
            SpecId::SPURIOUS_DRAGON => id::SPURIOUS_DRAGON,
            SpecId::BYZANTIUM => id::BYZANTIUM,
            SpecId::CONSTANTINOPLE => id::CONSTANTINOPLE,
            SpecId::PETERSBURG => id::PETERSBURG,
            SpecId::ISTANBUL => id::ISTANBUL,
            SpecId::MUIR_GLACIER => id::MUIR_GLACIER,
            SpecId::BERLIN => id::BERLIN,
            SpecId::LONDON => id::LONDON,
            SpecId::ARROW_GLACIER => id::ARROW_GLACIER,
            SpecId::GRAY_GLACIER => id::GRAY_GLACIER,
            SpecId::MERGE => id::MERGE,
            SpecId::SHANGHAI => id::SHANGHAI,
            SpecId::CANCUN => id::CANCUN,
            SpecId::PRAGUE => id::PRAGUE,
            SpecId::PRAGUE_EOF => id::PRAGUE_EOF,
            SpecId::LATEST => id::LATEST,
        }
    }
}

pub trait Spec: Sized + 'static {
    /// The specification ID.
    const SPEC_ID: SpecId;

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    fn enabled(spec_id: SpecId) -> bool {
        SpecId::enabled(Self::SPEC_ID, spec_id)
    }
}

macro_rules! spec {
    ($spec_id:ident, $spec_name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $spec_name;

        impl Spec for $spec_name {
            const SPEC_ID: SpecId = $spec_id;
        }
    };
}

spec!(FRONTIER, FrontierSpec);
// FRONTIER_THAWING no EVM spec change
spec!(HOMESTEAD, HomesteadSpec);
// DAO_FORK no EVM spec change
spec!(TANGERINE, TangerineSpec);
spec!(SPURIOUS_DRAGON, SpuriousDragonSpec);
spec!(BYZANTIUM, ByzantiumSpec);
// CONSTANTINOPLE was overridden with PETERSBURG
spec!(PETERSBURG, PetersburgSpec);
spec!(ISTANBUL, IstanbulSpec);
// MUIR_GLACIER no EVM spec change
spec!(BERLIN, BerlinSpec);
spec!(LONDON, LondonSpec);
// ARROW_GLACIER no EVM spec change
// GRAY_GLACIER no EVM spec change
spec!(MERGE, MergeSpec);
spec!(SHANGHAI, ShanghaiSpec);
spec!(CANCUN, CancunSpec);
spec!(PRAGUE, PragueSpec);
spec!(PRAGUE_EOF, PragueEofSpec);

spec!(LATEST, LatestSpec);

#[macro_export]
macro_rules! spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        match $spec_id {
            $crate::hardfork::SpecId::FRONTIER | $crate::hardfork::SpecId::FRONTIER_THAWING => {
                use $crate::hardfork::FrontierSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::HOMESTEAD | $crate::hardfork::SpecId::DAO_FORK => {
                use $crate::hardfork::HomesteadSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::TANGERINE => {
                use $crate::hardfork::TangerineSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::SPURIOUS_DRAGON => {
                use $crate::hardfork::SpuriousDragonSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::BYZANTIUM => {
                use $crate::hardfork::ByzantiumSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::PETERSBURG | $crate::hardfork::SpecId::CONSTANTINOPLE => {
                use $crate::hardfork::PetersburgSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::ISTANBUL | $crate::hardfork::SpecId::MUIR_GLACIER => {
                use $crate::hardfork::IstanbulSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::BERLIN => {
                use $crate::hardfork::BerlinSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::LONDON
            | $crate::hardfork::SpecId::ARROW_GLACIER
            | $crate::hardfork::SpecId::GRAY_GLACIER => {
                use $crate::hardfork::LondonSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::MERGE => {
                use $crate::hardfork::MergeSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::SHANGHAI => {
                use $crate::hardfork::ShanghaiSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::CANCUN => {
                use $crate::hardfork::CancunSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::LATEST => {
                use $crate::hardfork::LatestSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::PRAGUE => {
                use $crate::hardfork::PragueSpec as SPEC;
                $e
            }
            $crate::hardfork::SpecId::PRAGUE_EOF => {
                use $crate::hardfork::PragueEofSpec as SPEC;
                $e
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_to_generic() {
        use SpecId::*;

        spec_to_generic!(FRONTIER, assert_eq!(SPEC::SPEC_ID, FRONTIER));
        spec_to_generic!(FRONTIER_THAWING, assert_eq!(SPEC::SPEC_ID, FRONTIER));
        spec_to_generic!(HOMESTEAD, assert_eq!(SPEC::SPEC_ID, HOMESTEAD));
        spec_to_generic!(DAO_FORK, assert_eq!(SPEC::SPEC_ID, HOMESTEAD));
        spec_to_generic!(TANGERINE, assert_eq!(SPEC::SPEC_ID, TANGERINE));
        spec_to_generic!(SPURIOUS_DRAGON, assert_eq!(SPEC::SPEC_ID, SPURIOUS_DRAGON));
        spec_to_generic!(BYZANTIUM, assert_eq!(SPEC::SPEC_ID, BYZANTIUM));
        spec_to_generic!(CONSTANTINOPLE, assert_eq!(SPEC::SPEC_ID, PETERSBURG));
        spec_to_generic!(PETERSBURG, assert_eq!(SPEC::SPEC_ID, PETERSBURG));
        spec_to_generic!(ISTANBUL, assert_eq!(SPEC::SPEC_ID, ISTANBUL));
        spec_to_generic!(MUIR_GLACIER, assert_eq!(SPEC::SPEC_ID, ISTANBUL));
        spec_to_generic!(BERLIN, assert_eq!(SPEC::SPEC_ID, BERLIN));
        spec_to_generic!(LONDON, assert_eq!(SPEC::SPEC_ID, LONDON));
        spec_to_generic!(ARROW_GLACIER, assert_eq!(SPEC::SPEC_ID, LONDON));
        spec_to_generic!(GRAY_GLACIER, assert_eq!(SPEC::SPEC_ID, LONDON));
        spec_to_generic!(MERGE, assert_eq!(SPEC::SPEC_ID, MERGE));
        spec_to_generic!(CANCUN, assert_eq!(SPEC::SPEC_ID, CANCUN));
        spec_to_generic!(PRAGUE, assert_eq!(SPEC::SPEC_ID, PRAGUE));
        spec_to_generic!(PRAGUE_EOF, assert_eq!(SPEC::SPEC_ID, PRAGUE_EOF));
        spec_to_generic!(LATEST, assert_eq!(SPEC::SPEC_ID, LATEST));
    }
}
