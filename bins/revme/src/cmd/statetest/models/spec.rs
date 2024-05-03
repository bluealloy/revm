use revm::primitives::EthSpecId;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Hash)]
pub enum SpecName {
    Frontier,
    FrontierToHomesteadAt5,
    Homestead,
    HomesteadToDaoAt5,
    HomesteadToEIP150At5,
    EIP150,
    EIP158, // EIP-161: State trie clearing
    EIP158ToByzantiumAt5,
    Byzantium,
    ByzantiumToConstantinopleAt5, // SKIPPED
    ByzantiumToConstantinopleFixAt5,
    Constantinople, // SKIPPED
    ConstantinopleFix,
    Istanbul,
    Berlin,
    BerlinToLondonAt5,
    London,
    Merge,
    Shanghai,
    Cancun,
    #[serde(other)]
    Unknown,
}

impl SpecName {
    pub fn to_spec_id(&self) -> EthSpecId {
        match self {
            Self::Frontier => EthSpecId::FRONTIER,
            Self::Homestead | Self::FrontierToHomesteadAt5 => EthSpecId::HOMESTEAD,
            Self::EIP150 | Self::HomesteadToDaoAt5 | Self::HomesteadToEIP150At5 => {
                EthSpecId::TANGERINE
            }
            Self::EIP158 => EthSpecId::SPURIOUS_DRAGON,
            Self::Byzantium | Self::EIP158ToByzantiumAt5 => EthSpecId::BYZANTIUM,
            Self::ConstantinopleFix | Self::ByzantiumToConstantinopleFixAt5 => {
                EthSpecId::PETERSBURG
            }
            Self::Istanbul => EthSpecId::ISTANBUL,
            Self::Berlin => EthSpecId::BERLIN,
            Self::London | Self::BerlinToLondonAt5 => EthSpecId::LONDON,
            Self::Merge => EthSpecId::MERGE,
            Self::Shanghai => EthSpecId::SHANGHAI,
            Self::Cancun => EthSpecId::CANCUN,
            Self::ByzantiumToConstantinopleAt5 | Self::Constantinople => {
                panic!("Overridden with PETERSBURG")
            }
            Self::Unknown => panic!("Unknown spec"),
        }
    }
}
