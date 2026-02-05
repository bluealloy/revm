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
//! use primitives::hardfork::SpecId;
//!
//! // Create validator from EVM context
//! let validator = TxValidator::from_cfg_and_block(&cfg, &block);
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
//! let validator = TxValidator::new(SpecId::CANCUN).skip_all();
//!
//! // Skip specific checks
//! let validator = TxValidator::new(SpecId::CANCUN)
//!     .skip_nonce_check()
//!     .skip_balance_check();
//!
//! // Enable only specific checks
//! let validator = TxValidator::new(SpecId::CANCUN)
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

/// Transaction validator with configurable checks.
///
/// This struct provides a fluent API for validating Ethereum transactions.
/// It can be configured to skip certain checks (e.g., for L2 deposit transactions)
/// or to validate only specific aspects of a transaction.
///
/// All builder methods return `Self` and should be chained. The `#[must_use]`
/// attribute ensures the result is not accidentally discarded.
///
/// # Example
///
/// ```ignore
/// // Standard validation
/// let validator = TxValidator::from_cfg_and_block(&cfg, &block);
/// validator.validate_tx(&tx)?;
///
/// // Optimism deposit - skip fee checks
/// let validator = TxValidator::new(SpecId::CANCUN)
///     .skip_base_fee_check()
///     .skip_priority_fee_check()
///     .skip_balance_check();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct TxValidator {
    /// Ethereum specification version.
    pub spec: SpecId,
    /// Chain ID for validation.
    pub chain_id: u64,
    /// Base fee from block (None skips base fee check).
    pub base_fee: Option<u128>,
    /// Blob gas price from block.
    pub blob_gasprice: Option<u128>,
    /// Transaction gas limit cap (EIP-7825).
    pub tx_gas_limit_cap: u64,
    /// Block gas limit.
    pub block_gas_limit: u64,
    /// Maximum blobs per transaction.
    pub max_blobs_per_tx: Option<u64>,
    /// Maximum initcode size.
    pub max_initcode_size: usize,
    /// Enabled validation checks.
    pub checks: ValidationChecks,
}

impl Default for TxValidator {
    #[inline]
    fn default() -> Self {
        Self {
            spec: SpecId::PRAGUE,
            chain_id: 1,
            base_fee: None,
            blob_gasprice: None,
            tx_gas_limit_cap: u64::MAX,
            block_gas_limit: u64::MAX,
            max_blobs_per_tx: None,
            max_initcode_size: primitives::eip3860::MAX_INITCODE_SIZE,
            checks: ValidationChecks::ALL,
        }
    }
}

impl TxValidator {
    /// Create a new validator for the given spec with all checks enabled.
    #[inline]
    pub fn new(spec: SpecId) -> Self {
        Self {
            spec,
            ..Default::default()
        }
    }

    // === Preset Constructors ===

    /// Create validator for L2 deposit transactions.
    ///
    /// Deposit transactions (like Optimism's system deposits) skip most validation:
    /// - No fee checks (base fee, priority fee, blob fee)
    /// - No balance check (system-funded)
    /// - No nonce check (system-managed)
    /// - Keeps: chain ID, block gas limit, initcode size, auth list
    #[inline]
    pub fn for_deposit(spec: SpecId) -> Self {
        Self {
            spec,
            checks: ValidationChecks::CHAIN_ID
                | ValidationChecks::BLOCK_GAS_LIMIT
                | ValidationChecks::AUTH_LIST
                | ValidationChecks::MAX_INITCODE_SIZE,
            ..Default::default()
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
    pub fn for_tx_pool(spec: SpecId) -> Self {
        Self {
            spec,
            checks: ValidationChecks::TX_STATELESS
                - ValidationChecks::BLOCK_GAS_LIMIT
                - ValidationChecks::HEADER,
            ..Default::default()
        }
    }

    /// Create validator for block builders.
    ///
    /// Block builders have more flexibility:
    /// - Skip nonce check (can reorder transactions)
    /// - Skip balance check (can ensure profitability differently)
    /// - Keep all stateless transaction checks
    #[inline]
    pub fn for_block_builder(spec: SpecId) -> Self {
        Self {
            spec,
            checks: ValidationChecks::TX_STATELESS,
            ..Default::default()
        }
    }

    /// Create validator from Cfg and Block traits.
    ///
    /// This is the recommended way to create a validator when you have
    /// access to the EVM context.
    #[inline]
    pub fn from_cfg_and_block(cfg: &impl Cfg, block: &impl Block) -> Self {
        // Get disabled checks in a single call (pre-computed or aggregated in Cfg)
        let disabled = cfg.disabled_validation_checks();
        let checks = ValidationChecks::ALL - disabled;

        Self {
            spec: cfg.spec().into(),
            chain_id: cfg.chain_id(),
            base_fee: if disabled.contains(ValidationChecks::BASE_FEE) {
                None
            } else {
                Some(block.basefee() as u128)
            },
            blob_gasprice: block.blob_gasprice(),
            tx_gas_limit_cap: cfg.tx_gas_limit_cap(),
            block_gas_limit: block.gas_limit(),
            max_blobs_per_tx: cfg.max_blobs_per_tx(),
            max_initcode_size: cfg.max_initcode_size(),
            checks,
        }
    }

    // === Builder methods for configuration ===

    /// Set the chain ID.
    #[inline]
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Set the base fee.
    #[inline]
    pub fn with_base_fee(mut self, base_fee: u128) -> Self {
        self.base_fee = Some(base_fee);
        self
    }

    /// Set the blob gas price.
    #[inline]
    pub fn with_blob_gasprice(mut self, price: u128) -> Self {
        self.blob_gasprice = Some(price);
        self
    }

    /// Set the block gas limit.
    #[inline]
    pub fn with_block_gas_limit(mut self, limit: u64) -> Self {
        self.block_gas_limit = limit;
        self
    }

    /// Set the transaction gas limit cap.
    #[inline]
    pub fn with_tx_gas_limit_cap(mut self, cap: u64) -> Self {
        self.tx_gas_limit_cap = cap;
        self
    }

    /// Set maximum blobs per transaction.
    #[inline]
    pub fn with_max_blobs(mut self, max: u64) -> Self {
        self.max_blobs_per_tx = Some(max);
        self
    }

    /// Set maximum initcode size.
    #[inline]
    pub fn with_max_initcode_size(mut self, size: usize) -> Self {
        self.max_initcode_size = size;
        self
    }

    /// Set custom validation checks (replaces all existing checks).
    #[inline]
    pub fn with_checks(mut self, checks: ValidationChecks) -> Self {
        self.checks = checks;
        self
    }

    /// Set the spec version.
    #[inline]
    pub fn with_spec(mut self, spec: SpecId) -> Self {
        self.spec = spec;
        self
    }

    /// Disable specific validation checks (additive removal).
    ///
    /// Unlike [`with_checks`](Self::with_checks), this removes checks from the
    /// current set rather than replacing all checks.
    #[inline]
    pub fn with_disabled_checks(mut self, checks: ValidationChecks) -> Self {
        self.checks.remove(checks);
        self
    }

    /// Enable specific validation checks (additive insertion).
    ///
    /// Unlike [`with_checks`](Self::with_checks), this adds checks to the
    /// current set rather than replacing all checks.
    #[inline]
    pub fn with_enabled_checks(mut self, checks: ValidationChecks) -> Self {
        self.checks.insert(checks);
        self
    }

    // === Skip methods ===

    /// Skip all validation checks.
    #[inline]
    pub fn skip_all(mut self) -> Self {
        self.checks = ValidationChecks::empty();
        self
    }

    /// Skip chain ID validation (EIP-155).
    #[inline]
    pub fn skip_chain_id_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::CHAIN_ID);
        self
    }

    /// Skip transaction gas limit cap validation (EIP-7825).
    #[inline]
    pub fn skip_tx_gas_limit_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::TX_GAS_LIMIT);
        self
    }

    /// Skip base fee validation.
    #[inline]
    pub fn skip_base_fee_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BASE_FEE);
        self
    }

    /// Skip priority fee validation.
    #[inline]
    pub fn skip_priority_fee_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::PRIORITY_FEE);
        self
    }

    /// Skip blob fee validation (EIP-4844).
    #[inline]
    pub fn skip_blob_fee_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BLOB_FEE);
        self
    }

    /// Skip authorization list validation (EIP-7702).
    #[inline]
    pub fn skip_auth_list_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::AUTH_LIST);
        self
    }

    /// Skip block gas limit check.
    #[inline]
    pub fn skip_block_gas_limit_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BLOCK_GAS_LIMIT);
        self
    }

    /// Skip max initcode size validation (EIP-3860).
    #[inline]
    pub fn skip_max_initcode_size_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::MAX_INITCODE_SIZE);
        self
    }

    /// Skip nonce validation.
    #[inline]
    pub fn skip_nonce_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::NONCE);
        self
    }

    /// Skip balance validation.
    #[inline]
    pub fn skip_balance_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BALANCE);
        self
    }

    /// Skip EIP-3607 code check (reject senders with deployed code).
    #[inline]
    pub fn skip_eip3607_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::EIP3607);
        self
    }

    /// Skip EIP-7623 floor gas check.
    #[inline]
    pub fn skip_eip7623_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::EIP7623);
        self
    }

    /// Skip header validation.
    #[inline]
    pub fn skip_header_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::HEADER);
        self
    }

    /// Skip all caller/state validation checks (nonce, balance, EIP-3607).
    #[inline]
    pub fn skip_caller_checks(mut self) -> Self {
        self.checks.remove(ValidationChecks::CALLER);
        self
    }

    /// Skip all gas and fee related checks.
    #[inline]
    pub fn skip_gas_fee_checks(mut self) -> Self {
        self.checks.remove(ValidationChecks::GAS_FEES);
        self
    }

    // === Enable methods (for use after skip_all) ===

    /// Enable all validation checks.
    #[inline]
    pub fn enable_all(mut self) -> Self {
        self.checks = ValidationChecks::ALL;
        self
    }

    /// Enable chain ID validation (EIP-155).
    #[inline]
    pub fn enable_chain_id_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::CHAIN_ID);
        self
    }

    /// Enable transaction gas limit cap validation (EIP-7825).
    #[inline]
    pub fn enable_tx_gas_limit_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::TX_GAS_LIMIT);
        self
    }

    /// Enable base fee validation.
    #[inline]
    pub fn enable_base_fee_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::BASE_FEE);
        self
    }

    /// Enable priority fee validation.
    #[inline]
    pub fn enable_priority_fee_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::PRIORITY_FEE);
        self
    }

    /// Enable blob fee validation (EIP-4844).
    #[inline]
    pub fn enable_blob_fee_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::BLOB_FEE);
        self
    }

    /// Enable authorization list validation (EIP-7702).
    #[inline]
    pub fn enable_auth_list_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::AUTH_LIST);
        self
    }

    /// Enable block gas limit validation.
    #[inline]
    pub fn enable_block_gas_limit_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::BLOCK_GAS_LIMIT);
        self
    }

    /// Enable max initcode size validation (EIP-3860).
    #[inline]
    pub fn enable_max_initcode_size_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::MAX_INITCODE_SIZE);
        self
    }

    /// Enable nonce validation.
    #[inline]
    pub fn enable_nonce_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::NONCE);
        self
    }

    /// Enable balance validation.
    #[inline]
    pub fn enable_balance_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::BALANCE);
        self
    }

    /// Enable EIP-3607 code check (reject senders with deployed code).
    #[inline]
    pub fn enable_eip3607_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::EIP3607);
        self
    }

    /// Enable EIP-7623 floor gas check.
    #[inline]
    pub fn enable_eip7623_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::EIP7623);
        self
    }

    /// Enable header validation.
    #[inline]
    pub fn enable_header_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::HEADER);
        self
    }

    /// Enable all gas and fee related checks.
    #[inline]
    pub fn enable_gas_fee_checks(mut self) -> Self {
        self.checks.insert(ValidationChecks::GAS_FEES);
        self
    }

    /// Enable all caller/state checks (nonce, balance, EIP-3607).
    #[inline]
    pub fn enable_caller_checks(mut self) -> Self {
        self.checks.insert(ValidationChecks::CALLER);
        self
    }

    // === Query methods ===

    /// Check if a specific validation is enabled.
    #[inline]
    pub fn should_check(&self, check: ValidationChecks) -> bool {
        self.checks.contains(check)
    }

    /// Check if any validation is enabled.
    #[inline]
    pub fn has_any_checks(&self) -> bool {
        !self.checks.is_empty()
    }

    /// Check if all validations are enabled.
    #[inline]
    pub fn has_all_checks(&self) -> bool {
        self.checks == ValidationChecks::ALL
    }

    /// Returns the currently enabled validation checks.
    #[inline]
    pub fn enabled_checks(&self) -> ValidationChecks {
        self.checks
    }

    // === Validation methods ===

    /// Validate block header fields.
    ///
    /// Checks that required header fields are present for the current spec:
    /// - `prevrandao` is required after the Merge
    /// - `excess_blob_gas` is required after Cancun
    #[inline]
    pub fn validate_header(&self, block: &impl Block) -> Result<(), InvalidHeader> {
        if !self.should_check(ValidationChecks::HEADER) {
            return Ok(());
        }

        if self.spec.is_enabled_in(SpecId::MERGE) && block.prevrandao().is_none() {
            return Err(InvalidHeader::PrevrandaoNotSet);
        }

        if self.spec.is_enabled_in(SpecId::CANCUN) && block.blob_excess_gas_and_price().is_none() {
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
        if self.checks.is_empty() {
            return Ok(());
        }

        let tx_type = TransactionType::from(tx.tx_type());

        // Chain ID validation (EIP-155)
        if self.should_check(ValidationChecks::CHAIN_ID) {
            self.validate_chain_id(tx, tx_type)?;
        }

        // Transaction gas limit cap (EIP-7825)
        if self.should_check(ValidationChecks::TX_GAS_LIMIT)
            && tx.gas_limit() > self.tx_gas_limit_cap
        {
            return Err(InvalidTransaction::TxGasLimitGreaterThanCap {
                gas_limit: tx.gas_limit(),
                cap: self.tx_gas_limit_cap,
            });
        }

        // Type-specific validation
        self.validate_tx_type(tx, tx_type)?;

        // Block gas limit check
        if self.should_check(ValidationChecks::BLOCK_GAS_LIMIT)
            && tx.gas_limit() > self.block_gas_limit
        {
            return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
        }

        // Initcode size limit (EIP-3860)
        if self.should_check(ValidationChecks::MAX_INITCODE_SIZE)
            && self.spec.is_enabled_in(SpecId::SHANGHAI)
            && tx.kind().is_create()
            && tx.input().len() > self.max_initcode_size
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
        let mut gas = calculate_initial_tx_gas_for_tx(tx, self.spec);

        if !self.should_check(ValidationChecks::EIP7623) {
            gas.floor_gas = 0;
        }

        if gas.initial_gas > tx.gas_limit() {
            return Err(InvalidTransaction::CallGasCostMoreThanGasLimit {
                gas_limit: tx.gas_limit(),
                initial_gas: gas.initial_gas,
            });
        }

        if self.spec.is_enabled_in(SpecId::PRAGUE) && gas.floor_gas > tx.gas_limit() {
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
        // EIP-3607: Reject transactions from senders with deployed code
        if self.should_check(ValidationChecks::EIP3607) {
            if let Some(code) = caller_info.code.as_ref() {
                // Reject if code is non-empty and not an EIP-7702 delegation
                if !code.is_empty() && !code.is_eip7702() {
                    return Err(InvalidTransaction::RejectCallerWithCode);
                }
            }
            // If code is None, it's empty - check passes
        }

        // Nonce validation
        if self.should_check(ValidationChecks::NONCE) {
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
        let basefee = self.base_fee.unwrap_or(0);
        let blob_price = self.blob_gasprice.unwrap_or(0);

        if self.should_check(ValidationChecks::BALANCE) {
            tx.ensure_enough_balance(caller_balance)?;
        }

        let effective_balance_spending = tx
            .effective_balance_spending(basefee, blob_price)
            .expect("effective balance is always smaller than max balance so it can't overflow");

        let gas_balance_spending = effective_balance_spending - tx.value();

        let mut new_balance = caller_balance.saturating_sub(gas_balance_spending);

        if !self.should_check(ValidationChecks::BALANCE) {
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
            if tx_chain_id != self.chain_id {
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
    ) -> Result<(), InvalidTransaction> {
        match tx_type {
            TransactionType::Legacy => {
                if self.should_check(ValidationChecks::BASE_FEE) {
                    validate_legacy_gas_price(tx.gas_price(), self.base_fee)?;
                }
            }
            TransactionType::Eip2930 => {
                if !self.spec.is_enabled_in(SpecId::BERLIN) {
                    return Err(InvalidTransaction::Eip2930NotSupported);
                }
                if self.should_check(ValidationChecks::BASE_FEE) {
                    validate_legacy_gas_price(tx.gas_price(), self.base_fee)?;
                }
            }
            TransactionType::Eip1559 => {
                if !self.spec.is_enabled_in(SpecId::LONDON) {
                    return Err(InvalidTransaction::Eip1559NotSupported);
                }
                self.validate_eip1559_fees(tx)?;
            }
            TransactionType::Eip4844 => {
                if !self.spec.is_enabled_in(SpecId::CANCUN) {
                    return Err(InvalidTransaction::Eip4844NotSupported);
                }
                self.validate_eip1559_fees(tx)?;
                if self.should_check(ValidationChecks::BLOB_FEE) {
                    validate_eip4844_tx(
                        tx.blob_versioned_hashes(),
                        tx.max_fee_per_blob_gas(),
                        self.blob_gasprice.unwrap_or(0),
                        self.max_blobs_per_tx,
                    )?;
                }
            }
            TransactionType::Eip7702 => {
                if !self.spec.is_enabled_in(SpecId::PRAGUE) {
                    return Err(InvalidTransaction::Eip7702NotSupported);
                }
                self.validate_eip1559_fees(tx)?;
                if self.should_check(ValidationChecks::AUTH_LIST)
                    && tx.authorization_list_len() == 0
                {
                    return Err(InvalidTransaction::EmptyAuthorizationList);
                }
            }
            TransactionType::Custom => {}
        }
        Ok(())
    }

    #[inline]
    fn validate_eip1559_fees(&self, tx: &impl Transaction) -> Result<(), InvalidTransaction> {
        let check_base_fee = self.should_check(ValidationChecks::BASE_FEE);
        let check_priority_fee = self.should_check(ValidationChecks::PRIORITY_FEE);

        if !check_base_fee && !check_priority_fee {
            return Ok(());
        }

        validate_priority_fee(
            tx.max_fee_per_gas(),
            tx.max_priority_fee_per_gas().unwrap_or_default(),
            if check_base_fee { self.base_fee } else { None },
            !check_priority_fee,
        )
    }
}

// === Standalone helper functions (kept for backward compatibility) ===

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

// === Legacy API (for backward compatibility with validation.rs and pre_execution.rs) ===

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
        let validator = TxValidator::from_cfg_and_block(cfg, block);
        Self {
            spec: validator.spec,
            chain_id: validator.chain_id,
            base_fee: validator.base_fee,
            blob_gasprice: validator.blob_gasprice,
            tx_gas_limit_cap: validator.tx_gas_limit_cap,
            block_gas_limit: validator.block_gas_limit,
            max_blobs_per_tx: validator.max_blobs_per_tx,
            max_initcode_size: validator.max_initcode_size,
            validation_kind: if validator.checks == ValidationChecks::ALL {
                ValidationKind::ByTxType
            } else if validator.checks.is_empty() {
                ValidationKind::None
            } else {
                ValidationKind::Custom(validator.checks)
            },
        }
    }

    /// Create params for caller validation from Cfg trait.
    pub fn caller_params_from_cfg(cfg: &impl Cfg) -> Self {
        let mut checks = ValidationChecks::CALLER;

        if cfg.is_nonce_check_disabled() {
            checks.remove(ValidationChecks::NONCE);
        }
        if cfg.is_balance_check_disabled() {
            checks.remove(ValidationChecks::BALANCE);
        }
        if cfg.is_eip3607_disabled() {
            checks.remove(ValidationChecks::EIP3607);
        }

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

/// Validate block header fields.
pub fn validate_block_header(
    spec: SpecId,
    block: &impl Block,
    kind: ValidationKind,
) -> Result<(), InvalidHeader> {
    let validator = TxValidator {
        spec,
        checks: match kind {
            ValidationKind::None => ValidationChecks::empty(),
            ValidationKind::ByTxType => ValidationChecks::ALL,
            ValidationKind::Custom(c) => c,
        },
        ..Default::default()
    };
    validator.validate_header(block)
}

/// Validate transaction against parameters.
pub fn validate_tx(
    tx: &impl Transaction,
    params: &ValidationParams,
) -> Result<(), InvalidTransaction> {
    let validator = TxValidator {
        spec: params.spec,
        chain_id: params.chain_id,
        base_fee: params.base_fee,
        blob_gasprice: params.blob_gasprice,
        tx_gas_limit_cap: params.tx_gas_limit_cap,
        block_gas_limit: params.block_gas_limit,
        max_blobs_per_tx: params.max_blobs_per_tx,
        max_initcode_size: params.max_initcode_size,
        checks: match params.validation_kind {
            ValidationKind::None => ValidationChecks::empty(),
            ValidationKind::ByTxType => ValidationChecks::ALL,
            ValidationKind::Custom(c) => c,
        },
    };
    validator.validate_tx(tx)
}

/// Calculate initial and floor gas.
pub fn calculate_initial_gas(
    tx: &impl Transaction,
    spec: SpecId,
    skip_eip7623: bool,
) -> Result<InitialAndFloorGas, InvalidTransaction> {
    let mut validator = TxValidator::new(spec);
    if skip_eip7623 {
        validator = validator.skip_eip7623_check();
    }
    validator.initial_gas(tx)
}

/// Validate caller account state.
pub fn validate_caller(
    caller_info: &AccountInfo,
    tx_nonce: u64,
    kind: ValidationKind,
) -> Result<(), InvalidTransaction> {
    let validator = TxValidator {
        checks: match kind {
            ValidationKind::None => ValidationChecks::empty(),
            ValidationKind::ByTxType => ValidationChecks::ALL,
            ValidationKind::Custom(c) => c,
        },
        ..Default::default()
    };
    validator.validate_caller(caller_info, tx_nonce)
}

/// Calculate fee to deduct from caller.
pub fn calculate_caller_fee(
    caller_balance: U256,
    tx: &impl Transaction,
    block: &impl Block,
    skip_balance_check: bool,
) -> Result<CallerFee, InvalidTransaction> {
    let mut validator = TxValidator::default()
        .with_base_fee(block.basefee() as u128)
        .with_blob_gasprice(block.blob_gasprice().unwrap_or(0));

    if skip_balance_check {
        validator = validator.skip_balance_check();
    }

    validator.caller_fee(caller_balance, tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_checks_default_is_all() {
        // ValidationChecks::default() now returns ALL for safety
        let checks = ValidationChecks::default();
        assert_eq!(checks, ValidationChecks::ALL);
    }

    #[test]
    fn test_validation_checks_all() {
        let checks = ValidationChecks::ALL;
        assert!(checks.contains(ValidationChecks::CHAIN_ID));
        assert!(checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
        assert!(checks.contains(ValidationChecks::EIP3607));
        // Verify ALL = TX_STATELESS | CALLER
        assert_eq!(
            checks,
            ValidationChecks::TX_STATELESS | ValidationChecks::CALLER
        );
    }

    #[test]
    fn test_tx_validator_builder() {
        let validator = TxValidator::new(SpecId::CANCUN)
            .with_chain_id(42)
            .with_base_fee(1000)
            .skip_nonce_check()
            .skip_balance_check();

        assert_eq!(validator.spec, SpecId::CANCUN);
        assert_eq!(validator.chain_id, 42);
        assert_eq!(validator.base_fee, Some(1000));
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
    }

    #[test]
    fn test_tx_validator_skip_all_then_enable() {
        let validator = TxValidator::new(SpecId::CANCUN)
            .skip_all()
            .enable_chain_id_check()
            .enable_nonce_check();

        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(!validator.should_check(ValidationChecks::BASE_FEE));
    }

    #[test]
    fn test_validation_kind_should_check() {
        assert!(!ValidationKind::None.should_check(ValidationChecks::CHAIN_ID));
        assert!(ValidationKind::ByTxType.should_check(ValidationChecks::CHAIN_ID));

        let custom = ValidationKind::Custom(ValidationChecks::CHAIN_ID | ValidationChecks::NONCE);
        assert!(custom.should_check(ValidationChecks::CHAIN_ID));
        assert!(custom.should_check(ValidationChecks::NONCE));
        assert!(!custom.should_check(ValidationChecks::BALANCE));
    }

    #[test]
    fn test_preset_for_deposit() {
        let validator = TxValidator::for_deposit(SpecId::CANCUN);

        // Should have chain ID and block gas limit checks
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(validator.should_check(ValidationChecks::AUTH_LIST));
        assert!(validator.should_check(ValidationChecks::MAX_INITCODE_SIZE));

        // Should NOT have fee and state checks
        assert!(!validator.should_check(ValidationChecks::BASE_FEE));
        assert!(!validator.should_check(ValidationChecks::PRIORITY_FEE));
        assert!(!validator.should_check(ValidationChecks::BLOB_FEE));
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(!validator.should_check(ValidationChecks::EIP3607));
    }

    #[test]
    fn test_preset_for_tx_pool() {
        let validator = TxValidator::for_tx_pool(SpecId::CANCUN);

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
    fn test_preset_for_block_builder() {
        let validator = TxValidator::for_block_builder(SpecId::CANCUN);

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
    fn test_query_methods() {
        let full_validator = TxValidator::new(SpecId::CANCUN);
        assert!(full_validator.has_any_checks());
        assert!(full_validator.has_all_checks());
        assert_eq!(full_validator.enabled_checks(), ValidationChecks::ALL);

        let empty_validator = TxValidator::new(SpecId::CANCUN).skip_all();
        assert!(!empty_validator.has_any_checks());
        assert!(!empty_validator.has_all_checks());
        assert_eq!(empty_validator.enabled_checks(), ValidationChecks::empty());
    }

    #[test]
    fn test_composite_skip_enable() {
        // Test skip_caller_checks
        let validator = TxValidator::new(SpecId::CANCUN).skip_caller_checks();
        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(!validator.should_check(ValidationChecks::EIP3607));
        assert!(validator.should_check(ValidationChecks::BASE_FEE)); // other checks preserved

        // Test enable_caller_checks
        let validator = TxValidator::new(SpecId::CANCUN)
            .skip_all()
            .enable_caller_checks();
        assert!(validator.should_check(ValidationChecks::NONCE));
        assert!(validator.should_check(ValidationChecks::BALANCE));
        assert!(validator.should_check(ValidationChecks::EIP3607));

        // Test skip_gas_fee_checks
        let validator = TxValidator::new(SpecId::CANCUN).skip_gas_fee_checks();
        assert!(!validator.should_check(ValidationChecks::BASE_FEE));
        assert!(!validator.should_check(ValidationChecks::PRIORITY_FEE));
        assert!(!validator.should_check(ValidationChecks::BLOB_FEE));
        assert!(!validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(validator.should_check(ValidationChecks::NONCE)); // other checks preserved

        // Test enable_gas_fee_checks
        let validator = TxValidator::new(SpecId::CANCUN)
            .skip_all()
            .enable_gas_fee_checks();
        assert!(validator.should_check(ValidationChecks::BASE_FEE));
        assert!(validator.should_check(ValidationChecks::PRIORITY_FEE));
        assert!(validator.should_check(ValidationChecks::BLOCK_GAS_LIMIT));
    }

    #[test]
    fn test_tx_validator_is_copy() {
        let validator = TxValidator::new(SpecId::CANCUN);
        let copy = validator; // This works because TxValidator is Copy
        assert_eq!(validator.spec, copy.spec);
    }

    #[test]
    fn test_gas_fees_composite_flag() {
        let checks = ValidationChecks::GAS_FEES;
        assert!(checks.contains(ValidationChecks::TX_GAS_LIMIT));
        assert!(checks.contains(ValidationChecks::BASE_FEE));
        assert!(checks.contains(ValidationChecks::PRIORITY_FEE));
        assert!(checks.contains(ValidationChecks::BLOB_FEE));
        assert!(checks.contains(ValidationChecks::BLOCK_GAS_LIMIT));
        assert!(checks.contains(ValidationChecks::EIP7623));
    }

    #[test]
    fn test_with_spec() {
        let validator = TxValidator::new(SpecId::CANCUN).with_spec(SpecId::PRAGUE);
        assert_eq!(validator.spec, SpecId::PRAGUE);
    }

    #[test]
    fn test_with_disabled_checks() {
        let validator = TxValidator::new(SpecId::CANCUN)
            .with_disabled_checks(ValidationChecks::NONCE | ValidationChecks::BALANCE);

        assert!(!validator.should_check(ValidationChecks::NONCE));
        assert!(!validator.should_check(ValidationChecks::BALANCE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID)); // others preserved
    }

    #[test]
    fn test_with_enabled_checks() {
        let validator = TxValidator::new(SpecId::CANCUN)
            .skip_all()
            .with_enabled_checks(ValidationChecks::NONCE | ValidationChecks::CHAIN_ID);

        assert!(validator.should_check(ValidationChecks::NONCE));
        assert!(validator.should_check(ValidationChecks::CHAIN_ID));
        assert!(!validator.should_check(ValidationChecks::BALANCE)); // not enabled
    }

    #[test]
    fn test_tx_stateless_composite() {
        let checks = ValidationChecks::TX_STATELESS;
        assert!(checks.contains(ValidationChecks::CHAIN_ID));
        assert!(checks.contains(ValidationChecks::GAS_FEES));
        assert!(checks.contains(ValidationChecks::AUTH_LIST));
        assert!(checks.contains(ValidationChecks::MAX_INITCODE_SIZE));
        assert!(checks.contains(ValidationChecks::HEADER));
        // Should NOT contain caller checks
        assert!(!checks.contains(ValidationChecks::NONCE));
        assert!(!checks.contains(ValidationChecks::BALANCE));
        assert!(!checks.contains(ValidationChecks::EIP3607));
    }

    #[test]
    fn test_caller_composite() {
        let checks = ValidationChecks::CALLER;
        assert!(checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
        assert!(checks.contains(ValidationChecks::EIP3607));
        // Should NOT contain stateless checks
        assert!(!checks.contains(ValidationChecks::CHAIN_ID));
        assert!(!checks.contains(ValidationChecks::BASE_FEE));
    }

    #[test]
    fn test_enable_all_restores_all_checks() {
        let validator = TxValidator::new(SpecId::CANCUN).skip_all().enable_all();
        assert!(validator.has_all_checks());
        assert_eq!(validator.enabled_checks(), ValidationChecks::ALL);
    }

    #[test]
    fn test_validate_caller_with_empty_checks() {
        let validator = TxValidator::new(SpecId::CANCUN).skip_all();
        let caller_info = AccountInfo {
            nonce: 999,                            // wrong nonce
            code: Some(bytecode::Bytecode::new()), // has code
            ..Default::default()
        };
        // Should pass because all checks are skipped
        assert!(validator.validate_caller(&caller_info, 0).is_ok());
    }
}
