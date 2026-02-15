//! Configuration for the EVM. Containing [`SpecId`].

pub mod gas;
pub mod gas_params;

pub use gas_params::{GasId, GasParams};

use auto_impl::auto_impl;
use core::{fmt::Debug, hash::Hash};
use primitives::{hardfork::SpecId, Address, TxKind, ValidationChecks, U256};

/// Configuration for the EVM.
#[auto_impl(&, &mut, Box, Arc)]
pub trait Cfg {
    /// Specification id type, it requires to be convertible to `SpecId` so it can be used
    /// by default in mainnet.
    type Spec: Into<SpecId> + Clone;

    /// Returns the chain ID of the EVM that is compared with the transaction's chain ID.
    fn chain_id(&self) -> u64;

    /// Returns whether the transaction's chain ID check is enabled.
    fn tx_chain_id_check(&self) -> bool;

    /// Returns the gas limit cap for the transaction.
    ///
    /// Cap is introduced in [`EIP-7825: Transaction Gas Limit Cap`](https://eips.ethereum.org/EIPS/eip-7825)
    /// with initial cap of 30M gas.
    ///
    /// Value before EIP-7825 is `u64::MAX`.
    fn tx_gas_limit_cap(&self) -> u64;

    /// Specification id
    fn spec(&self) -> Self::Spec;

    /// Returns the maximum number of blobs allowed per transaction.
    /// If it is None, check for max count will be skipped.
    fn max_blobs_per_tx(&self) -> Option<u64>;

    /// Returns the maximum code size for the given spec id.
    fn max_code_size(&self) -> usize;

    /// Returns the max initcode size for the given spec id.
    fn max_initcode_size(&self) -> usize;

    /// Returns whether the EIP-3607 (account clearing) is disabled.
    fn is_eip3607_disabled(&self) -> bool;

    /// Returns whether the EIP-3541 (disallowing new contracts with 0xEF prefix) is disabled.
    fn is_eip3541_disabled(&self) -> bool;

    /// Returns whether the EIP-7623 (increased calldata cost) is disabled.
    fn is_eip7623_disabled(&self) -> bool;

    /// Returns whether the balance check is disabled.
    fn is_balance_check_disabled(&self) -> bool;

    /// Returns whether the block gas limit check is disabled.
    fn is_block_gas_limit_disabled(&self) -> bool;

    /// Returns whether the nonce check is disabled.
    fn is_nonce_check_disabled(&self) -> bool;

    /// Returns whether the base fee check is disabled.
    fn is_base_fee_check_disabled(&self) -> bool;

    /// Returns whether the priority fee check is disabled.
    fn is_priority_fee_check_disabled(&self) -> bool;

    /// Returns whether the fee charge is disabled.
    fn is_fee_charge_disabled(&self) -> bool;

    /// Returns whether EIP-7708 (ETH transfers emit logs) is disabled.
    fn is_eip7708_disabled(&self) -> bool;

    /// Returns whether EIP-7708 delayed burn logging is disabled.
    ///
    /// When enabled, revm tracks all self-destructed addresses and emits logs for
    /// accounts that still have remaining balance at the end of the transaction.
    /// This can be disabled for performance reasons as it requires storing and
    /// iterating over all self-destructed accounts. When disabled, the logging
    /// can be done outside of revm when applying accounts to database state.
    fn is_eip7708_delayed_burn_disabled(&self) -> bool;

    /// Returns the limit in bytes for the memory buffer.
    fn memory_limit(&self) -> u64;

    /// Returns the gas params for the EVM.
    fn gas_params(&self) -> &GasParams;

    /// Returns the validation checks that are disabled.
    ///
    /// This method aggregates the individual `is_*_disabled()` methods into a single
    /// [`ValidationChecks`] bitflag value, enabling efficient validation configuration.
    ///
    /// # Covered Checks
    ///
    /// The following checks can be disabled via their corresponding methods:
    /// - [`CHAIN_ID`](ValidationChecks::CHAIN_ID) - via [`tx_chain_id_check()`](Self::tx_chain_id_check) (inverted)
    /// - [`BASE_FEE`](ValidationChecks::BASE_FEE) - via [`is_base_fee_check_disabled()`](Self::is_base_fee_check_disabled)
    /// - [`PRIORITY_FEE`](ValidationChecks::PRIORITY_FEE) - via [`is_priority_fee_check_disabled()`](Self::is_priority_fee_check_disabled)
    /// - [`BLOCK_GAS_LIMIT`](ValidationChecks::BLOCK_GAS_LIMIT) - via [`is_block_gas_limit_disabled()`](Self::is_block_gas_limit_disabled)
    /// - [`NONCE`](ValidationChecks::NONCE) - via [`is_nonce_check_disabled()`](Self::is_nonce_check_disabled)
    /// - [`BALANCE`](ValidationChecks::BALANCE) - via [`is_balance_check_disabled()`](Self::is_balance_check_disabled)
    /// - [`EIP3607`](ValidationChecks::EIP3607) - via [`is_eip3607_disabled()`](Self::is_eip3607_disabled)
    /// - [`EIP7623`](ValidationChecks::EIP7623) - via [`is_eip7623_disabled()`](Self::is_eip7623_disabled)
    ///
    /// # Not Covered
    ///
    /// The following checks are always enabled (cannot be disabled via this trait):
    /// `TX_GAS_LIMIT`, `BLOB_FEE`, `AUTH_LIST`, `MAX_INITCODE_SIZE`, `HEADER`
    ///
    /// # Performance
    ///
    /// The default implementation calls 8 individual methods. Implementations can
    /// override this to return a pre-computed value for better performance.
    #[inline]
    fn disabled_validation_checks(&self) -> ValidationChecks {
        let mut disabled = ValidationChecks::empty();
        if !self.tx_chain_id_check() {
            disabled |= ValidationChecks::CHAIN_ID;
        }
        if self.is_base_fee_check_disabled() {
            disabled |= ValidationChecks::BASE_FEE;
        }
        if self.is_priority_fee_check_disabled() {
            disabled |= ValidationChecks::PRIORITY_FEE;
        }
        if self.is_block_gas_limit_disabled() {
            disabled |= ValidationChecks::BLOCK_GAS_LIMIT;
        }
        if self.is_nonce_check_disabled() {
            disabled |= ValidationChecks::NONCE;
        }
        if self.is_balance_check_disabled() {
            disabled |= ValidationChecks::BALANCE;
        }
        if self.is_eip3607_disabled() {
            disabled |= ValidationChecks::EIP3607;
        }
        if self.is_eip7623_disabled() {
            disabled |= ValidationChecks::EIP7623;
        }
        disabled
    }

    /// Returns the validation checks that are enabled.
    ///
    /// This is the inverse of [`disabled_validation_checks()`](Self::disabled_validation_checks),
    /// returning `ALL - disabled`.
    #[inline]
    fn enabled_validation_checks(&self) -> ValidationChecks {
        ValidationChecks::ALL - self.disabled_validation_checks()
    }
}

/// What bytecode analysis to perform
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    /// Do not perform bytecode analysis
    Raw,
    /// Perform bytecode analysis
    #[default]
    Analyse,
}

/// Transaction destination
pub type TransactTo = TxKind;

/// Create scheme
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`
    #[default]
    Create,
    /// Create scheme of `CREATE2`
    Create2 {
        /// Salt
        salt: U256,
    },
    /// Custom scheme where we set up the original address
    Custom {
        /// Custom contract creation address.
        address: Address,
    },
}
