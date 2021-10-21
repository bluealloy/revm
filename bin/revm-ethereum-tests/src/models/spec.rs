use revm::SpecId;
use serde_derive::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Deserialize)]
pub enum SpecName {
    EIP150,
    EIP158,
    Frontier,
    Homestead,
    Byzantium,
    Constantinople,
    ConstantinopleFix,
    Istanbul,
    EIP158ToByzantiumAt5,
    FrontierToHomesteadAt5,
    HomesteadToDaoAt5,
    HomesteadToEIP150At5,
    ByzantiumToConstantinopleAt5,
    ByzantiumToConstantinopleFixAt5,
    Berlin,
    London,
    BerlinToLondonAt5,
}

impl SpecName {
    pub fn to_spec_id(&self) -> SpecId {
        match self {
            Self::Berlin => SpecId::BERLIN,
            Self::Istanbul => SpecId::ISTANBUL,
            _ => panic!("Conversion failed"),
        }
    }
}
