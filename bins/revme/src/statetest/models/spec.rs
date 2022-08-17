use revm::SpecId;
use serde_derive::*;

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
    Byzantium,                    // done
    ByzantiumToConstantinopleAt5, // SKIPPED
    ByzantiumToConstantinopleFixAt5,
    Constantinople, // SKIPPED
    ConstantinopleFix,
    Istanbul,
    Berlin,            //done
    BerlinToLondonAt5, // done
    London,            // done
    Merge,             //done
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
            Self::ByzantiumToConstantinopleAt5 | Self::Constantinople => panic!("Overriden with PETERSBURG"),
            //_ => panic!("Conversion failed"),
        }
    }
}

/*
    spec!(FRONTIER);
    // FRONTIER_THAWING no EVM spec change
    spec!(HOMESTEAD);
    // DAO_FORK no EVM spec change
    spec!(TANGERINE);
    spec!(SPURIOUS_DRAGON);
    spec!(BYZANTIUM);
    // CONSTANTINOPLE was overriden with PETERSBURG
    spec!(PETERSBURG);
    spec!(ISTANBUL);
    // MUIR_GLACIER no EVM spec change
    spec!(BERLIN);
    spec!(LONDON);
    // ARROW_GLACIER no EVM spec change
    // GRAT_GLACIER no EVM spec change
    spec!(MERGE);
    spec!(LATEST);
*/
