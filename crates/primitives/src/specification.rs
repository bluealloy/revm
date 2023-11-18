#![allow(non_camel_case_types)]

pub use SpecId::*;

/// Specification IDs and their activation block.
///
/// Information was obtained from: <https://github.com/ethereum/execution-specs>
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpecId {
    FRONTIER = 0,         // Frontier	            0
    FRONTIER_THAWING = 1, // Frontier Thawing       200000
    HOMESTEAD = 2,        // Homestead	            1150000
    DAO_FORK = 3,         // DAO Fork	            1920000
    TANGERINE = 4,        // Tangerine Whistle	    2463000
    SPURIOUS_DRAGON = 5,  // Spurious Dragon        2675000
    BYZANTIUM = 6,        // Byzantium	            4370000
    CONSTANTINOPLE = 7,   // Constantinople         7280000 is overwritten with PETERSBURG
    PETERSBURG = 8,       // Petersburg             7280000
    ISTANBUL = 9,         // Istanbul	            9069000
    MUIR_GLACIER = 10,    // Muir Glacier	        9200000
    BERLIN = 11,          // Berlin	                12244000
    LONDON = 12,          // London	                12965000
    ARROW_GLACIER = 13,   // Arrow Glacier	        13773000
    GRAY_GLACIER = 14,    // Gray Glacier	        15050000
    MERGE = 15,           // Paris/Merge	        15537394 (TTD: 58750000000000000000000)
    SHANGHAI = 16,        // Shanghai	            17034870 (TS: 1681338455)
    CANCUN = 17,          // Cancun	                TBD
    #[cfg(feature = "optimism")]
    BEDROCK = 128,
    #[cfg(feature = "optimism")]
    REGOLITH = 129,
    LATEST = u8::MAX,
}

impl SpecId {
    #[inline]
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
    }

    #[inline]
    pub const fn enabled(our: SpecId, other: SpecId) -> bool {
        #[cfg(feature = "optimism")]
        {
            let (our, other) = (our as u8, other as u8);
            let (merge, bedrock, regolith) =
                (Self::MERGE as u8, Self::BEDROCK as u8, Self::REGOLITH as u8);
            // If the Spec is Bedrock or Regolith, and the input is not Bedrock or Regolith,
            // then no hardforks should be enabled after the merge. This is because Optimism's
            // Bedrock and Regolith hardforks implement changes on top of the Merge hardfork.
            let is_self_optimism = our == bedrock || our == regolith;
            let input_not_optimism = other != bedrock && other != regolith;
            let after_merge = other > merge;

            if is_self_optimism && input_not_optimism && after_merge {
                return false;
            }
        }

        our as u8 >= other as u8
    }
}

impl From<&str> for SpecId {
    fn from(name: &str) -> Self {
        match name {
            "Frontier" => Self::FRONTIER,
            "Homestead" => Self::HOMESTEAD,
            "Tangerine" => Self::TANGERINE,
            "Spurious" => Self::SPURIOUS_DRAGON,
            "Byzantium" => Self::BYZANTIUM,
            "Constantinople" => Self::CONSTANTINOPLE,
            "Petersburg" => Self::PETERSBURG,
            "Istanbul" => Self::ISTANBUL,
            "MuirGlacier" => Self::MUIR_GLACIER,
            "Berlin" => Self::BERLIN,
            "London" => Self::LONDON,
            "Merge" => Self::MERGE,
            "Shanghai" => Self::SHANGHAI,
            "Cancun" => Self::CANCUN,
            #[cfg(feature = "optimism")]
            "Bedrock" => SpecId::BEDROCK,
            #[cfg(feature = "optimism")]
            "Regolith" => SpecId::REGOLITH,
            _ => Self::LATEST,
        }
    }
}

pub trait Spec: Sized {
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

spec!(LATEST, LatestSpec);

// Optimism Hardforks
#[cfg(feature = "optimism")]
spec!(BEDROCK, BedrockSpec);
#[cfg(feature = "optimism")]
spec!(REGOLITH, RegolithSpec);

#[cfg(feature = "optimism")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bedrock_post_merge_hardforks() {
        assert!(BedrockSpec::enabled(SpecId::MERGE));
        assert!(!BedrockSpec::enabled(SpecId::SHANGHAI));
        assert!(!BedrockSpec::enabled(SpecId::CANCUN));
        assert!(!BedrockSpec::enabled(SpecId::LATEST));
        assert!(BedrockSpec::enabled(SpecId::BEDROCK));
        assert!(!BedrockSpec::enabled(SpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks() {
        assert!(RegolithSpec::enabled(SpecId::MERGE));
        assert!(!RegolithSpec::enabled(SpecId::SHANGHAI));
        assert!(!RegolithSpec::enabled(SpecId::CANCUN));
        assert!(!RegolithSpec::enabled(SpecId::LATEST));
        assert!(RegolithSpec::enabled(SpecId::BEDROCK));
        assert!(RegolithSpec::enabled(SpecId::REGOLITH));
    }

    #[test]
    fn test_bedrock_post_merge_hardforks_spec_id() {
        assert!(SpecId::enabled(SpecId::BEDROCK, SpecId::MERGE));
        assert!(!SpecId::enabled(SpecId::BEDROCK, SpecId::SHANGHAI));
        assert!(!SpecId::enabled(SpecId::BEDROCK, SpecId::CANCUN));
        assert!(!SpecId::enabled(SpecId::BEDROCK, SpecId::LATEST));
        assert!(SpecId::enabled(SpecId::BEDROCK, SpecId::BEDROCK));
        assert!(!SpecId::enabled(SpecId::BEDROCK, SpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks_spec_id() {
        assert!(SpecId::enabled(SpecId::REGOLITH, SpecId::MERGE));
        assert!(!SpecId::enabled(SpecId::REGOLITH, SpecId::SHANGHAI));
        assert!(!SpecId::enabled(SpecId::REGOLITH, SpecId::CANCUN));
        assert!(!SpecId::enabled(SpecId::REGOLITH, SpecId::LATEST));
        assert!(SpecId::enabled(SpecId::REGOLITH, SpecId::BEDROCK));
        assert!(SpecId::enabled(SpecId::REGOLITH, SpecId::REGOLITH));
    }
}
