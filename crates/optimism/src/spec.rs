use revm::specification::hardfork::SpecId;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpSpec {
    Eth(SpecId),
    Op(OpSpecId),
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum OpSpecId {
    BEDROCK = 100,
    REGOLITH,
    CANYON,
    ECOTONE,
    FJORD,
    GRANITE,
}

impl OpSpecId {
    /// Converts the [`OpSpec`] into a [`SpecId`].
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            Self::BEDROCK | Self::REGOLITH => SpecId::MERGE,
            Self::CANYON => SpecId::SHANGHAI,
            Self::ECOTONE | Self::FJORD | Self::GRANITE => SpecId::CANCUN,
        }
    }

    pub const fn is_enabled_in(self, other: OpSpecId) -> bool {
        self as u8 <= other as u8
    }
}

impl From<OpSpecId> for OpSpec {
    fn from(spec: OpSpecId) -> Self {
        OpSpec::Op(spec)
    }
}

impl From<SpecId> for OpSpec {
    fn from(spec: SpecId) -> Self {
        OpSpec::Eth(spec)
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
}

impl OpSpec {
    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub fn is_enabled_in(self, other: impl Into<Self>) -> bool {
        match (self, other.into()) {
            (OpSpec::Eth(this), OpSpec::Eth(other)) => other as u8 <= this as u8,
            (OpSpec::Op(this), OpSpec::Op(other)) => other as u8 <= this as u8,
            (OpSpec::Eth(this), OpSpec::Op(other)) => other.into_eth_spec() as u8 <= this as u8,
            (OpSpec::Op(this), OpSpec::Eth(other)) => other as u8 <= this.into_eth_spec() as u8,
        }
    }

    /// Converts the [`OpSpec`] into a [`SpecId`].
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            OpSpec::Eth(spec) => spec,
            OpSpec::Op(spec) => spec.into_eth_spec(),
        }
    }
}

impl From<&str> for OpSpec {
    fn from(name: &str) -> Self {
        let eth = SpecId::from(name);
        if eth != SpecId::LATEST {
            return Self::Eth(eth);
        }
        match OpSpecId::try_from(name) {
            Ok(op) => Self::Op(op),
            Err(_) => Self::Eth(SpecId::LATEST),
        }
    }
}

impl From<OpSpec> for &'static str {
    fn from(value: OpSpec) -> Self {
        match value {
            OpSpec::Eth(eth) => eth.into(),
            OpSpec::Op(op) => op.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bedrock_post_merge_hardforks() {
        assert!(OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(SpecId::MERGE));
        assert!(!OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(SpecId::LATEST));
        assert!(OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(OpSpecId::BEDROCK));
        assert!(!OpSpec::Op(OpSpecId::BEDROCK).is_enabled_in(OpSpecId::REGOLITH));
    }

    #[test]
    fn test_regolith_post_merge_hardforks() {
        assert!(OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(SpecId::MERGE));
        assert!(!OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(SpecId::LATEST));
        assert!(OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpec::Op(OpSpecId::REGOLITH).is_enabled_in(OpSpecId::REGOLITH));
    }

    #[test]
    fn test_canyon_post_merge_hardforks() {
        assert!(OpSpec::Op(OpSpecId::CANYON).is_enabled_in(SpecId::MERGE));
        assert!(OpSpec::Op(OpSpecId::CANYON).is_enabled_in(SpecId::SHANGHAI));
        assert!(!OpSpec::Op(OpSpecId::CANYON).is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpec::Op(OpSpecId::CANYON).is_enabled_in(SpecId::LATEST));
        assert!(OpSpec::Op(OpSpecId::CANYON).is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpec::Op(OpSpecId::CANYON).is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpec::Op(OpSpecId::CANYON).is_enabled_in(OpSpecId::CANYON));
    }

    #[test]
    fn test_ecotone_post_merge_hardforks() {
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(SpecId::MERGE));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(SpecId::SHANGHAI));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(SpecId::LATEST));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(OpSpecId::CANYON));
        assert!(OpSpec::Op(OpSpecId::ECOTONE).is_enabled_in(OpSpecId::ECOTONE));
    }

    #[test]
    fn test_fjord_post_merge_hardforks() {
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(SpecId::MERGE));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(SpecId::SHANGHAI));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(SpecId::CANCUN));
        assert!(!OpSpec::Op(OpSpecId::FJORD).is_enabled_in(SpecId::LATEST));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(OpSpecId::BEDROCK));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(OpSpecId::REGOLITH));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(OpSpecId::CANYON));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(OpSpecId::ECOTONE));
        assert!(OpSpec::Op(OpSpecId::FJORD).is_enabled_in(OpSpecId::FJORD));
    }
}
