use crate::{Log, State, B160};
use bytes::Bytes;

pub type EVMResult<DB> = std::result::Result<ResultAndState, EVMError<DB>>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultAndState {
    /// Status of execution
    pub result: ExecutionResult,
    /// State that got updated
    pub state: State,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult {
    /// Returned successfully
    Success {
        reason: Eval,
        gas_used: u64,
        gas_refunded: u64,
        logs: Vec<Log>,
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all gas.
    Revert { gas_used: u64 },
    /// Reverted for various reasons and spend all gas.
    Halt {
        reason: Halt,
        /// Halting will spend all the gas, and will be equal to gas_limit.
        gas_used: u64,
    },
}

impl ExecutionResult {
    /// Returns if transaction execution is successful.
    /// 1 indicates success, 0 indicates revert.
    /// https://eips.ethereum.org/EIPS/eip-658
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Return logs, if execution is not successful, function will return empty vec.
    pub fn logs(&self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs.clone(),
            _ => Vec::new(),
        }
    }

    pub fn gas_used(&self) -> u64 {
        let (Self::Success { gas_used, .. }
        | Self::Revert { gas_used }
        | Self::Halt { gas_used, .. }) = self;

        *gas_used
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Output {
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    Call(Bytes),
    Create(
        #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))] Bytes,
        Option<B160>,
    ),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EVMError<DB> {
    Transaction(InvalidTransaction),
    /// REVM specific and related to environment.
    PrevrandaoNotSet,
    Database(DB),
}

impl<DB> From<InvalidTransaction> for EVMError<DB> {
    fn from(invalid: InvalidTransaction) -> Self {
        EVMError::Transaction(invalid)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidTransaction {
    GasMaxFeeGreaterThanPriorityFee,
    GasPriceLessThenBasefee,
    CallerGasLimitMoreThenBlock,
    CallGasCostMoreThenGasLimit,
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    /// Transaction account does not have enough amount of ether to cover transferred value and gas_limit*gas_price.
    LackOfFundForGasLimit,
    /// Overflow payment in transaction.
    OverflowPaymentInTransaction,
    /// Nonce overflows in transaction,
    NonceOverflowInTransaction,
}

/// When transaction return successfully without halts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Eval {
    Stop,
    Return,
    SelfDestruct,
}

/// Indicates that the EVM has experienced an exceptional halt. This causes execution to
/// immediately end with all gas being consumed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Halt {
    OutOfGas,
    OpcodeNotFound,
    InvalidFEOpcode,
    InvalidJump,
    NotActivated,
    StackUnderflow,
    StackOverflow,
    OutOfOffset,
    CreateCollision,
    OverflowPayment,
    PrecompileError,
    NonceOverflow,
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit,
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF,
}
