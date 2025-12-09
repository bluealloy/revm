// Transaction validation with custom validation

use bitflags::bitflags;

bitflags! {
    /// Bitflags for selecting specific validations
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct ValidationChecks: u32 {
        /// Check if transaction has a valid chain id
        const CHAIN_ID = 0b00000001;    
        /// Check of transaction has a valid gas limit
        const TX_GAS_LIMIT = 0b00000010;
        /// Check if the transaction has a valid base fee
        const BASE_FEE = 0b00000100;
        /// Check if the transaction has a valid priority fee
        const PRIORITY_FEE = 0b00001000;
        /// Check if the transaction has a valid blob fee
        const BLOB_FEE = 0b00010000;
        /// Check if the transaction has a valid auth list
        const AUTH_LIST = 0b00100000;
        /// Check if the transaction has a valid block gas limit
        const BLOCK_GAS_LIMIT = 0b01000000;
        /// Check if the transaction has a valid initcode size
        const MAX_INITCODE_SIZE = 0b10000000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValidationKind {
    /// No validation
    None,
    /// Validate by transaction type
    #[default]
    ByTxType,
    /// Validate by custom checks
    Custom(ValidationChecks),
}
