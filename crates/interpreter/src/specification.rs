/// SpecId and their activation block
/// Information was obtained from: https://github.com/ethereum/execution-specs
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, enumn::N)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
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
    MERGE = 15,           // Paris/Merge	        TBD (Depends on difficulty)
    MERGE_EOF = 16,       // Merge+EOF	            TBD
    LATEST = 17,
}

impl SpecId {
    pub fn try_from_u8(spec_id: u8) -> Option<Self> {
        Self::n(spec_id)
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
            "MergeEOF" => SpecId::MERGE_EOF,
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

pub trait Spec: Sized {
    #[inline(always)]
    fn enabled(spec_id: SpecId) -> bool {
        Self::SPEC_ID as u8 >= spec_id as u8
    }
    const SPEC_ID: SpecId;
}

macro_rules! spec {
    ($spec_id:tt,$spec_name:tt) => {
        pub struct $spec_name;

        impl Spec for $spec_name {
            //specification id
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
// CONSTANTINOPLE was overriden with PETERSBURG
spec!(PETERSBURG, PetersburgSpec);
spec!(ISTANBUL, IstanbulSpec);
// MUIR_GLACIER no EVM spec change
spec!(BERLIN, BerlinSpec);
spec!(LONDON, LondonSpec);
// ARROW_GLACIER no EVM spec change
// GRAY_GLACIER no EVM spec change
spec!(MERGE, MergeSpec);
// MERGE_EOF is pending EVM change
spec!(LATEST, LatestSpec);
