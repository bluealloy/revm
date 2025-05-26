use crate::transaction::TransactionError;
use core::fmt::{self, Debug};
use database_interface::DBErrorMarker;
use primitives::{Address, Bytes, Log, U256};
use state::EvmState;
use std::{boxed::Box, string::String, vec::Vec};

pub trait HaltReasonTr: Clone + Debug + PartialEq + Eq + From<HaltReason> {}

impl<T> HaltReasonTr for T where T: Clone + Debug + PartialEq + Eq + From<HaltReason> {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultAndState<HaltReasonTy = HaltReason> {
    /// Status of execution
    pub result: ExecutionResult<HaltReasonTy>,
    /// State that got updated
    pub state: EvmState,
}

impl<HaltReasonTy> ResultAndState<HaltReasonTy> {
    /// Maps a `DBError` to a new error type using the provided closure, leaving other variants unchanged.
    pub fn map_haltreason<F, OHR>(self, op: F) -> ResultAndState<OHR>
    where
        F: FnOnce(HaltReasonTy) -> OHR,
    {
        ResultAndState {
            result: self.result.map_haltreason(op),
            state: self.state,
        }
    }
}

/// Result of a transaction execution
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult<HaltReasonTy = HaltReason> {
    /// Returned successfully
    Success {
        reason: SuccessReason,
        gas_used: u64,
        gas_refunded: u64,
        logs: Vec<Log>,
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all gas
    Revert { gas_used: u64, output: Bytes },
    /// Reverted for various reasons and spend all gas
    Halt {
        reason: HaltReasonTy,
        /// Halting will spend all the gas, and will be equal to gas_limit.
        gas_used: u64,
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
                gas_used,
                gas_refunded,
                logs,
                output,
            } => ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            },
            Self::Revert { gas_used, output } => ExecutionResult::Revert { gas_used, output },
            Self::Halt { reason, gas_used } => ExecutionResult::Halt {
                reason: op(reason),
                gas_used,
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

    /// Returns the gas used.
    pub fn gas_used(&self) -> u64 {
        match *self {
            Self::Success { gas_used, .. }
            | Self::Revert { gas_used, .. }
            | Self::Halt { gas_used, .. } => gas_used,
        }
    }
}

/// Output of a transaction execution
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

impl<DBError: DBErrorMarker, TX> From<DBError> for EVMError<DBError, TX> {
    fn from(value: DBError) -> Self {
        Self::Database(value)
    }
}

pub trait FromStringError {
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
        initial_gas: u64,
        gas_limit: u64,
    },
    /// Gas floor calculated from EIP-7623 Increase calldata cost
    /// is more than the gas limit.
    ///
    /// Tx data is too large to be executed.
    GasFloorMoreThanGasLimit {
        gas_floor: u64,
        gas_limit: u64,
    },
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
    ///
    /// `to` must be present
    BlobCreateTransaction,
    /// Transaction has more then `max` blobs
    TooManyBlobs {
        max: usize,
        have: usize,
    },
    /// Blob transaction contains a versioned hash with an incorrect version
    BlobVersionNotSupported,
    /// EOF create should have `to` address
    EofCreateShouldHaveToAddress,
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
    // TODO (EOF)
    // /// EIP-7873 needs to have at least one initcode.
    // Eip7873EmptyInitcodeList,
    // /// EIP-7873 initcode can't be zero length.
    // Eip7873EmptyInitcode {
    //     i: usize,
    // },
    // /// EIP-7873 initcodes can't be more than [`MAX_INITCODE_COUNT`].
    // Eip7873TooManyInitcodes {
    //     size: usize,
    // },
    // /// EIP-7873 initcodes can't be more than [`MAX_INITCODE_SIZE`].
    // Eip7873InitcodeTooLarge {
    //     i: usize,
    //     size: usize,
    // },
    /// EIP-7873 initcode transaction should have `to` address.
    Eip7873MissingTarget,
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
            Self::EofCreateShouldHaveToAddress => write!(f, "EOF crate should have `to` address"),
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
            // TODO(EOF)
            // Self::Eip7873EmptyInitcodeList => {
            //     write!(f, "Eip7873 initcode list should have at least one initcode")
            // }
            // Self::Eip7873EmptyInitcode { i } => {
            //     write!(f, "Eip7873 initcode {i} can't be zero length")
            // }
            // Self::Eip7873TooManyInitcodes { size } => {
            //     write!(
            //         f,
            //         "Eip7873 initcodes can't be more than {MAX_INITCODE_COUNT}, have {size}"
            //     )
            // }
            // Self::Eip7873InitcodeTooLarge { i, size } => {
            //     write!(
            //         f,
            //         "Eip7873 initcode {i} can't be more than {MAX_INITCODE_SIZE}, have {size}"
            //     )
            // }
            Self::Eip7873MissingTarget => {
                write!(f, "Eip7873 initcode transaction should have `to` address")
            }
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
    Stop,
    Return,
    SelfDestruct,
    EofReturnContract,
}

/// Indicates that the EVM has experienced an exceptional halt.
///
/// This causes execution to immediately end with all gas being consumed.
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

    /// Aux data overflow, new aux data is larger than [u16] max size.
    EofAuxDataOverflow,
    /// Aux data is smaller than already present data size.
    EofAuxDataTooSmall,
    /// EOF Subroutine stack overflow
    SubRoutineStackOverflow,
    /// Check for target address validity is only done inside subcall.
    InvalidEXTCALLTarget,
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
    // When performing SSTORE the gasleft is less than or equal to 2300
    ReentrancySentry,
}
