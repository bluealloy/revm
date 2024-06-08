use crate::{Address, Bytes, EvmState, Log, U256};
use core::fmt;
use std::{boxed::Box, string::String, vec::Vec};

/// Result of EVM execution.
pub type EVMResult<DBError> = EVMResultGeneric<ResultAndState, DBError>;

/// Generic result of EVM execution. Used to represent error and generic output.
pub type EVMResultGeneric<T, DBError> = core::result::Result<T, EVMError<DBError>>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultAndState {
    /// Status of execution
    pub result: ExecutionResult,
    /// State that got updated
    pub state: EvmState,
}

/// Result of a transaction execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult {
    /// Returned successfully
    Success {
        reason: SuccessReason,
        gas_used: u64,
        gas_refunded: u64,
        logs: Vec<Log>,
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all gas.
    Revert { gas_used: u64, output: Bytes },
    /// Reverted for various reasons and spend all gas.
    Halt {
        reason: HaltReason,
        /// Halting will spend all the gas, and will be equal to gas_limit.
        gas_used: u64,
    },
}

impl ExecutionResult {
    /// Returns if transaction execution is successful.
    /// 1 indicates success, 0 indicates revert.
    /// <https://eips.ethereum.org/EIPS/eip-658>
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Returns true if execution result is a Halt.
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::Halt { .. })
    }

    /// Returns the output data of the execution.
    ///
    /// Returns `None` if the execution was halted.
    pub fn output(&self) -> Option<&Bytes> {
        match self {
            Self::Success { output, .. } => Some(output.data()),
            Self::Revert { output, .. } => Some(output),
            _ => None,
        }
    }

    /// Consumes the type and returns the output data of the execution.
    ///
    /// Returns `None` if the execution was halted.
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
            Self::Success { logs, .. } => logs,
            _ => &[],
        }
    }

    /// Consumes `self` and returns the logs if execution is successful, or an empty list otherwise.
    pub fn into_logs(self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs,
            _ => Vec::new(),
        }
    }

    /// Returns the gas used.
    pub fn gas_used(&self) -> u64 {
        match *self {
            Self::Success { gas_used, .. }
            | Self::Revert { gas_used, .. }
            | Self::Halt { gas_used, .. } => gas_used,
        }
    }
}

/// Output of a transaction execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Output {
    Call(Bytes),
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

/// Main EVM error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EVMError<DBError> {
    /// Transaction validation error.
    Transaction(InvalidTransaction),
    /// Header validation error.
    Header(InvalidHeader),
    /// Database error.
    Database(DBError),
    /// Custom error.
    ///
    /// Useful for handler registers where custom logic would want to return their own custom error.
    Custom(String),
    /// Precompile error.
    Precompile(String),
}

#[cfg(feature = "std")]
impl<DBError: std::error::Error + 'static> std::error::Error for EVMError<DBError> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transaction(e) => Some(e),
            Self::Header(e) => Some(e),
            Self::Database(e) => Some(e),
            Self::Precompile(_) | Self::Custom(_) => None,
        }
    }
}

impl<DBError: fmt::Display> fmt::Display for EVMError<DBError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transaction(e) => write!(f, "transaction validation error: {e}"),
            Self::Header(e) => write!(f, "header validation error: {e}"),
            Self::Database(e) => write!(f, "database error: {e}"),
            Self::Precompile(e) | Self::Custom(e) => f.write_str(e),
        }
    }
}

impl<DBError> From<InvalidTransaction> for EVMError<DBError> {
    fn from(value: InvalidTransaction) -> Self {
        Self::Transaction(value)
    }
}

impl<DBError> From<InvalidHeader> for EVMError<DBError> {
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
    CallGasCostMoreThanGasLimit,
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    /// Transaction account does not have enough amount of ether to cover transferred value and gas_limit*gas_price.
    LackOfFundForMaxFee {
        fee: Box<U256>,
        balance: Box<U256>,
    },
    /// Overflow payment in transaction.
    OverflowPaymentInTransaction,
    /// Nonce overflows in transaction.
    NonceOverflowInTransaction,
    NonceTooHigh {
        tx: u64,
        state: u64,
    },
    NonceTooLow {
        tx: u64,
        state: u64,
    },
    /// EIP-3860: Limit and meter initcode
    CreateInitCodeSizeLimit,
    /// Transaction chain id does not match the config chain id.
    InvalidChainId,
    /// Access list is not supported for blocks before the Berlin hardfork.
    AccessListNotSupported,
    /// `max_fee_per_blob_gas` is not supported for blocks before the Cancun hardfork.
    MaxFeePerBlobGasNotSupported,
    /// `blob_hashes`/`blob_versioned_hashes` is not supported for blocks before the Cancun hardfork.
    BlobVersionedHashesNotSupported,
    /// Block `blob_gas_price` is greater than tx-specified `max_fee_per_blob_gas` after Cancun.
    BlobGasPriceGreaterThanMax,
    /// There should be at least one blob in Blob transaction.
    EmptyBlobs,
    /// Blob transaction can't be a create transaction.
    /// `to` must be present
    BlobCreateTransaction,
    /// Transaction has more then [`crate::MAX_BLOB_NUMBER_PER_BLOCK`] blobs
    TooManyBlobs {
        max: usize,
        have: usize,
    },
    /// Blob transaction contains a versioned hash with an incorrect version
    BlobVersionNotSupported,
    /// EOF crate should have `to` address
    EofCrateShouldHaveToAddress,
    /// System transactions are not supported post-regolith hardfork.
    ///
    /// Before the Regolith hardfork, there was a special field in the `Deposit` transaction
    /// type that differentiated between `system` and `user` deposit transactions. This field
    /// was deprecated in the Regolith hardfork, and this error is thrown if a `Deposit` transaction
    /// is found with this field set to `true` after the hardfork activation.
    ///
    /// In addition, this error is internal, and bubbles up into a [HaltReason::FailedDeposit] error
    /// in the `revm` handler for the consumer to easily handle. This is due to a state transition
    /// rule on OP Stack chains where, if for any reason a deposit transaction fails, the transaction
    /// must still be included in the block, the sender nonce is bumped, the `mint` value persists, and
    /// special gas accounting rules are applied. Normally on L1, [EVMError::Transaction] errors
    /// are cause for non-inclusion, so a special [HaltReason] variant was introduced to handle this
    /// case for failed deposit transactions.
    #[cfg(feature = "optimism")]
    DepositSystemTxPostRegolith,
    /// Deposit transaction haults bubble up to the global main return handler, wiping state and
    /// only increasing the nonce + persisting the mint value.
    ///
    /// This is a catch-all error for any deposit transaction that is results in a [HaltReason] error
    /// post-regolith hardfork. This allows for a consumer to easily handle special cases where
    /// a deposit transaction fails during validation, but must still be included in the block.
    ///
    /// In addition, this error is internal, and bubbles up into a [HaltReason::FailedDeposit] error
    /// in the `revm` handler for the consumer to easily handle. This is due to a state transition
    /// rule on OP Stack chains where, if for any reason a deposit transaction fails, the transaction
    /// must still be included in the block, the sender nonce is bumped, the `mint` value persists, and
    /// special gas accounting rules are applied. Normally on L1, [EVMError::Transaction] errors
    /// are cause for non-inclusion, so a special [HaltReason] variant was introduced to handle this
    /// case for failed deposit transactions.
    #[cfg(feature = "optimism")]
    HaltedDepositPostRegolith,
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidTransaction {}

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
            Self::CallGasCostMoreThanGasLimit => {
                write!(f, "call gas cost exceeds the gas limit")
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
            Self::AccessListNotSupported => write!(f, "access list not supported"),
            Self::MaxFeePerBlobGasNotSupported => {
                write!(f, "max fee per blob gas not supported")
            }
            Self::BlobVersionedHashesNotSupported => {
                write!(f, "blob versioned hashes not supported")
            }
            Self::BlobGasPriceGreaterThanMax => {
                write!(f, "blob gas price is greater than max fee per blob gas")
            }
            Self::EmptyBlobs => write!(f, "empty blobs"),
            Self::BlobCreateTransaction => write!(f, "blob create transaction"),
            Self::TooManyBlobs { max, have } => {
                write!(f, "too many blobs, have {have}, max {max}")
            }
            Self::BlobVersionNotSupported => write!(f, "blob version not supported"),
            Self::EofCrateShouldHaveToAddress => write!(f, "EOF crate should have `to` address"),
            #[cfg(feature = "optimism")]
            Self::DepositSystemTxPostRegolith => {
                write!(
                    f,
                    "deposit system transactions post regolith hardfork are not supported"
                )
            }
            #[cfg(feature = "optimism")]
            Self::HaltedDepositPostRegolith => {
                write!(
                    f,
                    "deposit transaction halted post-regolith; error will be bubbled up to main return handler"
                )
            }
        }
    }
}

/// Errors related to misconfiguration of a [`crate::env::BlockEnv`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidHeader {
    /// `prevrandao` is not set for Merge and above.
    PrevrandaoNotSet,
    /// `excess_blob_gas` is not set for Cancun and above.
    ExcessBlobGasNotSet,
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidHeader {}

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
    Stop,
    Return,
    SelfDestruct,
    EofReturnContract,
}

/// Indicates that the EVM has experienced an exceptional halt. This causes execution to
/// immediately end with all gas being consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HaltReason {
    OutOfGas(OutOfGasError),
    OpcodeNotFound,
    InvalidFEOpcode,
    InvalidJump,
    NotActivated,
    StackUnderflow,
    StackOverflow,
    OutOfOffset,
    CreateCollision,
    PrecompileError,
    NonceOverflow,
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit,
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF,
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitCodeSizeLimit,

    /* Internal Halts that can be only found inside Inspector */
    OverflowPayment,
    StateChangeDuringStaticCall,
    CallNotAllowedInsideStatic,
    OutOfFunds,
    CallTooDeep,

    /* Optimism errors */
    #[cfg(feature = "optimism")]
    FailedDeposit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OutOfGasError {
    // Basic OOG error
    Basic,
    // Tried to expand past REVM limit
    MemoryLimit,
    // Basic OOG error from memory expansion
    Memory,
    // Precompile threw OOG error
    Precompile,
    // When performing something that takes a U256 and casts down to a u64, if its too large this would fire
    // i.e. in `as_usize_or_fail`
    InvalidOperand,
}
