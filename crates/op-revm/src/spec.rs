//! Optimism-specific hardfork specifications.
//!
//! This module defines the hardfork versions (specifications) for Optimism:
//!
//! - **Bedrock** (March 2023): Initial version of the new modular architecture for OP Stack.
//!   Mapped to Ethereum's Merge (Paris) specification.
//!
//! - **Regolith** (June 2023): Improved gas accounting for deposit transactions and removed
//!   special handling for "system transactions." Mapped to Ethereum's Merge specification.
//!
//! - **Canyon** (January 2024): Added support for Shanghai EIPs including EIP-4895 (withdrawals).
//!   Mapped to Ethereum's Shanghai specification.
//!
//! - **Ecotone** (May 2024): Introduced Ethereum Cancun features including EIP-4844 (blobs).
//!   Mapped to Ethereum's Cancun specification.
//!
//! - **Fjord** (Planned): Extended Cancun with additional features like P256 signature verification.
//!   Mapped to Ethereum's Cancun specification.
//!
//! - **Granite** (Planned): Further extension of Cancun features.
//!   Mapped to Ethereum's Cancun specification.
//!
//! - **Holocene** (Planned): Final Cancun-based hardfork.
//!   Mapped to Ethereum's Cancun specification.
//!
//! - **Isthmus** (Planned): First Prague-based hardfork for Optimism.
//!   Mapped to Ethereum's Prague specification.
//!
//! - **Interop** (Test): Used for testing Prague-based hardforks.
//!   Mapped to Ethereum's Prague specification.
//!
//! - **Osaka** (Planned): Update with EVM Object Format (EOF) support.
//!   Mapped to Ethereum's Osaka specification.
//!
//! Each Optimism hardfork maps to an Ethereum specification and may include
//! Optimism-specific features and modifications.

use core::str::FromStr;
use revm::primitives::hardfork::{name as eth_name, SpecId, UnknownHardfork};

/// Specification IDs for Optimism hardforks.
///
/// Each hardfork represents a distinct set of protocol changes and maps to an
/// Ethereum specification (SpecId). The mapping allows Optimism to leverage
/// Ethereum's protocol changes while adding rollup-specific features.
///
/// The value of each variant is used for hardfork ordering and activation checks
/// via the `is_enabled_in` method.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum OpSpecId {
    /// Bedrock: Initial OP Stack modular architecture (March 2023)
    BEDROCK = 100,
    /// Regolith: Improved gas accounting for deposits (June 2023)
    REGOLITH,
    /// Canyon: Added Shanghai EIPs including withdrawals (January 2024)
    CANYON,
    /// Ecotone: Added Cancun features including EIP-4844 blobs (May 2024)
    ECOTONE,
    /// Fjord: Extends Cancun with additional features like P256 verify (Planned)
    FJORD,
    /// Granite: Further extension of Cancun features (Planned)
    GRANITE,
    /// Holocene: Final Cancun-based hardfork (Planned)
    HOLOCENE,
    /// Isthmus: First Prague-based hardfork for Optimism (Planned)
    #[default]
    ISTHMUS,
    /// Interop: Used for testing Prague-based hardforks
    INTEROP,
    /// Osaka: Update with EVM Object Format support (Planned)
    OSAKA,
}

impl OpSpecId {
    /// Converts the [`OpSpecId`] into a [`SpecId`].
    ///
    /// This mapping allows Optimism hardforks to inherit Ethereum protocol rules
    /// from the corresponding Ethereum hardfork while adding Optimism-specific behavior:
    ///
    /// - Bedrock/Regolith → Merge (Paris)
    /// - Canyon → Shanghai
    /// - Ecotone/Fjord/Granite/Holocene → Cancun
    /// - Isthmus/Interop → Prague
    /// - Osaka → Osaka
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            Self::BEDROCK | Self::REGOLITH => SpecId::MERGE,
            Self::CANYON => SpecId::SHANGHAI,
            Self::ECOTONE | Self::FJORD | Self::GRANITE | Self::HOLOCENE => SpecId::CANCUN,
            Self::ISTHMUS | Self::INTEROP => SpecId::PRAGUE,
            Self::OSAKA => SpecId::OSAKA,
        }
    }

    /// Checks if a given hardfork is enabled in the current hardfork.
    ///
    /// Returns `true` if the hardfork specified by `other` is active in
    /// the current hardfork. For example:
    ///
    /// ```ignore
    /// let is_regolith_enabled = current_spec.is_enabled_in(OpSpecId::REGOLITH);
    /// ```
    ///
    /// This is used to conditionally enable hardfork-specific behavior.
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

/// String identifiers for Optimism hardforks.
///
/// This module provides constant string representations for each Optimism hardfork,
/// which are used for parsing hardfork names from configuration and for displaying
/// hardfork information in logs and interfaces.
pub mod name {
    /// String identifier for the Bedrock hardfork
    pub const BEDROCK: &str = "Bedrock";
    /// String identifier for the Regolith hardfork
    pub const REGOLITH: &str = "Regolith";
    /// String identifier for the Canyon hardfork
    pub const CANYON: &str = "Canyon";
    /// String identifier for the Ecotone hardfork
    pub const ECOTONE: &str = "Ecotone";
    /// String identifier for the Fjord hardfork
    pub const FJORD: &str = "Fjord";
    /// String identifier for the Granite hardfork
    pub const GRANITE: &str = "Granite";
    /// String identifier for the Holocene hardfork
    pub const HOLOCENE: &str = "Holocene";
    /// String identifier for the Isthmus hardfork
    pub const ISTHMUS: &str = "Isthmus";
    /// String identifier for the Interop test hardfork
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
                    (SpecId::default(), false),
                ],
                vec![(OpSpecId::BEDROCK, true), (OpSpecId::REGOLITH, false)],
            ),
            (
                OpSpecId::REGOLITH,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, false),
                    (SpecId::CANCUN, false),
                    (SpecId::default(), false),
                ],
                vec![(OpSpecId::BEDROCK, true), (OpSpecId::REGOLITH, true)],
            ),
            (
                OpSpecId::CANYON,
                vec![
                    (SpecId::MERGE, true),
                    (SpecId::SHANGHAI, true),
                    (SpecId::CANCUN, false),
                    (SpecId::default(), false),
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
                    (SpecId::default(), false),
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
                    (SpecId::default(), false),
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
