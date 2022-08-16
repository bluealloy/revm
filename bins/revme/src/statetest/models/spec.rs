use revm::SpecId;
use serde_derive::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Deserialize)]
pub enum SpecName {
    EIP150,
    EIP158,
    Frontier,
    Homestead,
    Byzantium, // done
    Constantinople,
    ConstantinopleFix,
    Istanbul,
    EIP158ToByzantiumAt5,
    FrontierToHomesteadAt5,
    HomesteadToDaoAt5,
    HomesteadToEIP150At5,
    ByzantiumToConstantinopleAt5,
    ByzantiumToConstantinopleFixAt5,
    Berlin, //done
    London, // done
    BerlinToLondonAt5, // done
    Merge, //done
}

impl SpecName {
    pub fn to_spec_id(&self) -> SpecId {
        match self {
            Self::Merge => SpecId::MERGE,
            Self::BerlinToLondonAt5 => SpecId::LONDON,
            Self::London => SpecId::LONDON,
            Self::Berlin => SpecId::BERLIN,
            Self::Istanbul => SpecId::ISTANBUL,
            Self::ConstantinopleFix => SpecId::PETERSBURG,
            Self::Byzantium => SpecId::BYZANTIUM,
            _ => panic!("Conversion failed"),
        }
    }
}
