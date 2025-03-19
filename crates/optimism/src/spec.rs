use core::str::FromStr;
use revm::primitives::hardfork::{name as eth_name, SpecId, UnknownHardfork};

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

impl FromStr for OpSpecId {
    type Err = UnknownHardfork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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
            _ => Err(UnknownHardfork),
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
    use std::vec;

    #[test]
    fn test_op_spec_id_eth_spec_compatibility() {
        // Define test cases: (OpSpecId, enabled in ETH specs, enabled in OP specs)
        let test_cases = [
            (
                OpSpecId::BEDROCK,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, false),
                    (SpecId::CANCUN, false),
                    (SpecId::LATEST, false),
                ],
                vec![(OpSpecId::BEDROCK, true), (OpSpecId::REGOLITH, false)],
            ),
            (
                OpSpecId::REGOLITH,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, false),
                    (SpecId::CANCUN, false),
                    (SpecId::LATEST, false),
                ],
                vec![(OpSpecId::BEDROCK, true), (OpSpecId::REGOLITH, true)],
            ),
            (
                OpSpecId::CANYON,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, true),
                    (SpecId::CANCUN, false),
                    (SpecId::LATEST, false),
                ],
                vec![
                    (OpSpecId::BEDROCK, true),
                    (OpSpecId::REGOLITH, true),
                    (OpSpecId::CANYON, true),
                ],
            ),
            (
                OpSpecId::ECOTONE,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, true),
                    (SpecId::CANCUN, true),
                    (SpecId::LATEST, false),
                ],
                vec![
                    (OpSpecId::BEDROCK, true),
                    (OpSpecId::REGOLITH, true),
                    (OpSpecId::CANYON, true),
                    (OpSpecId::ECOTONE, true),
                ],
            ),
            (
                OpSpecId::FJORD,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, true),
                    (SpecId::CANCUN, true),
                    (SpecId::LATEST, false),
                ],
                vec![
                    (OpSpecId::BEDROCK, true),
                    (OpSpecId::REGOLITH, true),
                    (OpSpecId::CANYON, true),
                    (OpSpecId::ECOTONE, true),
                    (OpSpecId::FJORD, true),
                ],
            ),
        ];

        for (op_spec, eth_tests, op_tests) in test_cases {
            // Test ETH spec compatibility
            for (eth_spec, expected) in eth_tests {
                assert_eq!(
                    op_spec.into_eth_spec().is_enabled_in(eth_spec),
                    expected,
                    "{:?} should {} be enabled in ETH {:?}",
                    op_spec,
                    if expected { "" } else { "not " },
                    eth_spec
                );
            }

            // Test OP spec compatibility
            for (other_op_spec, expected) in op_tests {
                assert_eq!(
                    op_spec.is_enabled_in(other_op_spec),
                    expected,
                    "{:?} should {} be enabled in OP {:?}",
                    op_spec,
                    if expected { "" } else { "not " },
                    other_op_spec
                );
            }
        }
    }
}
