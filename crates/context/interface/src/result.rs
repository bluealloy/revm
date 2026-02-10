//! Result of the EVM execution. Containing both execution result, state and errors.
//!
//! [`ExecutionResult`] is the result of the EVM execution.
//!
//! [`InvalidTransaction`] is the error that is returned when the transaction is invalid.
//!
//! [`InvalidHeader`] is the error that is returned when the header is invalid.
//!
//! [`SuccessReason`] is the reason that the transaction successfully completed.
use crate::{context::ContextError, transaction::TransactionError};
use core::fmt::{self, Debug};
use database_interface::DBErrorMarker;
use primitives::{Address, Bytes, Log, U256};
use state::EvmState;
use std::{borrow::Cow, boxed::Box, string::String, vec::Vec};

/// Trait for the halt reason.
pub trait HaltReasonTr: Clone + Debug + PartialEq + Eq + From<HaltReason> {}

impl<T> HaltReasonTr for T where T: Clone + Debug + PartialEq + Eq + From<HaltReason> {}

/// Tuple containing evm execution result and state.s
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecResultAndState<R, S = EvmState> {
    /// Execution result
    pub result: R,
    /// Output State.
    pub state: S,
}

/// Type alias for backwards compatibility.
pub type ResultAndState<H = HaltReason, S = EvmState> = ExecResultAndState<ExecutionResult<H>, S>;

/// Tuple containing multiple execution results and state.
pub type ResultVecAndState<R, S> = ExecResultAndState<Vec<R>, S>;

impl<R, S> ExecResultAndState<R, S> {
    /// Creates new ResultAndState.
    pub fn new(result: R, state: S) -> Self {
        Self { result, state }
    }
}

/// Gas accounting result from transaction execution.
///
/// Self-contained gas snapshot with all values needed for downstream consumers:
///
/// | Field       | Source                          | Description                               |
/// |-------------|---------------------------------|-------------------------------------------|
/// | `limit`     | `Gas::limit()`                  | Transaction gas limit                     |
/// | `spent`     | `Gas::spent()` = limit − remaining | Total gas consumed before refund       |
/// | `refunded`  | `Gas::refunded()` as u64        | Gas refunded (capped per EIP-3529)        |
/// | `floor_gas` | `InitialAndFloorGas::floor_gas` | EIP-7623 floor gas (0 if not applicable)  |
/// | `intrinsic_gas` | `InitialAndFloorGas::initial_gas` | Initial tx overhead gas (0 for system calls) |
///
/// Derived values:
/// - [`used()`](ResultGas::used) = `spent − refunded` (the value that goes into receipts)
/// - [`remaining()`](ResultGas::remaining) = `limit − spent`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultGas {
    /// Transaction gas limit.
    limit: u64,
    /// Gas consumed before final refund (limit − remaining).
    /// For actual gas used, use [`used()`](ResultGas::used).
    #[cfg_attr(feature = "serde", serde(rename = "gas_spent"))]
    spent: u64,
    /// Gas refund amount (capped per EIP-3529).
    ///
    /// Note: It does not apply EIP-7623 floor gas check for the refund amount.
    /// If tx
    ///
    #[cfg_attr(feature = "serde", serde(rename = "gas_refunded"))]
    refunded: u64,
    /// EIP-7623 floor gas. Zero when not applicable.
    floor_gas: u64,
    /// Intrinsic gas: the initial transaction overhead (calldata, access list, etc.).
    /// Zero for system calls.
    intrinsic_gas: u64,
}

impl ResultGas {
    /// Creates a new `ResultGas`.
    #[inline]
    pub const fn new(
        limit: u64,
        spent: u64,
        refunded: u64,
        floor_gas: u64,
        intrinsic_gas: u64,
    ) -> Self {
        Self {
            limit,
            spent,
            refunded,
            floor_gas,
            intrinsic_gas,
        }
    }

    /// Returns the transaction gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the gas spent (consumed before refund).
    #[inline]
    pub const fn spent(&self) -> u64 {
        self.spent
    }

    /// Returns the gas refunded (capped per EIP-3529).
    #[inline]
    pub const fn refunded(&self) -> u64 {
        self.refunded
    }

    /// Returns the EIP-7623 floor gas.
    #[inline]
    pub const fn floor_gas(&self) -> u64 {
        self.floor_gas
    }

    /// Returns the intrinsic gas.
    #[inline]
    pub const fn intrinsic_gas(&self) -> u64 {
        self.intrinsic_gas
    }

    /// Sets the `limit` field.
    #[inline]
    pub const fn with_limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }

    /// Sets the `spent` field.
    #[inline]
    pub const fn with_spent(mut self, spent: u64) -> Self {
        self.spent = spent;
        self
    }

    /// Sets the `refunded` field.
    #[inline]
    pub const fn with_refunded(mut self, refunded: u64) -> Self {
        self.refunded = refunded;
        self
    }

    /// Sets the `floor_gas` field.
    #[inline]
    pub const fn with_floor_gas(mut self, floor_gas: u64) -> Self {
        self.floor_gas = floor_gas;
        self
    }

    /// Sets the `intrinsic_gas` field.
    #[inline]
    pub const fn with_intrinsic_gas(mut self, intrinsic_gas: u64) -> Self {
        self.intrinsic_gas = intrinsic_gas;
        self
    }

    /// Sets the `limit` field by mutable reference.
    #[inline]
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Sets the `spent` field by mutable reference.
    #[inline]
    pub fn set_spent(&mut self, spent: u64) {
        self.spent = spent;
    }

    /// Sets the `refunded` field by mutable reference.
    #[inline]
    pub fn set_refunded(&mut self, refunded: u64) {
        self.refunded = refunded;
    }

    /// Sets the `floor_gas` field by mutable reference.
    #[inline]
    pub fn set_floor_gas(&mut self, floor_gas: u64) {
        self.floor_gas = floor_gas;
    }

    /// Sets the `intrinsic_gas` field by mutable reference.
    #[inline]
    pub fn set_intrinsic_gas(&mut self, intrinsic_gas: u64) {
        self.intrinsic_gas = intrinsic_gas;
    }

    /// Returns the final gas used: `spent - refunded`.
    ///
    /// This is the value used for receipt `cumulative_gas_used` accumulation
    /// and the per-transaction gas charge.
    #[inline]
    pub const fn used(&self) -> u64 {
        // EIP-7623: Increase calldata cost
        // spend at least a gas_floor amount of gas.
        let spent_sub_refunded = self.spent_sub_refunded();
        if spent_sub_refunded < self.floor_gas {
            return self.floor_gas;
        }
        spent_sub_refunded
    }

    /// Returns the gas spent minus the refunded gas.
    #[inline]
    pub const fn spent_sub_refunded(&self) -> u64 {
        self.spent.saturating_sub(self.refunded)
    }

    /// Returns the remaining gas: `limit - spent`.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.spent)
    }

    /// Returns the final gas used, same as [`used()`](ResultGas::used).
    ///
    /// This is `max(spent - refunded, floor_gas)` — the value that goes into receipts.
    #[inline]
    pub const fn final_used(&self) -> u64 {
        self.used()
    }

    /// Returns the raw refund from EVM execution, before EIP-7623 floor gas adjustment.
    ///
    /// This is the `refunded` field value (capped per EIP-3529 but not adjusted for floor gas).
    /// See [`final_refunded()`](ResultGas::final_refunded) for the effective refund.
    #[inline]
    pub const fn inner_refunded(&self) -> u64 {
        self.refunded
    }

    /// Returns the effective refund after EIP-7623 floor gas adjustment: `spent - used()`.
    ///
    /// When `floor_gas` kicks in, this may be less than [`inner_refunded()`](ResultGas::inner_refunded).
    /// Always satisfies: `spent == final_used() + final_refunded()`.
    #[inline]
    pub const fn final_refunded(&self) -> u64 {
        self.spent.saturating_sub(self.used())
    }
}

impl fmt::Display for ResultGas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gas used: {}, limit: {}, spent: {}",
            self.used(),
            self.limit,
            self.spent
        )?;
        if self.refunded > 0 {
            write!(f, ", refunded: {}", self.refunded)?;
        }
        if self.floor_gas > 0 {
            write!(f, ", floor: {}", self.floor_gas)?;
        }
        if self.intrinsic_gas > 0 {
            write!(f, ", intrinsic: {}", self.intrinsic_gas)?;
        }
        Ok(())
    }
}

/// Result of a transaction execution
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult<HaltReasonTy = HaltReason> {
    /// Returned successfully
    Success {
        /// Reason for the success.
        reason: SuccessReason,
        /// Gas accounting for the transaction.
        gas: ResultGas,
        /// Logs emitted by the transaction.
        logs: Vec<Log>,
        /// Output of the transaction.
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all gas
    Revert {
        /// Gas accounting for the transaction.
        gas: ResultGas,
        /// Output of the transaction.
        output: Bytes,
    },
    /// Reverted for various reasons and spend all gas
    Halt {
        /// Reason for the halt.
        reason: HaltReasonTy,
        /// Gas accounting for the transaction.
        ///
        /// Halting will spend all the gas, and will be equal to gas_limit.
        gas: ResultGas,
    },
}

impl<HaltReasonTy> ExecutionResult<HaltReasonTy> {
    /// Returns if transaction execution is successful.
    ///
    /// 1 indicates success, 0 indicates revert.
    ///
    /// <https://eips.ethereum.org/EIPS/eip-658>
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Maps a `DBError` to a new error type using the provided closure, leaving other variants unchanged.
    pub fn map_haltreason<F, OHR>(self, op: F) -> ExecutionResult<OHR>
    where
        F: FnOnce(HaltReasonTy) -> OHR,
    {
        match self {
            Self::Success {
                reason,
                gas,
                logs,
                output,
            } => ExecutionResult::Success {
                reason,
                gas,
                logs,
                output,
            },
            Self::Revert { gas, output } => ExecutionResult::Revert { gas, output },
            Self::Halt { reason, gas } => ExecutionResult::Halt {
                reason: op(reason),
                gas,
            },
        }
    }

    /// Returns created address if execution is Create transaction
    /// and Contract was created.
    pub fn created_address(&self) -> Option<Address> {
        match self {
            Self::Success { output, .. } => output.address().cloned(),
            _ => None,
        }
    }

    /// Returns true if execution result is a Halt.
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::Halt { .. })
    }

    /// Returns the output data of the execution.
    ///
    /// Returns [`None`] if the execution was halted.
    pub fn output(&self) -> Option<&Bytes> {
        match self {
            Self::Success { output, .. } => Some(output.data()),
            Self::Revert { output, .. } => Some(output),
            _ => None,
        }
    }

    /// Consumes the type and returns the output data of the execution.
    ///
    /// Returns [`None`] if the execution was halted.
    pub fn into_output(self) -> Option<Bytes> {
        match self {
            Self::Success { output, .. } => Some(output.into_data()),
            Self::Revert { output, .. } => Some(output),
            _ => None,
        }
    }

    /// Returns the logs if execution is successful, or an empty list otherwise.
    pub fn logs(&self) -> &[Log] {
        match self {
            Self::Success { logs, .. } => logs.as_slice(),
            _ => &[],
        }
    }

    /// Consumes [`self`] and returns the logs if execution is successful, or an empty list otherwise.
    pub fn into_logs(self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs,
            _ => Vec::new(),
        }
    }

    /// Returns the gas accounting information.
    pub fn gas(&self) -> &ResultGas {
        match self {
            Self::Success { gas, .. } | Self::Revert { gas, .. } | Self::Halt { gas, .. } => gas,
        }
    }

    /// Returns the gas used.
    pub fn gas_used(&self) -> u64 {
        self.gas().used()
    }
}

impl<HaltReasonTy: fmt::Display> fmt::Display for ExecutionResult<HaltReasonTy> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success {
                reason,
                gas,
                logs,
                output,
            } => {
                write!(f, "Success ({reason}): {gas}")?;
                if !logs.is_empty() {
                    write!(
                        f,
                        ", {} log{}",
                        logs.len(),
                        if logs.len() == 1 { "" } else { "s" }
                    )?;
                }
                write!(f, ", {output}")
            }
            Self::Revert { gas, output } => {
                write!(f, "Revert: {gas}")?;
                if !output.is_empty() {
                    write!(f, ", {} bytes output", output.len())?;
                }
                Ok(())
            }
            Self::Halt { reason, gas } => {
                write!(f, "Halted: {reason} ({gas})")
            }
        }
    }
}

/// Output of a transaction execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Output {
    /// Output of a call.
    Call(Bytes),
    /// Output of a create.
    Create(Bytes, Option<Address>),
}

impl Output {
    /// Returns the output data of the execution output.
    pub fn into_data(self) -> Bytes {
        match self {
            Output::Call(data) => data,
            Output::Create(data, _) => data,
        }
    }

    /// Returns the output data of the execution output.
    pub fn data(&self) -> &Bytes {
        match self {
            Output::Call(data) => data,
            Output::Create(data, _) => data,
        }
    }

    /// Returns the created address, if any.
    pub fn address(&self) -> Option<&Address> {
        match self {
            Output::Call(_) => None,
            Output::Create(_, address) => address.as_ref(),
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Output::Call(data) => {
                if data.is_empty() {
                    write!(f, "no output")
                } else {
                    write!(f, "{} bytes output", data.len())
                }
            }
            Output::Create(data, Some(addr)) => {
                if data.is_empty() {
                    write!(f, "contract created at {}", addr)
                } else {
                    write!(f, "contract created at {} ({} bytes)", addr, data.len())
                }
            }
            Output::Create(data, None) => {
                if data.is_empty() {
                    write!(f, "contract creation (no address)")
                } else {
                    write!(f, "contract creation (no address, {} bytes)", data.len())
                }
            }
        }
    }
}

/// Main EVM error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EVMError<DBError, TransactionError = InvalidTransaction> {
    /// Transaction validation error
    Transaction(TransactionError),
    /// Header validation error
    Header(InvalidHeader),
    /// Database error
    Database(DBError),
    /// Custom error
    ///
    /// Useful for handler registers where custom logic would want to return their own custom error.
    Custom(String),
}

impl<DBError, TransactionValidationErrorT> From<ContextError<DBError>>
    for EVMError<DBError, TransactionValidationErrorT>
{
    fn from(value: ContextError<DBError>) -> Self {
        match value {
            ContextError::Db(e) => Self::Database(e),
            ContextError::Custom(e) => Self::Custom(e),
        }
    }
}

impl<DBError: DBErrorMarker, TX> From<DBError> for EVMError<DBError, TX> {
    fn from(value: DBError) -> Self {
        Self::Database(value)
    }
}

/// Trait for converting a string to an [`EVMError::Custom`] error.
pub trait FromStringError {
    /// Converts a string to an [`EVMError::Custom`] error.
    fn from_string(value: String) -> Self;
}

impl<DB, TX> FromStringError for EVMError<DB, TX> {
    fn from_string(value: String) -> Self {
        Self::Custom(value)
    }
}

impl<DB, TXE: From<InvalidTransaction>> From<InvalidTransaction> for EVMError<DB, TXE> {
    fn from(value: InvalidTransaction) -> Self {
        Self::Transaction(TXE::from(value))
    }
}

impl<DBError, TransactionValidationErrorT> EVMError<DBError, TransactionValidationErrorT> {
    /// Maps a `DBError` to a new error type using the provided closure, leaving other variants unchanged.
    pub fn map_db_err<F, E>(self, op: F) -> EVMError<E, TransactionValidationErrorT>
    where
        F: FnOnce(DBError) -> E,
    {
        match self {
            Self::Transaction(e) => EVMError::Transaction(e),
            Self::Header(e) => EVMError::Header(e),
            Self::Database(e) => EVMError::Database(op(e)),
            Self::Custom(e) => EVMError::Custom(e),
        }
    }
}

impl<DBError, TransactionValidationErrorT> core::error::Error
    for EVMError<DBError, TransactionValidationErrorT>
where
    DBError: core::error::Error + 'static,
    TransactionValidationErrorT: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Transaction(e) => Some(e),
            Self::Header(e) => Some(e),
            Self::Database(e) => Some(e),
            Self::Custom(_) => None,
        }
    }
}

impl<DBError, TransactionValidationErrorT> fmt::Display
    for EVMError<DBError, TransactionValidationErrorT>
where
    DBError: fmt::Display,
    TransactionValidationErrorT: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transaction(e) => write!(f, "transaction validation error: {e}"),
            Self::Header(e) => write!(f, "header validation error: {e}"),
            Self::Database(e) => write!(f, "database error: {e}"),
            Self::Custom(e) => f.write_str(e),
        }
    }
}

impl<DBError, TransactionValidationErrorT> From<InvalidHeader>
    for EVMError<DBError, TransactionValidationErrorT>
{
    fn from(value: InvalidHeader) -> Self {
        Self::Header(value)
    }
}

/// Transaction validation error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidTransaction {
    /// When using the EIP-1559 fee model introduced in the London upgrade, transactions specify two primary fee fields:
    /// - `gas_max_fee`: The maximum total fee a user is willing to pay, inclusive of both base fee and priority fee.
    /// - `gas_priority_fee`: The extra amount a user is willing to give directly to the miner, often referred to as the "tip".
    ///
    /// Provided `gas_priority_fee` exceeds the total `gas_max_fee`.
    PriorityFeeGreaterThanMaxFee,
    /// EIP-1559: `gas_price` is less than `basefee`.
    GasPriceLessThanBasefee,
    /// `gas_limit` in the tx is bigger than `block_gas_limit`.
    CallerGasLimitMoreThanBlock,
    /// Initial gas for a Call is bigger than `gas_limit`.
    ///
    /// Initial gas for a Call contains:
    /// - initial stipend gas
    /// - gas for access list and input data
    CallGasCostMoreThanGasLimit {
        /// Initial gas for a Call.
        initial_gas: u64,
        /// Gas limit for the transaction.
        gas_limit: u64,
    },
    /// Gas floor calculated from EIP-7623 Increase calldata cost
    /// is more than the gas limit.
    ///
    /// Tx data is too large to be executed.
    GasFloorMoreThanGasLimit {
        /// Gas floor calculated from EIP-7623 Increase calldata cost.
        gas_floor: u64,
        /// Gas limit for the transaction.
        gas_limit: u64,
    },
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    /// Transaction account does not have enough amount of ether to cover transferred value and gas_limit*gas_price.
    LackOfFundForMaxFee {
        /// Fee for the transaction.
        fee: Box<U256>,
        /// Balance of the sender.
        balance: Box<U256>,
    },
    /// Overflow payment in transaction.
    OverflowPaymentInTransaction,
    /// Nonce overflows in transaction.
    NonceOverflowInTransaction,
    /// Nonce is too high.
    NonceTooHigh {
        /// Nonce of the transaction.
        tx: u64,
        /// Nonce of the state.
        state: u64,
    },
    /// Nonce is too low.
    NonceTooLow {
        /// Nonce of the transaction.
        tx: u64,
        /// Nonce of the state.
        state: u64,
    },
    /// EIP-3860: Limit and meter initcode
    CreateInitCodeSizeLimit,
    /// Transaction chain id does not match the config chain id.
    InvalidChainId,
    /// Missing chain id.
    MissingChainId,
    /// Transaction gas limit is greater than the cap.
    TxGasLimitGreaterThanCap {
        /// Transaction gas limit.
        gas_limit: u64,
        /// Gas limit cap.
        cap: u64,
    },
    /// Access list is not supported for blocks before the Berlin hardfork.
    AccessListNotSupported,
    /// `max_fee_per_blob_gas` is not supported for blocks before the Cancun hardfork.
    MaxFeePerBlobGasNotSupported,
    /// `blob_hashes`/`blob_versioned_hashes` is not supported for blocks before the Cancun hardfork.
    BlobVersionedHashesNotSupported,
    /// Block `blob_gas_price` is greater than tx-specified `max_fee_per_blob_gas` after Cancun.
    BlobGasPriceGreaterThanMax {
        /// Block `blob_gas_price`.
        block_blob_gas_price: u128,
        /// Tx-specified `max_fee_per_blob_gas`.
        tx_max_fee_per_blob_gas: u128,
    },
    /// There should be at least one blob in Blob transaction.
    EmptyBlobs,
    /// Blob transaction can't be a create transaction.
    ///
    /// `to` must be present
    BlobCreateTransaction,
    /// Transaction has more then `max` blobs
    TooManyBlobs {
        /// Maximum number of blobs allowed.
        max: usize,
        /// Number of blobs in the transaction.
        have: usize,
    },
    /// Blob transaction contains a versioned hash with an incorrect version
    BlobVersionNotSupported,
    /// EIP-7702 is not enabled.
    AuthorizationListNotSupported,
    /// EIP-7702 transaction has invalid fields set.
    AuthorizationListInvalidFields,
    /// Empty Authorization List is not allowed.
    EmptyAuthorizationList,
    /// EIP-2930 is not supported.
    Eip2930NotSupported,
    /// EIP-1559 is not supported.
    Eip1559NotSupported,
    /// EIP-4844 is not supported.
    Eip4844NotSupported,
    /// EIP-7702 is not supported.
    Eip7702NotSupported,
    /// EIP-7873 is not supported.
    Eip7873NotSupported,
    /// EIP-7873 initcode transaction should have `to` address.
    Eip7873MissingTarget,
    /// Custom string error for flexible error handling.
    Str(Cow<'static, str>),
}

impl TransactionError for InvalidTransaction {}

impl core::error::Error for InvalidTransaction {}

impl fmt::Display for InvalidTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PriorityFeeGreaterThanMaxFee => {
                write!(f, "priority fee is greater than max fee")
            }
            Self::GasPriceLessThanBasefee => {
                write!(f, "gas price is less than basefee")
            }
            Self::CallerGasLimitMoreThanBlock => {
                write!(f, "caller gas limit exceeds the block gas limit")
            }
            Self::TxGasLimitGreaterThanCap { gas_limit, cap } => {
                write!(
                    f,
                    "transaction gas limit ({gas_limit}) is greater than the cap ({cap})"
                )
            }
            Self::CallGasCostMoreThanGasLimit {
                initial_gas,
                gas_limit,
            } => {
                write!(
                    f,
                    "call gas cost ({initial_gas}) exceeds the gas limit ({gas_limit})"
                )
            }
            Self::GasFloorMoreThanGasLimit {
                gas_floor,
                gas_limit,
            } => {
                write!(
                    f,
                    "gas floor ({gas_floor}) exceeds the gas limit ({gas_limit})"
                )
            }
            Self::RejectCallerWithCode => {
                write!(f, "reject transactions from senders with deployed code")
            }
            Self::LackOfFundForMaxFee { fee, balance } => {
                write!(f, "lack of funds ({balance}) for max fee ({fee})")
            }
            Self::OverflowPaymentInTransaction => {
                write!(f, "overflow payment in transaction")
            }
            Self::NonceOverflowInTransaction => {
                write!(f, "nonce overflow in transaction")
            }
            Self::NonceTooHigh { tx, state } => {
                write!(f, "nonce {tx} too high, expected {state}")
            }
            Self::NonceTooLow { tx, state } => {
                write!(f, "nonce {tx} too low, expected {state}")
            }
            Self::CreateInitCodeSizeLimit => {
                write!(f, "create initcode size limit")
            }
            Self::InvalidChainId => write!(f, "invalid chain ID"),
            Self::MissingChainId => write!(f, "missing chain ID"),
            Self::AccessListNotSupported => write!(f, "access list not supported"),
            Self::MaxFeePerBlobGasNotSupported => {
                write!(f, "max fee per blob gas not supported")
            }
            Self::BlobVersionedHashesNotSupported => {
                write!(f, "blob versioned hashes not supported")
            }
            Self::BlobGasPriceGreaterThanMax {
                block_blob_gas_price,
                tx_max_fee_per_blob_gas,
            } => {
                write!(
                    f,
                    "blob gas price ({block_blob_gas_price}) is greater than max fee per blob gas ({tx_max_fee_per_blob_gas})"
                )
            }
            Self::EmptyBlobs => write!(f, "empty blobs"),
            Self::BlobCreateTransaction => write!(f, "blob create transaction"),
            Self::TooManyBlobs { max, have } => {
                write!(f, "too many blobs, have {have}, max {max}")
            }
            Self::BlobVersionNotSupported => write!(f, "blob version not supported"),
            Self::AuthorizationListNotSupported => write!(f, "authorization list not supported"),
            Self::AuthorizationListInvalidFields => {
                write!(f, "authorization list tx has invalid fields")
            }
            Self::EmptyAuthorizationList => write!(f, "empty authorization list"),
            Self::Eip2930NotSupported => write!(f, "Eip2930 is not supported"),
            Self::Eip1559NotSupported => write!(f, "Eip1559 is not supported"),
            Self::Eip4844NotSupported => write!(f, "Eip4844 is not supported"),
            Self::Eip7702NotSupported => write!(f, "Eip7702 is not supported"),
            Self::Eip7873NotSupported => write!(f, "Eip7873 is not supported"),
            Self::Eip7873MissingTarget => {
                write!(f, "Eip7873 initcode transaction should have `to` address")
            }
            Self::Str(msg) => f.write_str(msg),
        }
    }
}

/// Errors related to misconfiguration of a [`crate::Block`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidHeader {
    /// `prevrandao` is not set for Merge and above.
    PrevrandaoNotSet,
    /// `excess_blob_gas` is not set for Cancun and above.
    ExcessBlobGasNotSet,
}

impl core::error::Error for InvalidHeader {}

impl fmt::Display for InvalidHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrevrandaoNotSet => write!(f, "`prevrandao` not set"),
            Self::ExcessBlobGasNotSet => write!(f, "`excess_blob_gas` not set"),
        }
    }
}

/// Reason a transaction successfully completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuccessReason {
    /// Stop [`state::bytecode::opcode::STOP`] opcode.
    Stop,
    /// Return [`state::bytecode::opcode::RETURN`] opcode.
    Return,
    /// Self destruct opcode.
    SelfDestruct,
}

impl fmt::Display for SuccessReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stop => write!(f, "Stop"),
            Self::Return => write!(f, "Return"),
            Self::SelfDestruct => write!(f, "SelfDestruct"),
        }
    }
}

/// Indicates that the EVM has experienced an exceptional halt.
///
/// This causes execution to immediately end with all gas being consumed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HaltReason {
    /// Out of gas error.
    OutOfGas(OutOfGasError),
    /// Opcode not found error.
    OpcodeNotFound,
    /// Invalid FE opcode error.
    InvalidFEOpcode,
    /// Invalid jump destination.
    InvalidJump,
    /// The feature or opcode is not activated in hardfork.
    NotActivated,
    /// Attempting to pop a value from an empty stack.
    StackUnderflow,
    /// Attempting to push a value onto a full stack.
    StackOverflow,
    /// Invalid memory or storage offset for [`state::bytecode::opcode::RETURNDATACOPY`].
    OutOfOffset,
    /// Address collision during contract creation.
    CreateCollision,
    /// Precompile error.
    PrecompileError,
    /// Precompile error with message from context.
    PrecompileErrorWithContext(String),
    /// Nonce overflow.
    NonceOverflow,
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit,
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF,
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitCodeSizeLimit,

    /* Internal Halts that can be only found inside Inspector */
    /// Overflow payment. Not possible to happen on mainnet.
    OverflowPayment,
    /// State change during static call.
    StateChangeDuringStaticCall,
    /// Call not allowed inside static call.
    CallNotAllowedInsideStatic,
    /// Out of funds to pay for the call.
    OutOfFunds,
    /// Call is too deep.
    CallTooDeep,
}

impl core::error::Error for HaltReason {}

impl fmt::Display for HaltReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfGas(err) => write!(f, "{err}"),
            Self::OpcodeNotFound => write!(f, "opcode not found"),
            Self::InvalidFEOpcode => write!(f, "invalid 0xFE opcode"),
            Self::InvalidJump => write!(f, "invalid jump destination"),
            Self::NotActivated => write!(f, "feature or opcode not activated"),
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::OutOfOffset => write!(f, "out of offset"),
            Self::CreateCollision => write!(f, "create collision"),
            Self::PrecompileError => write!(f, "precompile error"),
            Self::PrecompileErrorWithContext(msg) => write!(f, "precompile error: {msg}"),
            Self::NonceOverflow => write!(f, "nonce overflow"),
            Self::CreateContractSizeLimit => write!(f, "create contract size limit"),
            Self::CreateContractStartingWithEF => {
                write!(f, "create contract starting with 0xEF")
            }
            Self::CreateInitCodeSizeLimit => write!(f, "create initcode size limit"),
            Self::OverflowPayment => write!(f, "overflow payment"),
            Self::StateChangeDuringStaticCall => write!(f, "state change during static call"),
            Self::CallNotAllowedInsideStatic => write!(f, "call not allowed inside static call"),
            Self::OutOfFunds => write!(f, "out of funds"),
            Self::CallTooDeep => write!(f, "call too deep"),
        }
    }
}

/// Out of gas errors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OutOfGasError {
    /// Basic OOG error. Not enough gas to execute the opcode.
    Basic,
    /// Tried to expand past memory limit.
    MemoryLimit,
    /// Basic OOG error from memory expansion
    Memory,
    /// Precompile threw OOG error
    Precompile,
    /// When performing something that takes a U256 and casts down to a u64, if its too large this would fire
    /// i.e. in `as_usize_or_fail`
    InvalidOperand,
    /// When performing SSTORE the gasleft is less than or equal to 2300
    ReentrancySentry,
}

impl core::error::Error for OutOfGasError {}

impl fmt::Display for OutOfGasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic => write!(f, "out of gas"),
            Self::MemoryLimit => write!(f, "out of gas: memory limit exceeded"),
            Self::Memory => write!(f, "out of gas: memory expansion"),
            Self::Precompile => write!(f, "out of gas: precompile"),
            Self::InvalidOperand => write!(f, "out of gas: invalid operand"),
            Self::ReentrancySentry => write!(f, "out of gas: reentrancy sentry"),
        }
    }
}

/// Error that includes transaction index for batch transaction processing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionIndexedError<Error> {
    /// The original error that occurred.
    pub error: Error,
    /// The index of the transaction that failed.
    pub transaction_index: usize,
}

impl<Error> TransactionIndexedError<Error> {
    /// Create a new `TransactionIndexedError` with the given error and transaction index.
    #[must_use]
    pub fn new(error: Error, transaction_index: usize) -> Self {
        Self {
            error,
            transaction_index,
        }
    }

    /// Get a reference to the underlying error.
    pub fn error(&self) -> &Error {
        &self.error
    }

    /// Convert into the underlying error.
    #[must_use]
    pub fn into_error(self) -> Error {
        self.error
    }
}

impl<Error: fmt::Display> fmt::Display for TransactionIndexedError<Error> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transaction {} failed: {}",
            self.transaction_index, self.error
        )
    }
}

impl<Error: core::error::Error + 'static> core::error::Error for TransactionIndexedError<Error> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<&'static str> for InvalidTransaction {
    fn from(s: &'static str) -> Self {
        Self::Str(Cow::Borrowed(s))
    }
}

impl From<String> for InvalidTransaction {
    fn from(s: String) -> Self {
        Self::Str(Cow::Owned(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_display() {
        let result: ExecutionResult<HaltReason> = ExecutionResult::Success {
            reason: SuccessReason::Return,
            gas: ResultGas::new(100000, 26000, 5000, 0, 0),
            logs: vec![Log::default(), Log::default()],
            output: Output::Call(Bytes::from(vec![1, 2, 3])),
        };
        assert_eq!(
            result.to_string(),
            "Success (Return): gas used: 21000, limit: 100000, spent: 26000, refunded: 5000, 2 logs, 3 bytes output"
        );

        let result: ExecutionResult<HaltReason> = ExecutionResult::Revert {
            gas: ResultGas::new(100000, 100000, 0, 0, 0),
            output: Bytes::from(vec![1, 2, 3, 4]),
        };
        assert_eq!(
            result.to_string(),
            "Revert: gas used: 100000, limit: 100000, spent: 100000, 4 bytes output"
        );

        let result: ExecutionResult<HaltReason> = ExecutionResult::Halt {
            reason: HaltReason::OutOfGas(OutOfGasError::Basic),
            gas: ResultGas::new(1000000, 1000000, 0, 0, 0),
        };
        assert_eq!(
            result.to_string(),
            "Halted: out of gas (gas used: 1000000, limit: 1000000, spent: 1000000)"
        );
    }

    #[test]
    fn test_result_gas_display() {
        // No refund, no floor
        assert_eq!(
            ResultGas::new(100000, 21000, 0, 0, 0).to_string(),
            "gas used: 21000, limit: 100000, spent: 21000"
        );
        // With refund
        assert_eq!(
            ResultGas::new(100000, 50000, 10000, 0, 0).to_string(),
            "gas used: 40000, limit: 100000, spent: 50000, refunded: 10000"
        );
        // With refund and floor
        assert_eq!(
            ResultGas::new(100000, 50000, 10000, 30000, 0).to_string(),
            "gas used: 40000, limit: 100000, spent: 50000, refunded: 10000, floor: 30000"
        );
    }

    #[test]
    fn test_result_gas_used_and_remaining() {
        let gas = ResultGas::new(200, 100, 30, 0, 0);
        assert_eq!(gas.limit(), 200);
        assert_eq!(gas.spent(), 100);
        assert_eq!(gas.refunded(), 30);
        assert_eq!(gas.used(), 70);
        assert_eq!(gas.remaining(), 100);

        // Saturating: refunded > spent
        let gas = ResultGas::new(100, 10, 50, 0, 0);
        assert_eq!(gas.used(), 0);
        assert_eq!(gas.remaining(), 90);
    }
}
