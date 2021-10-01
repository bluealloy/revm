
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