use core::convert::TryFrom;
use num_enum::TryFromPrimitive;
use revm_precompiles::SpecId as PrecompileId;

#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum SpecId {
    FRONTIER = 1,
    HOMESTEAD = 2,
    TANGERINE = 3,
    SPURIOUS_DRAGON = 4,
    BYZANTINE = 5,
    CONSTANTINOPLE = 6,
    PETERSBURG = 7,
    ISTANBUL = 8,
    MUIRGLACIER = 9,
    BERLIN = 10,
    LONDON = 11,
    LATEST = 12,
}

impl SpecId {
    pub const fn to_precompile_id(self) -> u8 {
        match self {
            FRONTIER | HOMESTEAD | TANGERINE | SPURIOUS_DRAGON => PrecompileId::HOMESTEAD as u8,
            BYZANTINE | CONSTANTINOPLE | PETERSBURG => PrecompileId::BYZANTINE as u8,
            ISTANBUL | MUIRGLACIER => PrecompileId::ISTANBUL as u8,
            BERLIN | LONDON | LATEST => PrecompileId::BERLIN as u8,
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
            "Byzantium" => SpecId::BYZANTINE,
            "Constantinople" => SpecId::CONSTANTINOPLE,
            "Petersburg" => SpecId::PETERSBURG,
            "Istanbul" => SpecId::ISTANBUL,
            "MuirGlacier" => SpecId::MUIRGLACIER,
            "Berlin" => SpecId::BERLIN,
            "London" => SpecId::LONDON,
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
    /// litle bit of magic. We can have child version of Spec that contains static flag enabled
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

mod spec_impl {
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

    spec!(LATEST);
    spec!(LONDON);
    spec!(BERLIN);
    spec!(ISTANBUL);
    spec!(BYZANTINE);
    spec!(FRONTIER);
}

pub use spec_impl::{
    BERLIN::SpecImpl as BerlinSpec, BYZANTINE::SpecImpl as ByzantineSpec,
    FRONTIER::SpecImpl as FrontierSpec, ISTANBUL::SpecImpl as IstanbulSpec,
    LATEST::SpecImpl as LatestSpec, LONDON::SpecImpl as LondonSpec,
};
