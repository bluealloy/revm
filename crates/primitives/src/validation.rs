//! Transaction validation checks configuration.
//!
//! This module provides [`ValidationChecks`] bitflags for configuring which
//! transaction validation checks should be performed.
//!
//! # Example
//!
//! ```
//! use revm_primitives::ValidationChecks;
//!
//! // Default enables all checks
//! let checks = ValidationChecks::default();
//! assert_eq!(checks, ValidationChecks::ALL);
//!
//! // Disable specific checks
//! let custom = ValidationChecks::ALL - ValidationChecks::NONCE - ValidationChecks::BALANCE;
//! assert!(!custom.contains(ValidationChecks::NONCE));
//!
//! // Start with no checks and enable specific ones
//! let minimal = ValidationChecks::CHAIN_ID | ValidationChecks::NONCE;
//! ```

use bitflags::bitflags;

bitflags! {
    /// Bitflags for configurable transaction validation checks.
    ///
    /// Each flag represents a specific validation check that can be enabled or disabled.
    /// Combine flags using bitwise OR to create custom validation configurations.
    ///
    /// # Composite Flags
    ///
    /// Several composite flags are provided for common use cases:
    /// - [`GAS_FEES`](Self::GAS_FEES): All gas and fee related checks
    /// - [`TX_STATELESS`](Self::TX_STATELESS): Checks that don't require account state
    /// - [`CALLER`](Self::CALLER): Checks that require caller account state
    /// - [`ALL`](Self::ALL): All validation checks (default)
    ///
    /// # Default
    ///
    /// The default value is [`ALL`](Self::ALL), enabling all validation checks.
    /// Use [`ValidationChecks::empty()`] to start with no checks enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ValidationChecks: u16 {
        /// Validate chain ID matches (EIP-155).
        const CHAIN_ID = 1 << 0;
        /// Validate transaction gas limit against cap (EIP-7825).
        const TX_GAS_LIMIT = 1 << 1;
        /// Validate gas price against base fee (EIP-1559).
        const BASE_FEE = 1 << 2;
        /// Validate priority fee for EIP-1559+ transactions.
        const PRIORITY_FEE = 1 << 3;
        /// Validate blob fee for EIP-4844 transactions.
        const BLOB_FEE = 1 << 4;
        /// Validate authorization list for EIP-7702 transactions.
        const AUTH_LIST = 1 << 5;
        /// Validate transaction gas limit against block gas limit.
        const BLOCK_GAS_LIMIT = 1 << 6;
        /// Validate initcode size for contract creation (EIP-3860).
        const MAX_INITCODE_SIZE = 1 << 7;
        /// Validate nonce matches account state.
        const NONCE = 1 << 8;
        /// Validate caller balance for transaction cost.
        const BALANCE = 1 << 9;
        /// Reject transactions from senders with deployed code (EIP-3607).
        const EIP3607 = 1 << 10;
        /// Validate floor gas for calldata (EIP-7623).
        const EIP7623 = 1 << 11;
        /// Validate block header fields (prevrandao, excess_blob_gas).
        const HEADER = 1 << 12;

        /// All gas and fee related checks.
        ///
        /// Includes: [`TX_GAS_LIMIT`](Self::TX_GAS_LIMIT), [`BASE_FEE`](Self::BASE_FEE),
        /// [`PRIORITY_FEE`](Self::PRIORITY_FEE), [`BLOB_FEE`](Self::BLOB_FEE),
        /// [`BLOCK_GAS_LIMIT`](Self::BLOCK_GAS_LIMIT), [`EIP7623`](Self::EIP7623).
        const GAS_FEES = Self::TX_GAS_LIMIT.bits()
            | Self::BASE_FEE.bits()
            | Self::PRIORITY_FEE.bits()
            | Self::BLOB_FEE.bits()
            | Self::BLOCK_GAS_LIMIT.bits()
            | Self::EIP7623.bits();

        /// All stateless transaction checks (no account state needed).
        ///
        /// Includes: [`CHAIN_ID`](Self::CHAIN_ID), [`GAS_FEES`](Self::GAS_FEES),
        /// [`AUTH_LIST`](Self::AUTH_LIST), [`MAX_INITCODE_SIZE`](Self::MAX_INITCODE_SIZE),
        /// [`HEADER`](Self::HEADER).
        const TX_STATELESS = Self::CHAIN_ID.bits()
            | Self::GAS_FEES.bits()
            | Self::AUTH_LIST.bits()
            | Self::MAX_INITCODE_SIZE.bits()
            | Self::HEADER.bits();

        /// All caller/state checks (require account state).
        ///
        /// Includes: [`NONCE`](Self::NONCE), [`BALANCE`](Self::BALANCE),
        /// [`EIP3607`](Self::EIP3607).
        const CALLER = Self::NONCE.bits() | Self::BALANCE.bits() | Self::EIP3607.bits();

        /// All validation checks enabled.
        ///
        /// This is the default value. Equivalent to `TX_STATELESS | CALLER`.
        const ALL = Self::TX_STATELESS.bits() | Self::CALLER.bits();
    }
}

impl Default for ValidationChecks {
    /// Returns [`ValidationChecks::ALL`] - all checks enabled by default.
    #[inline]
    fn default() -> Self {
        Self::ALL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_all() {
        assert_eq!(ValidationChecks::default(), ValidationChecks::ALL);
    }

    #[test]
    fn test_all_equals_stateless_plus_caller() {
        assert_eq!(
            ValidationChecks::ALL,
            ValidationChecks::TX_STATELESS | ValidationChecks::CALLER
        );
    }

    #[test]
    fn test_tx_stateless_composite() {
        let checks = ValidationChecks::TX_STATELESS;
        assert!(checks.contains(ValidationChecks::CHAIN_ID));
        assert!(checks.contains(ValidationChecks::GAS_FEES));
        assert!(checks.contains(ValidationChecks::AUTH_LIST));
        assert!(checks.contains(ValidationChecks::MAX_INITCODE_SIZE));
        assert!(checks.contains(ValidationChecks::HEADER));
    }

    #[test]
    fn test_caller_composite() {
        let checks = ValidationChecks::CALLER;
        assert!(checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
        assert!(checks.contains(ValidationChecks::EIP3607));
    }

    #[test]
    fn test_gas_fees_composite() {
        let checks = ValidationChecks::GAS_FEES;
        assert!(checks.contains(ValidationChecks::TX_GAS_LIMIT));
        assert!(checks.contains(ValidationChecks::BASE_FEE));
        assert!(checks.contains(ValidationChecks::PRIORITY_FEE));
        assert!(checks.contains(ValidationChecks::BLOB_FEE));
        assert!(checks.contains(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(checks.contains(ValidationChecks::EIP7623));
    }

    #[test]
    fn test_subtract_checks() {
        let checks = ValidationChecks::ALL - ValidationChecks::NONCE;
        assert!(!checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
    }
}
