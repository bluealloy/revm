use core::convert::TryFrom;
use num_enum::TryFromPrimitive;
use revm_precompiles::SpecId as PrecompileId;

/// SpecId and their activation block
/// Information was obtained from: https://github.com/ethereum/execution-specs
#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum SpecId {
    FRONTIER = 0,         // Frontier	            0
    FRONTIER_THAWING = 1, // Frontier Thawing       200000
    HOMESTEAD = 2,        // Homestead	            1150000
    DAO_FORK = 3,         // DAO Fork	            1920000
    TANGERINE = 4,        // Tangerine Whistle	    2463000
    SPURIOUS_DRAGON = 5,  // Spurious Dragon        2675000
    BYZANTIUM = 6,        // Byzantium	            4370000
    CONSTANTINOPLE = 7,   // Constantinople         7280000 is overwriten with PETERSBURG
    PETERSBURG = 8,       // Petersburg             7280000
    ISTANBUL = 9,         // Istanbul	            9069000
    MUIR_GLACIER = 10,    // Muir Glacier	        9200000
    BERLIN = 11,          // Berlin	                12244000
    LONDON = 12,          // London	                12965000
    ARROW_GLACIER = 13,   // Arrow Glacier	        13773000
    GRAY_GLACIER = 14,    // Gray Glacier	        15050000
    MERGE = 15,           // Paris/Merge	        TBD (Depends on difficulty)
    LATEST = 16,
}

impl SpecId {
    pub const fn to_precompile_id(self) -> PrecompileId {
        match self {
            FRONTIER | FRONTIER_THAWING | HOMESTEAD | DAO_FORK | TANGERINE | SPURIOUS_DRAGON => {
                PrecompileId::HOMESTEAD
            }
            BYZANTIUM | CONSTANTINOPLE | PETERSBURG => PrecompileId::BYZANTIUM,
            ISTANBUL | MUIR_GLACIER => PrecompileId::ISTANBUL,
            BERLIN | LONDON | ARROW_GLACIER | GRAY_GLACIER | MERGE | LATEST => PrecompileId::BERLIN,
        }
    }

    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::try_from(spec_id).ok()
    }
}

pub use SpecId::*;

impl From<&str> for SpecId {
    fn from(name: &str) -> Self {
        match name {
            "Frontier" => SpecId::FRONTIER,
            "Homestead" => SpecId::HOMESTEAD,
            "Tangerine" => SpecId::TANGERINE,
            "Spurious" => SpecId::SPURIOUS_DRAGON,
            "Byzantium" => SpecId::BYZANTIUM,
            "Constantinople" => SpecId::CONSTANTINOPLE,
            "Petersburg" => SpecId::PETERSBURG,
            "Istanbul" => SpecId::ISTANBUL,
            "MuirGlacier" => SpecId::MUIR_GLACIER,
            "Berlin" => SpecId::BERLIN,
            "London" => SpecId::LONDON,
            "Merge" => SpecId::MERGE,
            _ => SpecId::LATEST,
        }
    }
}

impl SpecId {
    #[inline]
    pub const fn enabled(our: SpecId, other: SpecId) -> bool {
        our as u8 >= other as u8
    }
}

pub(crate) trait NotStaticSpec {}

pub trait Spec: Sized {
    /// little bit of magic. We can have child version of Spec that contains static flag enabled
    type STATIC: Spec;

    #[inline(always)]
    fn enabled(spec_id: SpecId) -> bool {
        Self::SPEC_ID as u8 >= spec_id as u8
    }
    const SPEC_ID: SpecId;
    /// static flag used in STATIC type;
    const IS_STATIC_CALL: bool;

    const ASSUME_PRECOMPILE_HAS_BALANCE: bool;
}

pub(crate) mod spec_impl {
    use super::{NotStaticSpec, Spec};

    macro_rules! spec {
        ($spec_id:tt) => {
            #[allow(non_snake_case)]
            pub mod $spec_id {
                use super::{NotStaticSpec, Spec};
                use crate::SpecId;

                pub struct SpecInner<
                    const STATIC_CALL: bool,
                    const ASSUME_PRECOMPILE_HAS_BALANCE: bool,
                >;

                pub type SpecImpl = SpecInner<false, true>;
                pub type SpecStaticImpl = SpecInner<true, true>;

                impl NotStaticSpec for SpecImpl {}

                impl<const IS_STATIC_CALL: bool, const ASSUME_PRECOMPILE_HAS_BALANCE: bool> Spec
                    for SpecInner<IS_STATIC_CALL, ASSUME_PRECOMPILE_HAS_BALANCE>
                {
                    type STATIC = SpecInner<true, ASSUME_PRECOMPILE_HAS_BALANCE>;

                    //specification id
                    const SPEC_ID: SpecId = SpecId::$spec_id;

                    const IS_STATIC_CALL: bool = IS_STATIC_CALL;

                    const ASSUME_PRECOMPILE_HAS_BALANCE: bool = ASSUME_PRECOMPILE_HAS_BALANCE;
                }
            }
        };
    }

    spec!(FRONTIER);
    // FRONTIER_THAWING no EVM spec change
    spec!(HOMESTEAD);
    // DAO_FORK no EVM spec change
    spec!(TANGERINE);
    spec!(SPURIOUS_DRAGON);
    spec!(BYZANTIUM);
    // CONSTANTINOPLE was overriden with PETERSBURG
    spec!(PETERSBURG);
    spec!(ISTANBUL);
    // MUIR_GLACIER no EVM spec change
    spec!(BERLIN);
    spec!(LONDON);
    // ARROW_GLACIER no EVM spec change
    // GRAT_GLACIER no EVM spec change
    spec!(MERGE);
    spec!(LATEST);
}

pub use spec_impl::BERLIN::SpecImpl as BerlinSpec;
pub use spec_impl::BYZANTIUM::SpecImpl as ByzantiumSpec;
pub use spec_impl::FRONTIER::SpecImpl as FrontierSpec;
pub use spec_impl::HOMESTEAD::SpecImpl as HomesteadSpec;
pub use spec_impl::ISTANBUL::SpecImpl as IstanbulSpec;
pub use spec_impl::LATEST::SpecImpl as LatestSpec;
pub use spec_impl::LONDON::SpecImpl as LondonSpec;
pub use spec_impl::MERGE::SpecImpl as MergeSpec;
pub use spec_impl::PETERSBURG::SpecImpl as PetersburgSpec;
pub use spec_impl::SPURIOUS_DRAGON::SpecImpl as SpuriousDragonSpec;
pub use spec_impl::TANGERINE::SpecImpl as TangerineSpec;
