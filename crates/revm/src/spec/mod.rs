mod spec;

use core::convert::TryFrom;
use num_enum::TryFromPrimitive;
use revm_precompiles::SpecId as PrecompileId;
pub use spec::*;

#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive)]
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
    pub fn enabled(self, current_id: u8) -> bool {
        self as u8 > current_id
    }
}
