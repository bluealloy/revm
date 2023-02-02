use revm::primitives::SpecId;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Deserialize)]
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
    #[serde(alias = "Merge+3540+3670")]
    MergeEOF,
    #[serde(alias = "Merge+3860")]
    MergeMeterInitCode,
    #[serde(alias = "Merge+3855")]
    MergePush0,
    #[serde(other)]
    Unknown,
}

impl SpecName {
    pub fn to_spec_id(&self) -> SpecId {
        match self {
            Self::Frontier => SpecId::FRONTIER,
            Self::Homestead | Self::FrontierToHomesteadAt5 => SpecId::HOMESTEAD,
            Self::EIP150 | Self::HomesteadToDaoAt5 | Self::HomesteadToEIP150At5 => {
                SpecId::TANGERINE
            }
            Self::EIP158 => SpecId::SPURIOUS_DRAGON,
            Self::Byzantium | Self::EIP158ToByzantiumAt5 => SpecId::BYZANTIUM,
            Self::ConstantinopleFix | Self::ByzantiumToConstantinopleFixAt5 => SpecId::PETERSBURG,
            Self::Istanbul => SpecId::ISTANBUL,
            Self::Berlin => SpecId::BERLIN,
            Self::London | Self::BerlinToLondonAt5 => SpecId::LONDON,
            Self::Merge => SpecId::MERGE,
            Self::MergeEOF => SpecId::MERGE_EOF,
            Self::MergeMeterInitCode => SpecId::MERGE_EOF,
            Self::MergePush0 => SpecId::MERGE_EOF,
            Self::ByzantiumToConstantinopleAt5 | Self::Constantinople => {
                panic!("Overriden with PETERSBURG")
            }
            Self::Unknown => panic!("Unknown spec"),
        }
    }
}
