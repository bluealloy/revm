use revm_precompile::PrecompileSpecId;

use crate::{
    chain_spec::ChainSpec,
    primitives::{EthSpecId, Spec},
};

use super::OptimismHaltReason;

pub struct OptimismChainSpec;

impl ChainSpec for OptimismChainSpec {
    type Hardfork = OptimismSpecId;
    type HaltReason = OptimismHaltReason;
}

/// Specification IDs for the optimism blockchain.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum OptimismSpecId {
    FRONTIER = 0,
    FRONTIER_THAWING = 1,
    HOMESTEAD = 2,
    DAO_FORK = 3,
    TANGERINE = 4,
    SPURIOUS_DRAGON = 5,
    BYZANTIUM = 6,
    CONSTANTINOPLE = 7,
    PETERSBURG = 8,
    ISTANBUL = 9,
    MUIR_GLACIER = 10,
    BERLIN = 11,
    LONDON = 12,
    ARROW_GLACIER = 13,
    GRAY_GLACIER = 14,
    MERGE = 15,
    BEDROCK = 16,
    REGOLITH = 17,
    SHANGHAI = 18,
    CANYON = 19,
    CANCUN = 20,
    ECOTONE = 21,
    PRAGUE = 22,
    #[default]
    LATEST = u8::MAX,
}

impl OptimismSpecId {
    /// Returns the `OptimismSpecId` for the given `u8`.
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
    pub const fn enabled(our: Self, other: Self) -> bool {
        our as u8 >= other as u8
    }

    /// Converts the `OptimismSpecId` into a `SpecId`.
    const fn into_eth_spec_id(self) -> EthSpecId {
        match self {
            OptimismSpecId::FRONTIER => EthSpecId::FRONTIER,
            OptimismSpecId::FRONTIER_THAWING => EthSpecId::FRONTIER_THAWING,
            OptimismSpecId::HOMESTEAD => EthSpecId::HOMESTEAD,
            OptimismSpecId::DAO_FORK => EthSpecId::DAO_FORK,
            OptimismSpecId::TANGERINE => EthSpecId::TANGERINE,
            OptimismSpecId::SPURIOUS_DRAGON => EthSpecId::SPURIOUS_DRAGON,
            OptimismSpecId::BYZANTIUM => EthSpecId::BYZANTIUM,
            OptimismSpecId::CONSTANTINOPLE => EthSpecId::CONSTANTINOPLE,
            OptimismSpecId::PETERSBURG => EthSpecId::PETERSBURG,
            OptimismSpecId::ISTANBUL => EthSpecId::ISTANBUL,
            OptimismSpecId::MUIR_GLACIER => EthSpecId::MUIR_GLACIER,
            OptimismSpecId::BERLIN => EthSpecId::BERLIN,
            OptimismSpecId::LONDON => EthSpecId::LONDON,
            OptimismSpecId::ARROW_GLACIER => EthSpecId::ARROW_GLACIER,
            OptimismSpecId::GRAY_GLACIER => EthSpecId::GRAY_GLACIER,
            OptimismSpecId::MERGE | OptimismSpecId::BEDROCK | OptimismSpecId::REGOLITH => {
                EthSpecId::MERGE
            }
            OptimismSpecId::SHANGHAI | OptimismSpecId::CANYON => EthSpecId::SHANGHAI,
            OptimismSpecId::CANCUN | OptimismSpecId::ECOTONE => EthSpecId::CANCUN,
            OptimismSpecId::PRAGUE => EthSpecId::PRAGUE,
            OptimismSpecId::LATEST => EthSpecId::LATEST,
        }
    }
}

impl From<OptimismSpecId> for EthSpecId {
    fn from(value: OptimismSpecId) -> Self {
        value.into_eth_spec_id()
    }
}

impl From<EthSpecId> for OptimismSpecId {
    fn from(value: EthSpecId) -> Self {
        match value {
            EthSpecId::FRONTIER => Self::FRONTIER,
            EthSpecId::FRONTIER_THAWING => Self::FRONTIER_THAWING,
            EthSpecId::HOMESTEAD => Self::HOMESTEAD,
            EthSpecId::DAO_FORK => Self::DAO_FORK,
            EthSpecId::TANGERINE => Self::TANGERINE,
            EthSpecId::SPURIOUS_DRAGON => Self::SPURIOUS_DRAGON,
            EthSpecId::BYZANTIUM => Self::BYZANTIUM,
            EthSpecId::CONSTANTINOPLE => Self::CONSTANTINOPLE,
            EthSpecId::PETERSBURG => Self::PETERSBURG,
            EthSpecId::ISTANBUL => Self::ISTANBUL,
            EthSpecId::MUIR_GLACIER => Self::MUIR_GLACIER,
            EthSpecId::BERLIN => Self::BERLIN,
            EthSpecId::LONDON => Self::LONDON,
            EthSpecId::ARROW_GLACIER => Self::ARROW_GLACIER,
            EthSpecId::GRAY_GLACIER => Self::GRAY_GLACIER,
            EthSpecId::MERGE => Self::MERGE,
            EthSpecId::SHANGHAI => Self::SHANGHAI,
            EthSpecId::CANCUN => Self::CANCUN,
            EthSpecId::PRAGUE => Self::PRAGUE,
            EthSpecId::LATEST => Self::LATEST,
        }
    }
}

impl From<OptimismSpecId> for PrecompileSpecId {
    fn from(value: OptimismSpecId) -> Self {
        PrecompileSpecId::from_spec_id(value.into_eth_spec_id())
    }
}

impl From<&str> for OptimismSpecId {
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
            "Prague" => Self::PRAGUE,
            "Bedrock" => Self::BEDROCK,
            "Regolith" => Self::REGOLITH,
            "Canyon" => Self::CANYON,
            "Ecotone" => Self::ECOTONE,
            _ => Self::LATEST,
        }
    }
}

impl From<OptimismSpecId> for &'static str {
    fn from(value: OptimismSpecId) -> Self {
        match value {
            OptimismSpecId::FRONTIER
            | OptimismSpecId::FRONTIER_THAWING
            | OptimismSpecId::HOMESTEAD
            | OptimismSpecId::DAO_FORK
            | OptimismSpecId::TANGERINE
            | OptimismSpecId::SPURIOUS_DRAGON
            | OptimismSpecId::BYZANTIUM
            | OptimismSpecId::CONSTANTINOPLE
            | OptimismSpecId::PETERSBURG
            | OptimismSpecId::ISTANBUL
            | OptimismSpecId::MUIR_GLACIER
            | OptimismSpecId::BERLIN
            | OptimismSpecId::LONDON
            | OptimismSpecId::ARROW_GLACIER
            | OptimismSpecId::GRAY_GLACIER
            | OptimismSpecId::MERGE
            | OptimismSpecId::SHANGHAI
            | OptimismSpecId::CANCUN
            | OptimismSpecId::PRAGUE => value.into_eth_spec_id().into(),
            OptimismSpecId::BEDROCK => "Bedrock",
            OptimismSpecId::REGOLITH => "Regolith",
            OptimismSpecId::CANYON => "Canyon",
            OptimismSpecId::ECOTONE => "Ecotone",
            OptimismSpecId::LATEST => "Latest",
        }
    }
}

pub trait OptimismSpec: Spec + Sized + 'static {
    /// The specification ID for optimism.
    const OPTIMISM_SPEC_ID: OptimismSpecId;

    /// Returns whether the provided `OptimismSpec` is enabled by this spec.
    #[inline]
    fn optimism_enabled(spec_id: OptimismSpecId) -> bool {
        OptimismSpecId::enabled(Self::OPTIMISM_SPEC_ID, spec_id)
    }
}

macro_rules! spec {
    ($spec_id:ident, $spec_name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $spec_name;

        impl OptimismSpec for $spec_name {
            const OPTIMISM_SPEC_ID: OptimismSpecId = OptimismSpecId::$spec_id;
        }

        impl Spec for $spec_name {
            const SPEC_ID: EthSpecId = $spec_name::OPTIMISM_SPEC_ID.into_eth_spec_id();
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

spec!(LATEST, LatestSpec);

// Optimism Hardforks
spec!(BEDROCK, BedrockSpec);
spec!(REGOLITH, RegolithSpec);
spec!(CANYON, CanyonSpec);
spec!(ECOTONE, EcotoneSpec);

#[macro_export]
macro_rules! optimism_spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        // We are transitioning from var to generic spec.
        match $spec_id {
            $crate::optimism::OptimismSpecId::FRONTIER
            | $crate::optimism::OptimismSpecId::FRONTIER_THAWING => {
                use $crate::optimism::FrontierSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::HOMESTEAD
            | $crate::optimism::OptimismSpecId::DAO_FORK => {
                use $crate::optimism::HomesteadSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::TANGERINE => {
                use $crate::optimism::TangerineSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::SPURIOUS_DRAGON => {
                use $crate::optimism::SpuriousDragonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BYZANTIUM => {
                use $crate::optimism::ByzantiumSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::PETERSBURG
            | $crate::optimism::OptimismSpecId::CONSTANTINOPLE => {
                use $crate::optimism::PetersburgSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::ISTANBUL
            | $crate::optimism::OptimismSpecId::MUIR_GLACIER => {
                use $crate::optimism::IstanbulSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BERLIN => {
                use $crate::optimism::BerlinSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::LONDON
            | $crate::optimism::OptimismSpecId::ARROW_GLACIER
            | $crate::optimism::OptimismSpecId::GRAY_GLACIER => {
                use $crate::optimism::LondonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::MERGE => {
                use $crate::optimism::MergeSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::SHANGHAI => {
                use $crate::optimism::ShanghaiSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::CANCUN => {
                use $crate::optimism::CancunSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::LATEST => {
                use $crate::optimism::LatestSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::PRAGUE => {
                use $crate::optimism::PragueSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::BEDROCK => {
                use $crate::optimism::BedrockSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::REGOLITH => {
                use $crate::optimism::RegolithSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::CANYON => {
                use $crate::optimism::CanyonSpec as SPEC;
                $e
            }
            $crate::optimism::OptimismSpecId::ECOTONE => {
                use $crate::optimism::EcotoneSpec as SPEC;
                $e
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimism_spec_to_generic() {
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER_THAWING,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::HOMESTEAD,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::DAO_FORK,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::TANGERINE,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::TANGERINE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SPURIOUS_DRAGON,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::SPURIOUS_DRAGON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BYZANTIUM,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::BYZANTIUM)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CONSTANTINOPLE,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PETERSBURG,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ISTANBUL,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MUIR_GLACIER,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BERLIN,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::BERLIN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LONDON,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ARROW_GLACIER,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRAY_GLACIER,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MERGE,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BEDROCK,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::REGOLITH,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SHANGHAI,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANYON,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANCUN,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ECOTONE,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LATEST,
            assert_eq!(SPEC::SPEC_ID, EthSpecId::LATEST)
        );

        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER_THAWING,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::HOMESTEAD,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::DAO_FORK,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::TANGERINE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::TANGERINE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SPURIOUS_DRAGON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::SPURIOUS_DRAGON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BYZANTIUM,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BYZANTIUM)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CONSTANTINOPLE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PETERSBURG,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ISTANBUL,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MUIR_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BERLIN,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BERLIN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LONDON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ARROW_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRAY_GLACIER,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MERGE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BEDROCK,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::BEDROCK)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::REGOLITH,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::REGOLITH)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SHANGHAI,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANYON,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::CANYON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANCUN,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ECOTONE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::ECOTONE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LATEST,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::LATEST)
        );
    }

    #[test]
    fn test_bedrock_post_merge_hardforks() {
        assert!(BedrockSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(BedrockSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(!BedrockSpec::optimism_enabled(OptimismSpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks() {
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!RegolithSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(RegolithSpec::optimism_enabled(OptimismSpecId::REGOLITH));
    }

    #[test]
    fn test_bedrock_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::MERGE
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::BEDROCK
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::BEDROCK,
            OptimismSpecId::REGOLITH
        ));
    }

    #[test]
    fn test_regolith_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::MERGE
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::REGOLITH,
            OptimismSpecId::REGOLITH
        ));
    }

    #[test]
    fn test_canyon_post_merge_hardforks() {
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(!CanyonSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!CanyonSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(CanyonSpec::optimism_enabled(OptimismSpecId::CANYON));
    }

    #[test]
    fn test_canyon_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::SHANGHAI
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::CANYON,
            OptimismSpecId::CANYON
        ));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks() {
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!EcotoneSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::CANYON));
        assert!(EcotoneSpec::optimism_enabled(OptimismSpecId::ECOTONE));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::SHANGHAI
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::CANYON
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::ECOTONE,
            OptimismSpecId::ECOTONE
        ));
    }
}
