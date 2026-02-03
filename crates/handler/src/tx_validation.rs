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
//!     .enable_gas_checks();
//! ```

use bitflags::bitflags;
use context_interface::{
    result::{InvalidHeader, InvalidTransaction},
    transaction::{Transaction, TransactionType},
    Block, Cfg,
};
use core::cmp::{self, Ordering};
use interpreter::{instructions::calculate_initial_tx_gas_for_tx, InitialAndFloorGas};
use primitives::{eip4844, hardfork::SpecId, B256, U256};
use state::AccountInfo;

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

        /// All stateless transaction checks (no account state needed).
        const TX_STATELESS = Self::CHAIN_ID.bits()
            | Self::TX_GAS_LIMIT.bits()
            | Self::BASE_FEE.bits()
            | Self::PRIORITY_FEE.bits()
            | Self::BLOB_FEE.bits()
            | Self::AUTH_LIST.bits()
            | Self::BLOCK_GAS_LIMIT.bits()
            | Self::MAX_INITCODE_SIZE.bits()
            | Self::EIP7623.bits()
            | Self::HEADER.bits();

        /// All caller/state checks.
        const CALLER = Self::NONCE.bits() | Self::BALANCE.bits() | Self::EIP3607.bits();

        /// All validation checks enabled.
        const ALL = Self::TX_STATELESS.bits() | Self::CALLER.bits();
    }
}

/// Transaction validator with configurable checks.
///
/// This struct provides a fluent API for validating Ethereum transactions.
/// It can be configured to skip certain checks (e.g., for L2 deposit transactions)
/// or to validate only specific aspects of a transaction.
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
#[derive(Debug, Clone)]
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
    pub fn new(spec: SpecId) -> Self {
        Self {
            spec,
            ..Default::default()
        }
    }

    /// Create validator from Cfg and Block traits.
    ///
    /// This is the recommended way to create a validator when you have
    /// access to the EVM context.
    pub fn from_cfg_and_block(cfg: &impl Cfg, block: &impl Block) -> Self {
        let mut validator = Self {
            spec: cfg.spec().into(),
            chain_id: cfg.chain_id(),
            base_fee: Some(block.basefee() as u128),
            blob_gasprice: block.blob_gasprice(),
            tx_gas_limit_cap: cfg.tx_gas_limit_cap(),
            block_gas_limit: block.gas_limit(),
            max_blobs_per_tx: cfg.max_blobs_per_tx(),
            max_initcode_size: cfg.max_initcode_size(),
            checks: ValidationChecks::ALL,
        };

        // Apply cfg skip flags
        if !cfg.tx_chain_id_check() {
            validator.checks.remove(ValidationChecks::CHAIN_ID);
        }
        if cfg.is_base_fee_check_disabled() {
            validator.checks.remove(ValidationChecks::BASE_FEE);
            validator.base_fee = None;
        }
        if cfg.is_priority_fee_check_disabled() {
            validator.checks.remove(ValidationChecks::PRIORITY_FEE);
        }
        if cfg.is_block_gas_limit_disabled() {
            validator.checks.remove(ValidationChecks::BLOCK_GAS_LIMIT);
        }
        if cfg.is_nonce_check_disabled() {
            validator.checks.remove(ValidationChecks::NONCE);
        }
        if cfg.is_balance_check_disabled() {
            validator.checks.remove(ValidationChecks::BALANCE);
        }
        if cfg.is_eip3607_disabled() {
            validator.checks.remove(ValidationChecks::EIP3607);
        }
        if cfg.is_eip7623_disabled() {
            validator.checks.remove(ValidationChecks::EIP7623);
        }

        validator
    }

    // === Builder methods for configuration ===

    /// Set the chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Set the base fee.
    pub fn with_base_fee(mut self, base_fee: u128) -> Self {
        self.base_fee = Some(base_fee);
        self
    }

    /// Set the blob gas price.
    pub fn with_blob_gasprice(mut self, price: u128) -> Self {
        self.blob_gasprice = Some(price);
        self
    }

    /// Set the block gas limit.
    pub fn with_block_gas_limit(mut self, limit: u64) -> Self {
        self.block_gas_limit = limit;
        self
    }

    /// Set the transaction gas limit cap.
    pub fn with_tx_gas_limit_cap(mut self, cap: u64) -> Self {
        self.tx_gas_limit_cap = cap;
        self
    }

    /// Set maximum blobs per transaction.
    pub fn with_max_blobs(mut self, max: u64) -> Self {
        self.max_blobs_per_tx = Some(max);
        self
    }

    /// Set maximum initcode size.
    pub fn with_max_initcode_size(mut self, size: usize) -> Self {
        self.max_initcode_size = size;
        self
    }

    // === Skip methods ===

    /// Skip all validation checks.
    pub fn skip_all(mut self) -> Self {
        self.checks = ValidationChecks::empty();
        self
    }

    /// Skip chain ID validation.
    pub fn skip_chain_id_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::CHAIN_ID);
        self
    }

    /// Skip base fee validation.
    pub fn skip_base_fee_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BASE_FEE);
        self
    }

    /// Skip priority fee validation.
    pub fn skip_priority_fee_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::PRIORITY_FEE);
        self
    }

    /// Skip nonce validation.
    pub fn skip_nonce_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::NONCE);
        self
    }

    /// Skip balance validation.
    pub fn skip_balance_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BALANCE);
        self
    }

    /// Skip EIP-3607 code check.
    pub fn skip_eip3607_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::EIP3607);
        self
    }

    /// Skip block gas limit check.
    pub fn skip_block_gas_limit_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::BLOCK_GAS_LIMIT);
        self
    }

    /// Skip EIP-7623 floor gas check.
    pub fn skip_eip7623_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::EIP7623);
        self
    }

    /// Skip header validation.
    pub fn skip_header_check(mut self) -> Self {
        self.checks.remove(ValidationChecks::HEADER);
        self
    }

    // === Enable methods (for use after skip_all) ===

    /// Enable chain ID validation.
    pub fn enable_chain_id_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::CHAIN_ID);
        self
    }

    /// Enable all gas-related checks.
    pub fn enable_gas_checks(mut self) -> Self {
        self.checks.insert(ValidationChecks::TX_GAS_LIMIT);
        self.checks.insert(ValidationChecks::BASE_FEE);
        self.checks.insert(ValidationChecks::PRIORITY_FEE);
        self.checks.insert(ValidationChecks::BLOCK_GAS_LIMIT);
        self
    }

    /// Enable nonce validation.
    pub fn enable_nonce_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::NONCE);
        self
    }

    /// Enable balance validation.
    pub fn enable_balance_check(mut self) -> Self {
        self.checks.insert(ValidationChecks::BALANCE);
        self
    }

    /// Set custom validation checks.
    pub fn with_checks(mut self, checks: ValidationChecks) -> Self {
        self.checks = checks;
        self
    }

    // === Validation methods ===

    /// Check if a specific validation is enabled.
    #[inline]
    pub fn should_check(&self, check: ValidationChecks) -> bool {
        self.checks.contains(check)
    }

    /// Validate block header fields.
    ///
    /// Checks that required header fields are present for the current spec:
    /// - `prevrandao` is required after the Merge
    /// - `excess_blob_gas` is required after Cancun
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
        if self.should_check(ValidationChecks::TX_GAS_LIMIT) && tx.gas_limit() > self.tx_gas_limit_cap
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
    pub fn initial_gas(&self, tx: &impl Transaction) -> Result<InitialAndFloorGas, InvalidTransaction> {
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
            let bytecode = match caller_info.code.as_ref() {
                Some(code) => code,
                None => &bytecode::Bytecode::default(),
            };

            if !bytecode.is_empty() && !bytecode.is_eip7702() {
                return Err(InvalidTransaction::RejectCallerWithCode);
            }
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
                if self.should_check(ValidationChecks::AUTH_LIST) && tx.authorization_list_len() == 0
                {
                    return Err(InvalidTransaction::EmptyAuthorizationList);
                }
            }
            TransactionType::Custom => {}
        }
        Ok(())
    }

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

/// Result of fee calculation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallerFee {
    /// The gas fee amount that was deducted.
    pub gas_fee_to_deduct: U256,
    /// The new balance after deducting gas fees.
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
    fn test_validation_checks_default() {
        let checks = ValidationChecks::default();
        assert!(checks.is_empty());
    }

    #[test]
    fn test_validation_checks_all() {
        let checks = ValidationChecks::ALL;
        assert!(checks.contains(ValidationChecks::CHAIN_ID));
        assert!(checks.contains(ValidationChecks::NONCE));
        assert!(checks.contains(ValidationChecks::BALANCE));
        assert!(checks.contains(ValidationChecks::EIP3607));
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
}
