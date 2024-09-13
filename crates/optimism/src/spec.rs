use crate::{
    env::TxEnv, optimism_handle_register, L1BlockInfo, OptimismContext, OptimismHaltReason,
};
use core::marker::PhantomData;
use revm::{
    database_interface::Database,
    handler::register::HandleRegisters,
    precompile::PrecompileSpecId,
    specification::hardfork::{Spec, SpecId},
    wiring::default::block::BlockEnv,
    wiring::EvmWiring,
    EvmHandler,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OptimismEvmWiring<DB: Database, EXT> {
    _phantom: PhantomData<(DB, EXT)>,
}

impl<DB: Database, EXT> EvmWiring for OptimismEvmWiring<DB, EXT> {
    type Block = BlockEnv;
    type Database = DB;
    type ChainContext = Context;
    type ExternalContext = EXT;
    type Hardfork = OptimismSpecId;
    type HaltReason = OptimismHaltReason;
    type Transaction = TxEnv;
}

impl<DB: Database, EXT> revm::EvmWiring for OptimismEvmWiring<DB, EXT> {
    fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
    where
        DB: Database,
    {
        let mut handler = EvmHandler::mainnet_with_spec(hardfork);

        handler.append_handler_register(HandleRegisters::Plain(optimism_handle_register::<Self>));

        handler
    }
}

/// Context for the Optimism chain.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Context {
    l1_block_info: Option<L1BlockInfo>,
}

impl OptimismContext for Context {
    fn l1_block_info(&self) -> Option<&L1BlockInfo> {
        self.l1_block_info.as_ref()
    }

    fn l1_block_info_mut(&mut self) -> &mut Option<L1BlockInfo> {
        &mut self.l1_block_info
    }
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
    FJORD = 22,
    GRANITE = 23,
    PRAGUE = 24,
    PRAGUE_EOF = 25,
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
    const fn into_eth_spec_id(self) -> SpecId {
        match self {
            OptimismSpecId::FRONTIER => SpecId::FRONTIER,
            OptimismSpecId::FRONTIER_THAWING => SpecId::FRONTIER_THAWING,
            OptimismSpecId::HOMESTEAD => SpecId::HOMESTEAD,
            OptimismSpecId::DAO_FORK => SpecId::DAO_FORK,
            OptimismSpecId::TANGERINE => SpecId::TANGERINE,
            OptimismSpecId::SPURIOUS_DRAGON => SpecId::SPURIOUS_DRAGON,
            OptimismSpecId::BYZANTIUM => SpecId::BYZANTIUM,
            OptimismSpecId::CONSTANTINOPLE => SpecId::CONSTANTINOPLE,
            OptimismSpecId::PETERSBURG => SpecId::PETERSBURG,
            OptimismSpecId::ISTANBUL => SpecId::ISTANBUL,
            OptimismSpecId::MUIR_GLACIER => SpecId::MUIR_GLACIER,
            OptimismSpecId::BERLIN => SpecId::BERLIN,
            OptimismSpecId::LONDON => SpecId::LONDON,
            OptimismSpecId::ARROW_GLACIER => SpecId::ARROW_GLACIER,
            OptimismSpecId::GRAY_GLACIER => SpecId::GRAY_GLACIER,
            OptimismSpecId::MERGE | OptimismSpecId::BEDROCK | OptimismSpecId::REGOLITH => {
                SpecId::MERGE
            }
            OptimismSpecId::SHANGHAI | OptimismSpecId::CANYON => SpecId::SHANGHAI,
            OptimismSpecId::CANCUN
            | OptimismSpecId::ECOTONE
            | OptimismSpecId::FJORD
            | OptimismSpecId::GRANITE => SpecId::CANCUN,
            OptimismSpecId::PRAGUE => SpecId::PRAGUE,
            OptimismSpecId::PRAGUE_EOF => SpecId::PRAGUE_EOF,
            OptimismSpecId::LATEST => SpecId::LATEST,
        }
    }
}

impl From<OptimismSpecId> for SpecId {
    fn from(value: OptimismSpecId) -> Self {
        value.into_eth_spec_id()
    }
}

impl From<SpecId> for OptimismSpecId {
    fn from(value: SpecId) -> Self {
        match value {
            SpecId::FRONTIER => Self::FRONTIER,
            SpecId::FRONTIER_THAWING => Self::FRONTIER_THAWING,
            SpecId::HOMESTEAD => Self::HOMESTEAD,
            SpecId::DAO_FORK => Self::DAO_FORK,
            SpecId::TANGERINE => Self::TANGERINE,
            SpecId::SPURIOUS_DRAGON => Self::SPURIOUS_DRAGON,
            SpecId::BYZANTIUM => Self::BYZANTIUM,
            SpecId::CONSTANTINOPLE => Self::CONSTANTINOPLE,
            SpecId::PETERSBURG => Self::PETERSBURG,
            SpecId::ISTANBUL => Self::ISTANBUL,
            SpecId::MUIR_GLACIER => Self::MUIR_GLACIER,
            SpecId::BERLIN => Self::BERLIN,
            SpecId::LONDON => Self::LONDON,
            SpecId::ARROW_GLACIER => Self::ARROW_GLACIER,
            SpecId::GRAY_GLACIER => Self::GRAY_GLACIER,
            SpecId::MERGE => Self::MERGE,
            SpecId::SHANGHAI => Self::SHANGHAI,
            SpecId::CANCUN => Self::CANCUN,
            SpecId::PRAGUE => Self::PRAGUE,
            SpecId::PRAGUE_EOF => Self::PRAGUE_EOF,
            SpecId::LATEST => Self::LATEST,
        }
    }
}

impl From<OptimismSpecId> for PrecompileSpecId {
    fn from(value: OptimismSpecId) -> Self {
        PrecompileSpecId::from_spec_id(value.into_eth_spec_id())
    }
}

/// String identifiers for Optimism hardforks.
pub mod id {
    // Re-export the Ethereum hardforks.
    pub use revm::specification::hardfork::id::*;

    pub const BEDROCK: &str = "Bedrock";
    pub const REGOLITH: &str = "Regolith";
    pub const CANYON: &str = "Canyon";
    pub const ECOTONE: &str = "Ecotone";
    pub const FJORD: &str = "Fjord";
    pub const GRANITE: &str = "Granite";
}

impl From<&str> for OptimismSpecId {
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
            id::BEDROCK => Self::BEDROCK,
            id::REGOLITH => Self::REGOLITH,
            id::CANYON => Self::CANYON,
            id::ECOTONE => Self::ECOTONE,
            id::FJORD => Self::FJORD,
            id::LATEST => Self::LATEST,
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
            | OptimismSpecId::PRAGUE
            | OptimismSpecId::PRAGUE_EOF => value.into_eth_spec_id().into(),
            OptimismSpecId::BEDROCK => id::BEDROCK,
            OptimismSpecId::REGOLITH => id::REGOLITH,
            OptimismSpecId::CANYON => id::CANYON,
            OptimismSpecId::ECOTONE => id::ECOTONE,
            OptimismSpecId::FJORD => id::FJORD,
            OptimismSpecId::GRANITE => id::GRANITE,
            OptimismSpecId::LATEST => id::LATEST,
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
            const SPEC_ID: SpecId = $spec_name::OPTIMISM_SPEC_ID.into_eth_spec_id();
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

// Optimism Hardforks
spec!(BEDROCK, BedrockSpec);
spec!(REGOLITH, RegolithSpec);
spec!(CANYON, CanyonSpec);
spec!(ECOTONE, EcotoneSpec);
spec!(FJORD, FjordSpec);
spec!(GRANITE, GraniteSpec);

#[macro_export]
macro_rules! optimism_spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        // We are transitioning from var to generic spec.
        match $spec_id {
            $crate::OptimismSpecId::FRONTIER | $crate::OptimismSpecId::FRONTIER_THAWING => {
                use $crate::FrontierSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::HOMESTEAD | $crate::OptimismSpecId::DAO_FORK => {
                use $crate::HomesteadSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::TANGERINE => {
                use $crate::TangerineSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::SPURIOUS_DRAGON => {
                use $crate::SpuriousDragonSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::BYZANTIUM => {
                use $crate::ByzantiumSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::PETERSBURG | $crate::OptimismSpecId::CONSTANTINOPLE => {
                use $crate::PetersburgSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::ISTANBUL | $crate::OptimismSpecId::MUIR_GLACIER => {
                use $crate::IstanbulSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::BERLIN => {
                use $crate::BerlinSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::LONDON
            | $crate::OptimismSpecId::ARROW_GLACIER
            | $crate::OptimismSpecId::GRAY_GLACIER => {
                use $crate::LondonSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::MERGE => {
                use $crate::MergeSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::SHANGHAI => {
                use $crate::ShanghaiSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::CANCUN => {
                use $crate::CancunSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::LATEST => {
                use $crate::LatestSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::PRAGUE => {
                use $crate::PragueSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::PRAGUE_EOF => {
                use $crate::PragueEofSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::BEDROCK => {
                use $crate::BedrockSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::REGOLITH => {
                use $crate::RegolithSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::CANYON => {
                use $crate::CanyonSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::GRANITE => {
                use $crate::GraniteSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::ECOTONE => {
                use $crate::EcotoneSpec as SPEC;
                $e
            }
            $crate::OptimismSpecId::FJORD => {
                use $crate::FjordSpec as SPEC;
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
            assert_eq!(SPEC::SPEC_ID, SpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FRONTIER_THAWING,
            assert_eq!(SPEC::SPEC_ID, SpecId::FRONTIER)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::HOMESTEAD,
            assert_eq!(SPEC::SPEC_ID, SpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::DAO_FORK,
            assert_eq!(SPEC::SPEC_ID, SpecId::HOMESTEAD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::TANGERINE,
            assert_eq!(SPEC::SPEC_ID, SpecId::TANGERINE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SPURIOUS_DRAGON,
            assert_eq!(SPEC::SPEC_ID, SpecId::SPURIOUS_DRAGON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BYZANTIUM,
            assert_eq!(SPEC::SPEC_ID, SpecId::BYZANTIUM)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CONSTANTINOPLE,
            assert_eq!(SPEC::SPEC_ID, SpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PETERSBURG,
            assert_eq!(SPEC::SPEC_ID, SpecId::PETERSBURG)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ISTANBUL,
            assert_eq!(SPEC::SPEC_ID, SpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MUIR_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::ISTANBUL)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BERLIN,
            assert_eq!(SPEC::SPEC_ID, SpecId::BERLIN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LONDON,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ARROW_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRAY_GLACIER,
            assert_eq!(SPEC::SPEC_ID, SpecId::LONDON)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::MERGE,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::BEDROCK,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::REGOLITH,
            assert_eq!(SPEC::SPEC_ID, SpecId::MERGE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::SHANGHAI,
            assert_eq!(SPEC::SPEC_ID, SpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANYON,
            assert_eq!(SPEC::SPEC_ID, SpecId::SHANGHAI)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::CANCUN,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::ECOTONE,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::FJORD,
            assert_eq!(SPEC::SPEC_ID, SpecId::CANCUN)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::SPEC_ID, SpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::LATEST,
            assert_eq!(SPEC::SPEC_ID, SpecId::LATEST)
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
            OptimismSpecId::FJORD,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::FJORD)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::GRANITE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::GRANITE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PRAGUE)
        );
        optimism_spec_to_generic!(
            OptimismSpecId::PRAGUE_EOF,
            assert_eq!(SPEC::OPTIMISM_SPEC_ID, OptimismSpecId::PRAGUE_EOF)
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

    #[test]
    fn test_fjord_post_merge_hardforks() {
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::MERGE));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::SHANGHAI));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::CANCUN));
        assert!(!FjordSpec::optimism_enabled(OptimismSpecId::LATEST));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::BEDROCK));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::REGOLITH));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::CANYON));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::ECOTONE));
        assert!(FjordSpec::optimism_enabled(OptimismSpecId::FJORD));
    }

    #[test]
    fn test_fjord_post_merge_hardforks_spec_id() {
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::MERGE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::SHANGHAI
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::CANCUN
        ));
        assert!(!OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::LATEST
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::BEDROCK
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::REGOLITH
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::CANYON
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::ECOTONE
        ));
        assert!(OptimismSpecId::enabled(
            OptimismSpecId::FJORD,
            OptimismSpecId::FJORD
        ));
    }
}
