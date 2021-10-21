mod berlin_spec;
mod byzantium_spec;
mod frontier_spec;
mod instanbul_spec;
mod latest_spec;
mod spec;

pub use berlin_spec::BerlinSpec;
pub use byzantium_spec::ByzantiumSpec;
pub use frontier_spec::FrontierSpec;
pub use instanbul_spec::IstanbulSpec;
pub use latest_spec::LatestSpec;
pub use spec::Spec;

pub(crate) use spec::NotStaticSpec;

#[repr(u8)]
#[derive(Debug)]
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
