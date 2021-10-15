mod berlin_spec;
mod instanbul_spec;
mod latest_spec;
mod frontier_spec;
mod spec;

pub use berlin_spec::BerlinSpec;
pub use instanbul_spec::IstanbulSpec;
pub use latest_spec::LatestSpec;
pub use frontier_spec::FrontierSpec;
pub use spec::Spec;

pub(crate) use spec::NotStaticSpec;


#[repr(u8)]
pub enum SpecId {
    HOMESTEAD = 1,
    DAO = 2,
    TANGERINE = 3,
    SPURIOUS = 4,
    FRONTIER = 5,
    BYZANTINE = 6,
    CONSTANTINOPLE = 7,
    PETERSBURG = 8,
    ISTANBUL = 9,
    MUIRGLACIER =10,
    BERLIN = 11,
    LONDON = 12,
    LATEST = 13,
}

pub use SpecId::*;

impl From<&str> for SpecId {
    fn from(name: &str) -> Self {
        match name {
            "Homestead" => SpecId::HOMESTEAD,
            "Dao" => SpecId::DAO,
            "Tangerine" => SpecId::TANGERINE,
            "Spurious" => SpecId::SPURIOUS,
            "Frontier" => SpecId::FRONTIER,
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
    pub fn enabled(self,current_id: u8) -> bool {
        self as u8 > current_id
    } 
}