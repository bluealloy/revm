//! Transaction validation logic extracted from Handler.
//!
//! This module provides configurable transaction validation through the [`TxValidator`] struct.
//! It is designed to be reusable outside of the full EVM execution context, such as in
//! transaction pools or block builders.
//!
//! # Quick Start
//!
//! ```ignore
//! use revm_handler::TxValidator;
//!
//! // Create validator from EVM context (holds references, zero-cost initialization)
//! let validator = TxValidator::new(&cfg, &block);
//!
//! // Validate transaction (stateless)
//! validator.validate_tx(&tx)?;
//! let gas = validator.initial_gas(&tx)?;
//!
//! // Validate against account state
//! validator.validate_caller(&caller_info, tx.nonce())?;
//! let fee = validator.caller_fee(caller_balance, &tx)?;
//! ```
//!
//! # Customization
//!
//! ```ignore
//! // Skip all validation (system/deposit transactions)
//! let validator = TxValidator::new(&cfg, &block).skip_all();
//!
//! // Skip specific checks
//! let validator = TxValidator::new(&cfg, &block)
//!     .skip_nonce_check()
//!     .skip_balance_check();
//!
//! // Enable only specific checks
//! let validator = TxValidator::new(&cfg, &block)
//!     .skip_all()
//!     .enable_chain_id_check()
//!     .enable_gas_fee_checks();
//! ```

use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, Cfg,
};
use core::cmp::{self, Ordering};
use interpreter::{instructions::calculate_initial_tx_gas_for_tx, InitialAndFloorGas};
use primitives::{eip4844, hardfork::SpecId, ValidationChecks, B256, U256};
use state::AccountInfo;

/// Transaction validator that works with Cfg and Block references.
///
/// This struct provides a fluent API for validating Ethereum transactions.
/// It holds references to the configuration and block, avoiding data copying
/// and providing efficient validation.
///
/// The validator can be configured to skip certain checks (e.g., for L2 deposit
/// transactions) or to validate only specific aspects of a transaction.
///
/// # Type Parameters
///
/// - `C`: Configuration type implementing [`Cfg`]. Can be a reference (`&CfgEnv`),
///   owned value, `Arc<CfgEnv>`, etc.
/// - `B`: Block type implementing [`Block`]. Can be a reference (`&BlockEnv`),
///   owned value, `Arc<BlockEnv>`, etc.
///
/// # Example
///
/// ```ignore
/// // Standard validation with references (recommended)
/// let validator = TxValidator::new(&cfg, &block);
/// validator.validate_tx(&tx)?;
///
/// // Optimism deposit - skip fee checks
/// let validator = TxValidator::new(&cfg, &block)
///     .skip_base_fee_check()
///     .skip_priority_fee_check()
///     .skip_balance_check();
/// ```
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct TxValidator<C: Cfg, B: Block> {
    /// Configuration reference.
    pub cfg: C,
    /// Block reference.
    pub block: B,
    /// Override for validation checks.
    /// If None, uses `cfg.enabled_validation_checks()`.
    checks_override: Option<ValidationChecks>,
}

impl<C: Cfg, B: Block> TxValidator<C, B> {
    /// Create a new validator from Cfg and Block.
    ///
    /// This is the primary constructor. Pass references for zero-cost initialization:
    /// ```ignore
    /// let validator = TxValidator::new(&cfg, &block);
    /// ```
    #[inline]
    pub fn new(cfg: C, block: B) -> Self {
        Self {
            cfg,
            block,
            checks_override: None,
        }
    }

    /// Create validator for L2 deposit transactions.
    ///
    /// Deposit transactions (like Optimism's system deposits) skip most validation:
    /// - No fee checks (base fee, priority fee, blob fee)
    /// - No balance check (system-funded)
    /// - No nonce check (system-managed)
    /// - Keeps: chain ID, block gas limit, initcode size, auth list
    #[inline]
    pub fn for_deposit(cfg: C, block: B) -> Self {
        Self {
            cfg,
            block,
            checks_override: Some(
                ValidationChecks::CHAIN_ID
                    | ValidationChecks::BLOCK_GAS_LIMIT
                    | ValidationChecks::AUTH_LIST
                    | ValidationChecks::MAX_INITCODE_SIZE,
            ),
        }
    }

    /// Create validator for transaction pool validation.
    ///
    /// Transaction pools validate incoming transactions before inclusion:
    /// - All fee checks enabled (ensure tx can pay)
    /// - No block gas limit check (pool doesn't know target block)
    /// - No header validation (no block context yet)
    /// - No nonce/balance/EIP-3607 checks (state may change before inclusion)
    #[inline]
    pub fn for_tx_pool(cfg: C, block: B) -> Self {
        Self {
            cfg,
            block,
            checks_override: Some(
                ValidationChecks::TX_STATELESS
                    - ValidationChecks::BLOCK_GAS_LIMIT
                    - ValidationChecks::HEADER,
            ),
        }
    }

    /// Create validator for block builders.
    ///
    /// Block builders have more flexibility:
    /// - Skip nonce check (can reorder transactions)
    /// - Skip balance check (can ensure profitability differently)
    /// - Keep all stateless transaction checks
    #[inline]
    pub fn for_block_builder(cfg: C, block: B) -> Self {
        Self {
            cfg,
            block,
            checks_override: Some(ValidationChecks::TX_STATELESS),
        }
    }

    // === Accessors ===

    /// Returns the spec ID from cfg.
    #[inline]
    pub fn spec(&self) -> SpecId {
        self.cfg.spec().into()
    }

    /// Returns the chain ID from cfg.
    #[inline]
    pub fn chain_id(&self) -> u64 {
        self.cfg.chain_id()
    }

    /// Returns the base fee from block, or None if BASE_FEE check is disabled.
    #[inline]
    pub fn base_fee(&self) -> Option<u128> {
        if self.checks().contains(ValidationChecks::BASE_FEE) {
            Some(self.block.basefee() as u128)
        } else {
            None
        }
    }

    /// Returns the blob gas price from block.
    #[inline]
    pub fn blob_gasprice(&self) -> Option<u128> {
        self.block.blob_gasprice()
    }

    /// Returns the transaction gas limit cap from cfg.
    #[inline]
    pub fn tx_gas_limit_cap(&self) -> u64 {
        self.cfg.tx_gas_limit_cap()
    }

    /// Returns the block gas limit.
    #[inline]
    pub fn block_gas_limit(&self) -> u64 {
        self.block.gas_limit()
    }

    /// Returns the maximum blobs per transaction from cfg.
    #[inline]
    pub fn max_blobs_per_tx(&self) -> Option<u64> {
        self.cfg.max_blobs_per_tx()
    }

    /// Returns the maximum initcode size from cfg.
    #[inline]
    pub fn max_initcode_size(&self) -> usize {
        self.cfg.max_initcode_size()
    }

    /// Returns the enabled validation checks.
    ///
    /// If checks were overridden via builder methods, returns the override.
    /// Otherwise returns `cfg.enabled_validation_checks()`.
    #[inline]
    pub fn checks(&self) -> ValidationChecks {
        self.checks_override
            .unwrap_or_else(|| self.cfg.enabled_validation_checks())
    }

    // === Builder methods for check configuration ===

    /// Set custom validation checks (replaces all existing checks).
    #[inline]
    pub fn with_checks(mut self, checks: ValidationChecks) -> Self {
        self.checks_override = Some(checks);
        self
    }

    /// Disable specific validation checks (additive removal).
    #[inline]
    pub fn with_disabled_checks(mut self, disabled: ValidationChecks) -> Self {
        let current = self.checks();
        self.checks_override = Some(current - disabled);
        self
    }

    /// Enable specific validation checks (additive insertion).
    #[inline]
    pub fn with_enabled_checks(mut self, enabled: ValidationChecks) -> Self {
        let current = self.checks();
        self.checks_override = Some(current | enabled);
        self
    }

    // === Skip methods ===

    /// Skip all validation checks.
    #[inline]
    pub fn skip_all(mut self) -> Self {
        self.checks_override = Some(ValidationChecks::empty());
        self
    }

    /// Skip chain ID validation (EIP-155).
    #[inline]
    pub fn skip_chain_id_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::CHAIN_ID)
    }

    /// Skip transaction gas limit cap validation (EIP-7825).
    #[inline]
    pub fn skip_tx_gas_limit_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::TX_GAS_LIMIT)
    }

    /// Skip base fee validation.
    #[inline]
    pub fn skip_base_fee_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::BASE_FEE)
    }

    /// Skip priority fee validation.
    #[inline]
    pub fn skip_priority_fee_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::PRIORITY_FEE)
    }

    /// Skip blob fee validation (EIP-4844).
    #[inline]
    pub fn skip_blob_fee_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::BLOB_FEE)
    }

    /// Skip authorization list validation (EIP-7702).
    #[inline]
    pub fn skip_auth_list_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::AUTH_LIST)
    }

    /// Skip block gas limit check.
    #[inline]
    pub fn skip_block_gas_limit_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::BLOCK_GAS_LIMIT)
    }

    /// Skip max initcode size validation (EIP-3860).
    #[inline]
    pub fn skip_max_initcode_size_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::MAX_INITCODE_SIZE)
    }

    /// Skip nonce validation.
    #[inline]
    pub fn skip_nonce_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::NONCE)
    }

    /// Skip balance validation.
    #[inline]
    pub fn skip_balance_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::BALANCE)
    }

    /// Skip EIP-3607 code check (reject senders with deployed code).
    #[inline]
    pub fn skip_eip3607_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::EIP3607)
    }

    /// Skip EIP-7623 floor gas check.
    #[inline]
    pub fn skip_eip7623_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::EIP7623)
    }

    /// Skip header validation.
    #[inline]
    pub fn skip_header_check(self) -> Self {
        self.with_disabled_checks(ValidationChecks::HEADER)
    }

    /// Skip all caller/state validation checks (nonce, balance, EIP-3607).
    #[inline]
    pub fn skip_caller_checks(self) -> Self {
        self.with_disabled_checks(ValidationChecks::CALLER)
    }

    /// Skip all gas and fee related checks.
    #[inline]
    pub fn skip_gas_fee_checks(self) -> Self {
        self.with_disabled_checks(ValidationChecks::GAS_FEES)
    }

    // === Enable methods (for use after skip_all) ===

    /// Enable all validation checks.
    #[inline]
    pub fn enable_all(mut self) -> Self {
        self.checks_override = Some(ValidationChecks::ALL);
        self
    }

    /// Enable chain ID validation (EIP-155).
    #[inline]
    pub fn enable_chain_id_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::CHAIN_ID)
    }

    /// Enable transaction gas limit cap validation (EIP-7825).
    #[inline]
    pub fn enable_tx_gas_limit_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::TX_GAS_LIMIT)
    }

    /// Enable base fee validation.
    #[inline]
    pub fn enable_base_fee_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::BASE_FEE)
    }

    /// Enable priority fee validation.
    #[inline]
    pub fn enable_priority_fee_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::PRIORITY_FEE)
    }

    /// Enable blob fee validation (EIP-4844).
    #[inline]
    pub fn enable_blob_fee_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::BLOB_FEE)
    }

    /// Enable authorization list validation (EIP-7702).
    #[inline]
    pub fn enable_auth_list_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::AUTH_LIST)
    }

    /// Enable block gas limit validation.
    #[inline]
    pub fn enable_block_gas_limit_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::BLOCK_GAS_LIMIT)
    }

    /// Enable max initcode size validation (EIP-3860).
    #[inline]
    pub fn enable_max_initcode_size_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::MAX_INITCODE_SIZE)
    }

    /// Enable nonce validation.
    #[inline]
    pub fn enable_nonce_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::NONCE)
    }

    /// Enable balance validation.
    #[inline]
    pub fn enable_balance_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::BALANCE)
    }

    /// Enable EIP-3607 code check (reject senders with deployed code).
    #[inline]
    pub fn enable_eip3607_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::EIP3607)
    }

    /// Enable EIP-7623 floor gas check.
    #[inline]
    pub fn enable_eip7623_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::EIP7623)
    }

    /// Enable header validation.
    #[inline]
    pub fn enable_header_check(self) -> Self {
        self.with_enabled_checks(ValidationChecks::HEADER)
    }

    /// Enable all gas and fee related checks.
    #[inline]
    pub fn enable_gas_fee_checks(self) -> Self {
        self.with_enabled_checks(ValidationChecks::GAS_FEES)
    }

    /// Enable all caller/state checks (nonce, balance, EIP-3607).
    #[inline]
    pub fn enable_caller_checks(self) -> Self {
        self.with_enabled_checks(ValidationChecks::CALLER)
    }

    // === Query methods ===

    /// Check if a specific validation is enabled.
    #[inline]
    pub fn should_check(&self, check: ValidationChecks) -> bool {
        self.checks().contains(check)
    }

    /// Check if any validation is enabled.
    #[inline]
    pub fn has_any_checks(&self) -> bool {
        !self.checks().is_empty()
    }

    /// Check if all validations are enabled.
    #[inline]
    pub fn has_all_checks(&self) -> bool {
        self.checks() == ValidationChecks::ALL
    }

    /// Returns the currently enabled validation checks.
    #[inline]
    pub fn enabled_checks(&self) -> ValidationChecks {
        self.checks()
    }

    // === Validation methods ===

    /// Validate block header fields.
    ///
    /// Checks that required header fields are present for the current spec:
    /// - `prevrandao` is required after the Merge
    /// - `excess_blob_gas` is required after Cancun
    #[inline]
    pub fn validate_header(&self) -> Result<(), InvalidHeader> {
        if !self.should_check(ValidationChecks::HEADER) {
            return Ok(());
        }

        let spec = self.spec();
        if spec.is_enabled_in(SpecId::MERGE) && self.block.prevrandao().is_none() {
            return Err(InvalidHeader::PrevrandaoNotSet);
        }

        if spec.is_enabled_in(SpecId::CANCUN) && self.block.blob_excess_gas_and_price().is_none() {
            return Err(InvalidHeader::ExcessBlobGasNotSet);
        }

        Ok(())
    }

    /// Validate transaction (stateless validation).
    ///
    /// This validates the transaction without requiring account state:
    /// - Chain ID
    /// - Gas limits and fees
    /// - Transaction type support
    /// - Blob and authorization list validation
    pub fn validate_tx(&self, tx: &impl Transaction) -> Result<(), InvalidTransaction> {
        let checks = self.checks();
        if checks.is_empty() {
            return Ok(());
        }

        let tx_type = TransactionType::from(tx.tx_type());
        let spec = self.spec();

        // Chain ID validation (EIP-155)
        if checks.contains(ValidationChecks::CHAIN_ID) {
            self.validate_chain_id(tx, tx_type)?;
        }

        // Transaction gas limit cap (EIP-7825)
        if checks.contains(ValidationChecks::TX_GAS_LIMIT)
            && tx.gas_limit() > self.tx_gas_limit_cap()
        {
            return Err(InvalidTransaction::TxGasLimitGreaterThanCap {
                gas_limit: tx.gas_limit(),
                cap: self.tx_gas_limit_cap(),
            });
        }

        // Type-specific validation
        self.validate_tx_type(tx, tx_type, spec, checks)?;

        // Block gas limit check
        if checks.contains(ValidationChecks::BLOCK_GAS_LIMIT)
            && tx.gas_limit() > self.block_gas_limit()
        {
            return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
        }

        // Initcode size limit (EIP-3860)
        if checks.contains(ValidationChecks::MAX_INITCODE_SIZE)
            && spec.is_enabled_in(SpecId::SHANGHAI)
            && tx.kind().is_create()
            && tx.input().len() > self.max_initcode_size()
        {
            return Err(InvalidTransaction::CreateInitCodeSizeLimit);
        }

        Ok(())
    }

    /// Calculate initial and floor gas for the transaction.
    #[inline]
    pub fn initial_gas(
        &self,
        tx: &impl Transaction,
    ) -> Result<InitialAndFloorGas, InvalidTransaction> {
        let spec = self.spec();
        let mut gas = calculate_initial_tx_gas_for_tx(tx, spec);

        if !self.should_check(ValidationChecks::EIP7623) {
            gas.floor_gas = 0;
        }

        if gas.initial_gas > tx.gas_limit() {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit {
                gas_limit: tx.gas_limit(),
                initial_gas: gas.initial_gas,
            });
        }

        if spec.is_enabled_in(SpecId::PRAGUE) && gas.floor_gas > tx.gas_limit() {
            return Err(InvalidTransaction::GasFloorMoreThanGasLimit {
                gas_floor: gas.floor_gas,
                gas_limit: tx.gas_limit(),
            });
        }

        Ok(gas)
    }

    /// Validate caller account state (nonce, EIP-3607 code check).
    pub fn validate_caller(
        &self,
        caller_info: &AccountInfo,
        tx_nonce: u64,
    ) -> Result<(), InvalidTransaction> {
        let checks = self.checks();

        // EIP-3607: Reject transactions from senders with deployed code
        if checks.contains(ValidationChecks::EIP3607) {
            if let Some(code) = caller_info.code.as_ref() {
                // Reject if code is non-empty and not an EIP-7702 delegation
                if !code.is_empty() && !code.is_eip7702() {
                    return Err(InvalidTransaction::RejectCallerWithCode);
                }
            }
            // If code is None, it's empty - check passes
        }

        // Nonce validation
        if checks.contains(ValidationChecks::NONCE) {
            let state = caller_info.nonce;
            match tx_nonce.cmp(&state) {
                Ordering::Greater => {
                    return Err(InvalidTransaction::NonceTooHigh {
                        tx: tx_nonce,
                        state,
                    });
                }
                Ordering::Less => {
                    return Err(InvalidTransaction::NonceTooLow {
                        tx: tx_nonce,
                        state,
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Calculate fee to deduct from caller.
    ///
    /// Returns the new balance and the gas fee to deduct.
    /// Does NOT mutate state - the caller is responsible for applying the balance change.
    #[inline]
    pub fn caller_fee(
        &self,
        caller_balance: U256,
        tx: &impl Transaction,
    ) -> Result<CallerFee, InvalidTransaction> {
        let basefee = self.base_fee().unwrap_or(0);
        let blob_price = self.blob_gasprice().unwrap_or(0);
        let checks = self.checks();

        if checks.contains(ValidationChecks::BALANCE) {
            tx.ensure_enough_balance(caller_balance)?;
        }

        let effective_balance_spending = tx
            .effective_balance_spending(basefee, blob_price)
            .expect("effective balance is always smaller than max balance so it can't overflow");

        let gas_balance_spending = effective_balance_spending - tx.value();

        let mut new_balance = caller_balance.saturating_sub(gas_balance_spending);

        if !checks.contains(ValidationChecks::BALANCE) {
            new_balance = new_balance.max(tx.value());
        }

        Ok(CallerFee {
            gas_fee_to_deduct: gas_balance_spending,
            new_balance,
        })
    }

    // === Internal helpers ===

    #[inline]
    fn validate_chain_id(
        &self,
        tx: &impl Transaction,
        tx_type: TransactionType,
    ) -> Result<(), InvalidTransaction> {
        if let Some(tx_chain_id) = tx.chain_id() {
            if tx_chain_id != self.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }
        } else if !tx_type.is_legacy() && !tx_type.is_custom() {
            return Err(InvalidTransaction::MissingChainId);
        }
        Ok(())
    }

    fn validate_tx_type(
        &self,
        tx: &impl Transaction,
        tx_type: TransactionType,
        spec: SpecId,
        checks: ValidationChecks,
    ) -> Result<(), InvalidTransaction> {
        match tx_type {
            TransactionType::Legacy => {
                if checks.contains(ValidationChecks::BASE_FEE) {
                    validate_legacy_gas_price(tx.gas_price(), self.base_fee())?;
                }
            }
            TransactionType::Eip2930 => {
                if !spec.is_enabled_in(SpecId::BERLIN) {
                    return Err(InvalidTransaction::Eip2930NotSupported);
                }
                if checks.contains(ValidationChecks::BASE_FEE) {
                    validate_legacy_gas_price(tx.gas_price(), self.base_fee())?;
                }
            }
            TransactionType::Eip1559 => {
                if !spec.is_enabled_in(SpecId::LONDON) {
                    return Err(InvalidTransaction::Eip1559NotSupported);
                }
                self.validate_eip1559_fees(tx, checks)?;
            }
            TransactionType::Eip4844 => {
                if !spec.is_enabled_in(SpecId::CANCUN) {
                    return Err(InvalidTransaction::Eip4844NotSupported);
                }
                self.validate_eip1559_fees(tx, checks)?;
                if checks.contains(ValidationChecks::BLOB_FEE) {
                    validate_eip4844_tx(
                        tx.blob_versioned_hashes(),
                        tx.max_fee_per_blob_gas(),
                        self.blob_gasprice().unwrap_or(0),
                        self.max_blobs_per_tx(),
                    )?;
                }
            }
            TransactionType::Eip7702 => {
                if !spec.is_enabled_in(SpecId::PRAGUE) {
                    return Err(InvalidTransaction::Eip7702NotSupported);
                }
                self.validate_eip1559_fees(tx, checks)?;
                if checks.contains(ValidationChecks::AUTH_LIST) && tx.authorization_list_len() == 0
                {
                    return Err(InvalidTransaction::EmptyAuthorizationList);
                }
            }
            TransactionType::Custom => {}
        }
        Ok(())
    }

    #[inline]
    fn validate_eip1559_fees(
        &self,
        tx: &impl Transaction,
        checks: ValidationChecks,
    ) -> Result<(), InvalidTransaction> {
        let check_base_fee = checks.contains(ValidationChecks::BASE_FEE);
        let check_priority_fee = checks.contains(ValidationChecks::PRIORITY_FEE);

        if !check_base_fee && !check_priority_fee {
            return Ok(());
        }

        validate_priority_fee(
            tx.max_fee_per_gas(),
            tx.max_priority_fee_per_gas().unwrap_or_default(),
            if check_base_fee {
                self.base_fee()
            } else {
                None
            },
            !check_priority_fee,
        )
    }
}

// === Standalone helper functions ===

/// Validate legacy transaction gas price against basefee.
#[inline]
pub fn validate_legacy_gas_price(
    gas_price: u128,
    base_fee: Option<u128>,
) -> Result<(), InvalidTransaction> {
    if let Some(base_fee) = base_fee {
        if gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }
    Ok(())
}

/// Validate priority fee for EIP-1559+ transactions.
#[inline]
pub fn validate_priority_fee(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<u128>,
    skip_priority_check: bool,
) -> Result<(), InvalidTransaction> {
    if !skip_priority_check && max_priority_fee > max_fee {
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    if let Some(base_fee) = base_fee {
        let effective_gas_price = cmp::min(max_fee, base_fee.saturating_add(max_priority_fee));
        if effective_gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }

    Ok(())
}

/// Validate EIP-4844 blob transaction.
#[inline]
pub fn validate_eip4844_tx(
    blobs: &[B256],
    max_blob_fee: u128,
    block_blob_gas_price: u128,
    max_blobs: Option<u64>,
) -> Result<(), InvalidTransaction> {
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax {
            block_blob_gas_price,
            tx_max_fee_per_blob_gas: max_blob_fee,
        });
    }

    if blobs.is_empty() {
        return Err(InvalidTransaction::EmptyBlobs);
    }

    for blob in blobs {
        if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
            return Err(InvalidTransaction::BlobVersionNotSupported);
        }
    }

    if let Some(max_blobs) = max_blobs {
        if blobs.len() > max_blobs as usize {
            return Err(InvalidTransaction::TooManyBlobs {
                have: blobs.len(),
                max: max_blobs as usize,
            });
        }
    }

    Ok(())
}

/// Result of caller fee calculation.
///
/// Returned by [`TxValidator::caller_fee`] to provide both the fee to deduct
/// and the resulting balance.
///
/// # Usage
///
/// After calling `caller_fee()`, use these values to update state:
/// - Set caller's balance to `new_balance`
/// - The `gas_fee_to_deduct` is provided for accounting/logging purposes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallerFee {
    /// The gas fee amount calculated for the transaction.
    ///
    /// This is the value that was subtracted from the caller's balance to
    /// compute `new_balance`. It does NOT include the transaction value.
    pub gas_fee_to_deduct: U256,
    /// The caller's new balance after deducting gas fees.
    ///
    /// This is the balance that should be set on the caller's account.
    /// The transaction value has NOT been deducted from this balance yet.
    pub new_balance: U256,
}

// === Legacy API (for backward compatibility) ===

/// Specifies how transaction validation should be performed.
///
/// This is kept for backward compatibility. New code should use [`TxValidator`] directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValidationKind {
    /// Skip all validation checks.
    None,
    /// Perform validation based on transaction type (default).
    #[default]
    ByTxType,
    /// Perform custom validation with specific checks enabled.
    Custom(ValidationChecks),
}

impl ValidationKind {
    /// Returns true if the given check should be performed.
    #[inline]
    pub fn should_check(&self, check: ValidationChecks) -> bool {
        match self {
            ValidationKind::None => false,
            ValidationKind::ByTxType => true,
            ValidationKind::Custom(checks) => checks.contains(check),
        }
    }
}

/// Legacy parameters struct for backward compatibility.
#[derive(Debug, Clone)]
pub struct ValidationParams {
    /// Ethereum specification version.
    pub spec: SpecId,
    /// Chain ID for validation.
    pub chain_id: u64,
    /// Base fee from block.
    pub base_fee: Option<u128>,
    /// Blob gas price from block.
    pub blob_gasprice: Option<u128>,
    /// Transaction gas limit cap.
    pub tx_gas_limit_cap: u64,
    /// Block gas limit.
    pub block_gas_limit: u64,
    /// Maximum blobs per transaction.
    pub max_blobs_per_tx: Option<u64>,
    /// Maximum initcode size.
    pub max_initcode_size: usize,
    /// Validation kind to use.
    pub validation_kind: ValidationKind,
}

impl ValidationParams {
    /// Create ValidationParams from Cfg and Block traits.
    pub fn from_cfg_and_block(cfg: &impl Cfg, block: &impl Block) -> Self {
        let checks = cfg.enabled_validation_checks();
        Self {
            spec: cfg.spec().into(),
            chain_id: cfg.chain_id(),
            base_fee: if checks.contains(ValidationChecks::BASE_FEE) {
                Some(block.basefee() as u128)
            } else {
                None
            },
            blob_gasprice: block.blob_gasprice(),
            tx_gas_limit_cap: cfg.tx_gas_limit_cap(),
            block_gas_limit: block.gas_limit(),
            max_blobs_per_tx: cfg.max_blobs_per_tx(),
            max_initcode_size: cfg.max_initcode_size(),
            validation_kind: if checks == ValidationChecks::ALL {
                ValidationKind::ByTxType
            } else if checks.is_empty() {
                ValidationKind::None
            } else {
                ValidationKind::Custom(checks)
            },
        }
    }

    /// Create params for caller validation from Cfg trait.
    pub fn caller_params_from_cfg(cfg: &impl Cfg) -> Self {
        let disabled = cfg.disabled_validation_checks();
        let mut checks = ValidationChecks::CALLER - disabled;
        // Only keep caller-relevant checks
        checks &= ValidationChecks::CALLER;

        Self {
            spec: cfg.spec().into(),
            chain_id: cfg.chain_id(),
            base_fee: None,
            blob_gasprice: None,
            tx_gas_limit_cap: u64::MAX,
            block_gas_limit: u64::MAX,
            max_blobs_per_tx: None,
            max_initcode_size: usize::MAX,
            validation_kind: if checks == ValidationChecks::CALLER {
                ValidationKind::ByTxType
            } else if checks.is_empty() {
                ValidationKind::None
            } else {
                ValidationKind::Custom(checks)
            },
        }
    }
}

// === Legacy standalone functions ===

/// Validate caller account state.
pub fn validate_caller(
    caller_info: &AccountInfo,
    tx_nonce: u64,
    kind: ValidationKind,
) -> Result<(), InvalidTransaction> {
    let checks = match kind {
        ValidationKind::None => ValidationChecks::empty(),
        ValidationKind::ByTxType => ValidationChecks::CALLER,
        ValidationKind::Custom(c) => c,
    };

    // EIP-3607: Reject transactions from senders with deployed code
    if checks.contains(ValidationChecks::EIP3607) {
        if let Some(code) = caller_info.code.as_ref() {
            if !code.is_empty() && !code.is_eip7702() {
                return Err(InvalidTransaction::RejectCallerWithCode);
            }
        }
    }

    // Nonce validation
    if checks.contains(ValidationChecks::NONCE) {
        let state = caller_info.nonce;
        match tx_nonce.cmp(&state) {
            Ordering::Greater => {
                return Err(InvalidTransaction::NonceTooHigh {
                    tx: tx_nonce,
                    state,
                });
            }
            Ordering::Less => {
                return Err(InvalidTransaction::NonceTooLow {
                    tx: tx_nonce,
                    state,
                });
            }
            _ => {}
        }
    }

    Ok(())
}

/// Calculate fee to deduct from caller.
pub fn calculate_caller_fee(
    caller_balance: U256,
    tx: &impl Transaction,
    block: &impl Block,
    skip_balance_check: bool,
) -> Result<CallerFee, InvalidTransaction> {
    let basefee = block.basefee() as u128;
    let blob_price = block.blob_gasprice().unwrap_or(0);

    if !skip_balance_check {
        tx.ensure_enough_balance(caller_balance)?;
    }

    let effective_balance_spending = tx
        .effective_balance_spending(basefee, blob_price)
        .expect("effective balance is always smaller than max balance so it can't overflow");

    let gas_balance_spending = effective_balance_spending - tx.value();

    let mut new_balance = caller_balance.saturating_sub(gas_balance_spending);

    if skip_balance_check {
        new_balance = new_balance.max(tx.value());
    }

    Ok(CallerFee {
        gas_fee_to_deduct: gas_balance_spending,
        new_balance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::{BlockEnv, CfgEnv};

    #[test]
    fn test_tx_validator_with_references() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        // Create validator with references
        let validator = TxValidator::new(&cfg, &block);

        assert_eq!(validator.spec(), SpecId::CANCUN);
        assert!(validator.has_all_checks());
    }

    #[test]
    fn test_tx_validator_skip_checks() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::new(&cfg, &block)
            .skip_nonce_check()
            .skip_balance_check();

        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
    }

    #[test]
    fn test_tx_validator_for_deposit() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::for_deposit(&cfg, &block);

        // Should have chain ID and block gas limit checks
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(validator.should_check(ValidationChecks::AUTH_LIST));
        assert!(validator.should_check(ValidationChecks::MAX_INITCODE_SIZE));

        // Should NOT have fee and state checks
        assert!(!validator.should_check(ValidationChecks::BASE_FEE));
        assert!(!validator.should_check(ValidationChecks::PRIORITY_FEE));
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
    }

    #[test]
    fn test_tx_validator_for_tx_pool() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::for_tx_pool(&cfg, &block);

        // Should have fee checks
        assert!(validator.should_check(ValidationChecks::BASE_FEE));
        assert!(validator.should_check(ValidationChecks::PRIORITY_FEE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));

        // Should NOT have block-specific checks
        assert!(!validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(!validator.should_check(ValidationChecks::HEADER));

        // Should NOT have state checks
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
    }

    #[test]
    fn test_tx_validator_for_block_builder() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::for_block_builder(&cfg, &block);

        // Should have all stateless checks
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(validator.should_check(ValidationChecks::BASE_FEE));
        assert!(validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));

        // Should NOT have state checks
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(!validator.should_check(ValidationChecks::EIP3607));
    }

    #[test]
    fn test_tx_validator_skip_all_enable() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::new(&cfg, &block)
            .skip_all()
            .enable_chain_id_check()
            .enable_nonce_check();

        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(!validator.should_check(ValidationChecks::BASE_FEE));
    }

    #[test]
    fn test_cfg_disabled_checks_integration() {
        // Create cfg with disabled checks
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN)
            .disable_nonce_check()
            .disable_balance_check();
        let block = BlockEnv::default();

        // Validator should respect cfg's disabled checks
        let validator = TxValidator::new(&cfg, &block);

        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
    }

    #[test]
    fn test_validate_caller_empty_checks() {
        let cfg = CfgEnv::new_with_spec(SpecId::CANCUN);
        let block = BlockEnv::default();

        let validator = TxValidator::new(&cfg, &block).skip_all();
        let caller_info = AccountInfo {
            nonce: 999,                            // wrong nonce
            code: Some(bytecode::Bytecode::new()), // has code
            ..Default::default()
        };
        // Should pass because all checks are skipped
        assert!(validator.validate_caller(&caller_info, 0).is_ok());
    }

    #[test]
    fn test_validation_checks_composites() {
        // Test that composite flags work as expected
        assert_eq!(
            ValidationChecks::ALL,
            ValidationChecks::TX_STATELESS | ValidationChecks::CALLER
        );

        let checks = ValidationChecks::CALLER;
        assert!(checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
        assert!(checks.contains(ValidationChecks::EIP3607));
    }
}
