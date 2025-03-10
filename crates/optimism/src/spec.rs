use revm::primitives::hardfork::{name as eth_name, SpecId};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum OpSpecId {
    BEDROCK = 100,
    REGOLITH,
    CANYON,
    ECOTONE,
    FJORD,
    GRANITE,
    HOLOCENE,
    #[default]
    ISTHMUS,
    INTEROP,
    OSAKA,
}

impl OpSpecId {
    /// Converts the [`OpSpecId`] into a [`SpecId`].
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            Self::BEDROCK | Self::REGOLITH => SpecId::MERGE,
            Self::CANYON => SpecId::SHANGHAI,
            Self::ECOTONE | Self::FJORD | Self::GRANITE | Self::HOLOCENE => SpecId::CANCUN,
            Self::ISTHMUS | Self::INTEROP => SpecId::PRAGUE,
            Self::OSAKA => SpecId::OSAKA,
        }
    }

    pub const fn is_enabled_in(self, other: OpSpecId) -> bool {
        other as u8 <= self as u8
    }
}

impl From<OpSpecId> for SpecId {
    fn from(spec: OpSpecId) -> Self {
        spec.into_eth_spec()
    }
}

impl TryFrom<&str> for OpSpecId {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match name {
            name::BEDROCK => Ok(OpSpecId::BEDROCK),
            name::REGOLITH => Ok(OpSpecId::REGOLITH),
            name::CANYON => Ok(OpSpecId::CANYON),
            name::ECOTONE => Ok(OpSpecId::ECOTONE),
            name::FJORD => Ok(OpSpecId::FJORD),
            name::GRANITE => Ok(OpSpecId::GRANITE),
            name::HOLOCENE => Ok(OpSpecId::HOLOCENE),
            name::ISTHMUS => Ok(OpSpecId::ISTHMUS),
            name::INTEROP => Ok(OpSpecId::INTEROP),
            eth_name::OSAKA => Ok(OpSpecId::OSAKA),
            _ => Err(()),
        }
    }
}

impl From<OpSpecId> for &'static str {
    fn from(spec_id: OpSpecId) -> Self {
        match spec_id {
            OpSpecId::BEDROCK => name::BEDROCK,
            OpSpecId::REGOLITH => name::REGOLITH,
            OpSpecId::CANYON => name::CANYON,
            OpSpecId::ECOTONE => name::ECOTONE,
            OpSpecId::FJORD => name::FJORD,
            OpSpecId::GRANITE => name::GRANITE,
            OpSpecId::HOLOCENE => name::HOLOCENE,
            OpSpecId::ISTHMUS => name::ISTHMUS,
            OpSpecId::INTEROP => name::INTEROP,
            OpSpecId::OSAKA => eth_name::OSAKA,
        }
    }
}

/// String identifiers for Optimism hardforks
pub mod name {
    pub const BEDROCK: &str = "Bedrock";
    pub const REGOLITH: &str = "Regolith";
    pub const CANYON: &str = "Canyon";
    pub const ECOTONE: &str = "Ecotone";
    pub const FJORD: &str = "Fjord";
    pub const GRANITE: &str = "Granite";
    pub const HOLOCENE: &str = "Holocene";
    pub const ISTHMUS: &str = "Isthmus";
    pub const INTEROP: &str = "Interop";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bedrock_post_merge_hardforks() {
        assert!(OpSpecId::BEDROCK
            .into_eth_spec()
            .is_enabled_in(SpecId::MERGE));
        assert!(!OpSpecId::BEDROCK
            .into_eth_spec()
            .is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpecId::BEDROCK
            .into_eth_spec()
            .is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpecId::BEDROCK
            .into_eth_spec()
            .is_enabled_in(SpecId::LATEST));
        assert!(OpSpecId::BEDROCK.is_enabled_in(OpSpecId::BEDROCK));
        assert!(!OpSpecId::BEDROCK.is_enabled_in(OpSpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks() {
        assert!(OpSpecId::REGOLITH
            .into_eth_spec()
            .is_enabled_in(SpecId::MERGE));
        assert!(!OpSpecId::REGOLITH
            .into_eth_spec()
            .is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpecId::REGOLITH
            .into_eth_spec()
            .is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpecId::REGOLITH
            .into_eth_spec()
            .is_enabled_in(SpecId::LATEST));
        assert!(OpSpecId::REGOLITH.is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpecId::REGOLITH.is_enabled_in(OpSpecId::REGOLITH));
    }

    #[test]
    fn test_canyon_post_merge_hardforks() {
        assert!(OpSpecId::CANYON
            .into_eth_spec()
            .is_enabled_in(SpecId::MERGE));
        assert!(OpSpecId::CANYON
            .into_eth_spec()
            .is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpecId::CANYON
            .into_eth_spec()
            .is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpecId::CANYON
            .into_eth_spec()
            .is_enabled_in(SpecId::LATEST));
        assert!(OpSpecId::CANYON.is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpecId::CANYON.is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpecId::CANYON.is_enabled_in(OpSpecId::CANYON));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks() {
        assert!(OpSpecId::ECOTONE
            .into_eth_spec()
            .is_enabled_in(SpecId::MERGE));
        assert!(OpSpecId::ECOTONE
            .into_eth_spec()
            .is_enabled_in(SpecId::SHANGHAI));
        assert!(OpSpecId::ECOTONE
            .into_eth_spec()
            .is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpecId::ECOTONE
            .into_eth_spec()
            .is_enabled_in(SpecId::LATEST));
        assert!(OpSpecId::ECOTONE.is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpecId::ECOTONE.is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpecId::ECOTONE.is_enabled_in(OpSpecId::CANYON));
        assert!(OpSpecId::ECOTONE.is_enabled_in(OpSpecId::ECOTONE));
    }

    #[test]
    fn test_fjord_post_merge_hardforks() {
        assert!(OpSpecId::FJORD.into_eth_spec().is_enabled_in(SpecId::MERGE));
        assert!(OpSpecId::FJORD
            .into_eth_spec()
            .is_enabled_in(SpecId::SHANGHAI));
        assert!(OpSpecId::FJORD
            .into_eth_spec()
            .is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpecId::FJORD
            .into_eth_spec()
            .is_enabled_in(SpecId::LATEST));
        assert!(OpSpecId::FJORD.is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpecId::FJORD.is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpecId::FJORD.is_enabled_in(OpSpecId::CANYON));
        assert!(OpSpecId::FJORD.is_enabled_in(OpSpecId::ECOTONE));
        assert!(OpSpecId::FJORD.is_enabled_in(OpSpecId::FJORD));
    }
}
