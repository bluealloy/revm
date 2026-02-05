//! Transaction validation checks configuration.
//!
//! This module provides [`ValidationChecks`] bitflags for configuring which
//! transaction validation checks should be performed.

use bitflags::bitflags;

bitflags! {
    /// Bitflags for configurable transaction validation checks.
    ///
    /// Each flag represents a specific validation check that can be enabled or disabled.
    /// Combine flags using bitwise OR to create custom validation configurations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ValidationChecks: u16 {
        /// Validate chain ID matches (EIP-155).
        const CHAIN_ID = 1 << 0;
        /// Validate transaction gas limit against cap (EIP-7825).
        const TX_GAS_LIMIT = 1 << 1;
        /// Validate gas price against base fee.
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
        /// Validate EIP-3607 (reject senders with deployed code).
        const EIP3607 = 1 << 10;
        /// Validate EIP-7623 floor gas.
        const EIP7623 = 1 << 11;
        /// Validate block header fields (prevrandao, excess_blob_gas).
        const HEADER = 1 << 12;

        /// All gas and fee related checks.
        const GAS_FEES = Self::TX_GAS_LIMIT.bits()
            | Self::BASE_FEE.bits()
            | Self::PRIORITY_FEE.bits()
            | Self::BLOB_FEE.bits()
            | Self::BLOCK_GAS_LIMIT.bits()
            | Self::EIP7623.bits();

        /// All stateless transaction checks (no account state needed).
        const TX_STATELESS = Self::CHAIN_ID.bits()
            | Self::GAS_FEES.bits()
            | Self::AUTH_LIST.bits()
            | Self::MAX_INITCODE_SIZE.bits()
            | Self::HEADER.bits();

        /// All caller/state checks.
        const CALLER = Self::NONCE.bits() | Self::BALANCE.bits() | Self::EIP3607.bits();

        /// All validation checks enabled.
        const ALL = Self::TX_STATELESS.bits() | Self::CALLER.bits();
    }
}
