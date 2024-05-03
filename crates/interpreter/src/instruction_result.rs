use revm_primitives::ChainSpec;

use crate::primitives::{HaltReason, OutOfGasError, SuccessReason};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstructionResult {
    // success codes
    #[default]
    Continue = 0x00,
    Stop,
    Return,
    SelfDestruct,
    ReturnContract,

    // revert codes
    Revert = 0x10, // revert opcode
    CallTooDeep,
    OutOfFunds,

    // Actions
    CallOrCreate = 0x20,

    // error codes
    OutOfGas = 0x50,
    MemoryOOG,
    MemoryLimitOOG,
    PrecompileOOG,
    InvalidOperandOOG,
    OpcodeNotFound,
    CallNotAllowedInsideStatic,
    StateChangeDuringStaticCall,
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
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitCodeSizeLimit,
    /// Fatal external error. Returned by database.
    FatalExternalError,
    /// RETURNCONTRACT called in not init eof code.
    ReturnContractInNotInitEOF,
    /// Legacy contract is calling opcode that is enabled only in EOF.
    EOFOpcodeDisabledInLegacy,
    /// EOF function stack overflow
    EOFFunctionStackOverflow,
}

impl From<SuccessReason> for InstructionResult {
    fn from(value: SuccessReason) -> Self {
        match value {
            SuccessReason::Return => InstructionResult::Return,
            SuccessReason::Stop => InstructionResult::Stop,
            SuccessReason::SelfDestruct => InstructionResult::SelfDestruct,
        }
    }
}

impl From<HaltReason> for InstructionResult {
    fn from(value: HaltReason) -> Self {
        match value {
            HaltReason::OutOfGas(error) => match error {
                OutOfGasError::Basic => Self::OutOfGas,
                OutOfGasError::InvalidOperand => Self::InvalidOperandOOG,
                OutOfGasError::Memory => Self::MemoryOOG,
                OutOfGasError::MemoryLimit => Self::MemoryLimitOOG,
                OutOfGasError::Precompile => Self::PrecompileOOG,
            },
            HaltReason::OpcodeNotFound => Self::OpcodeNotFound,
            HaltReason::InvalidFEOpcode => Self::InvalidFEOpcode,
            HaltReason::InvalidJump => Self::InvalidJump,
            HaltReason::NotActivated => Self::NotActivated,
            HaltReason::StackOverflow => Self::StackOverflow,
            HaltReason::StackUnderflow => Self::StackUnderflow,
            HaltReason::OutOfOffset => Self::OutOfOffset,
            HaltReason::CreateCollision => Self::CreateCollision,
            HaltReason::PrecompileError => Self::PrecompileError,
            HaltReason::NonceOverflow => Self::NonceOverflow,
            HaltReason::CreateContractSizeLimit => Self::CreateContractSizeLimit,
            HaltReason::CreateContractStartingWithEF => Self::CreateContractStartingWithEF,
            HaltReason::CreateInitCodeSizeLimit => Self::CreateInitCodeSizeLimit,
            HaltReason::OverflowPayment => Self::OverflowPayment,
            HaltReason::StateChangeDuringStaticCall => Self::StateChangeDuringStaticCall,
            HaltReason::CallNotAllowedInsideStatic => Self::CallNotAllowedInsideStatic,
            HaltReason::OutOfFunds => Self::OutOfFunds,
            HaltReason::CallTooDeep => Self::CallTooDeep,
        }
    }
}

#[macro_export]
macro_rules! return_ok {
    () => {
        InstructionResult::Continue
            | InstructionResult::Stop
            | InstructionResult::Return
            | InstructionResult::SelfDestruct
            | InstructionResult::ReturnContract
    };
}

#[macro_export]
macro_rules! return_revert {
    () => {
        InstructionResult::Revert | InstructionResult::CallTooDeep | InstructionResult::OutOfFunds
    };
}

#[macro_export]
macro_rules! return_error {
    () => {
        InstructionResult::OutOfGas
            | InstructionResult::MemoryOOG
            | InstructionResult::MemoryLimitOOG
            | InstructionResult::PrecompileOOG
            | InstructionResult::InvalidOperandOOG
            | InstructionResult::OpcodeNotFound
            | InstructionResult::CallNotAllowedInsideStatic
            | InstructionResult::StateChangeDuringStaticCall
            | InstructionResult::InvalidFEOpcode
            | InstructionResult::InvalidJump
            | InstructionResult::NotActivated
            | InstructionResult::StackUnderflow
            | InstructionResult::StackOverflow
            | InstructionResult::OutOfOffset
            | InstructionResult::CreateCollision
            | InstructionResult::OverflowPayment
            | InstructionResult::PrecompileError
            | InstructionResult::NonceOverflow
            | InstructionResult::CreateContractSizeLimit
            | InstructionResult::CreateContractStartingWithEF
            | InstructionResult::CreateInitCodeSizeLimit
            | InstructionResult::FatalExternalError
            | InstructionResult::ReturnContractInNotInitEOF
            | InstructionResult::EOFOpcodeDisabledInLegacy
            | InstructionResult::EOFFunctionStackOverflow
    };
}

impl InstructionResult {
    /// Returns whether the result is a success.
    #[inline]
    pub const fn is_ok(self) -> bool {
        matches!(self, crate::return_ok!())
    }

    /// Returns whether the result is a revert.
    #[inline]
    pub const fn is_revert(self) -> bool {
        matches!(self, crate::return_revert!())
    }

    /// Returns whether the result is an error.
    #[inline]
    pub const fn is_error(self) -> bool {
        matches!(self, return_error!())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SuccessOrHalt<ChainSpecT: ChainSpec> {
    Success(SuccessReason),
    Revert,
    Halt(ChainSpecT::HaltReason),
    FatalExternalError,
    /// Internal instruction that signals Interpreter should continue running.
    InternalContinue,
    /// Internal instruction that signals call or create.
    InternalCallOrCreate,
}

impl<ChainSpecT: ChainSpec> SuccessOrHalt<ChainSpecT> {
    /// Returns true if the transaction returned successfully without halts.
    #[inline]
    pub fn is_success(self) -> bool {
        matches!(self, SuccessOrHalt::Success(_))
    }

    /// Returns the [SuccessReason] value if this a successful result
    #[inline]
    pub fn to_success(self) -> Option<SuccessReason> {
        match self {
            SuccessOrHalt::Success(reason) => Some(reason),
            _ => None,
        }
    }

    /// Returns true if the transaction reverted.
    #[inline]
    pub fn is_revert(self) -> bool {
        matches!(self, SuccessOrHalt::Revert)
    }

    /// Returns true if the EVM has experienced an exceptional halt
    #[inline]
    pub fn is_halt(self) -> bool {
        matches!(self, SuccessOrHalt::Halt(_))
    }

    /// Returns the [HaltReason] value the EVM has experienced an exceptional halt
    #[inline]
    pub fn to_halt(self) -> Option<ChainSpecT::HaltReason> {
        match self {
            SuccessOrHalt::Halt(reason) => Some(reason),
            _ => None,
        }
    }
}

impl<ChainSpecT: ChainSpec> From<InstructionResult> for SuccessOrHalt<ChainSpecT> {
    fn from(result: InstructionResult) -> Self {
        match result {
            InstructionResult::Continue => Self::InternalContinue, // used only in interpreter loop
            InstructionResult::Stop => Self::Success(SuccessReason::Stop),
            InstructionResult::Return => Self::Success(SuccessReason::Return),
            InstructionResult::SelfDestruct => Self::Success(SuccessReason::SelfDestruct),
            InstructionResult::Revert => Self::Revert,
            InstructionResult::CallOrCreate => Self::InternalCallOrCreate, // used only in interpreter loop
            InstructionResult::CallTooDeep => Self::Halt(HaltReason::CallTooDeep.into()), // not gonna happen for first call
            InstructionResult::OutOfFunds => Self::Halt(HaltReason::OutOfFunds.into()), // Check for first call is done separately.
            InstructionResult::OutOfGas => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::Basic).into())
            }
            InstructionResult::MemoryLimitOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::MemoryLimit).into())
            }
            InstructionResult::MemoryOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::Memory).into())
            }
            InstructionResult::PrecompileOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::Precompile).into())
            }
            InstructionResult::InvalidOperandOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::InvalidOperand).into())
            }
            InstructionResult::OpcodeNotFound | InstructionResult::ReturnContractInNotInitEOF => {
                Self::Halt(HaltReason::OpcodeNotFound.into())
            }
            InstructionResult::CallNotAllowedInsideStatic => {
                Self::Halt(HaltReason::CallNotAllowedInsideStatic.into())
            } // first call is not static call
            InstructionResult::StateChangeDuringStaticCall => {
                Self::Halt(HaltReason::StateChangeDuringStaticCall.into())
            }
            InstructionResult::InvalidFEOpcode => Self::Halt(HaltReason::InvalidFEOpcode.into()),
            InstructionResult::InvalidJump => Self::Halt(HaltReason::InvalidJump.into()),
            InstructionResult::NotActivated => Self::Halt(HaltReason::NotActivated.into()),
            InstructionResult::StackUnderflow => Self::Halt(HaltReason::StackUnderflow.into()),
            InstructionResult::StackOverflow => Self::Halt(HaltReason::StackOverflow.into()),
            InstructionResult::OutOfOffset => Self::Halt(HaltReason::OutOfOffset.into()),
            InstructionResult::CreateCollision => Self::Halt(HaltReason::CreateCollision.into()),
            InstructionResult::OverflowPayment => Self::Halt(HaltReason::OverflowPayment.into()), // Check for first call is done separately.
            InstructionResult::PrecompileError => Self::Halt(HaltReason::PrecompileError.into()),
            InstructionResult::NonceOverflow => Self::Halt(HaltReason::NonceOverflow.into()),
            InstructionResult::CreateContractSizeLimit
            | InstructionResult::CreateContractStartingWithEF => {
                Self::Halt(HaltReason::CreateContractSizeLimit.into())
            }
            InstructionResult::CreateInitCodeSizeLimit => {
                Self::Halt(HaltReason::CreateInitCodeSizeLimit.into())
            }
            InstructionResult::FatalExternalError => Self::FatalExternalError,
            InstructionResult::EOFOpcodeDisabledInLegacy => {
                Self::Halt(HaltReason::OpcodeNotFound.into())
            }
            InstructionResult::EOFFunctionStackOverflow => Self::FatalExternalError,
            InstructionResult::ReturnContract => {
                panic!("Unexpected EOF internal Return Contract")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InstructionResult;

    #[test]
    fn all_results_are_covered() {
        match InstructionResult::Continue {
            return_error!() => {}
            return_revert!() => {}
            return_ok!() => {}
            InstructionResult::CallOrCreate => {}
        }
    }

    #[test]
    fn test_results() {
        let ok_results = vec![
            InstructionResult::Continue,
            InstructionResult::Stop,
            InstructionResult::Return,
            InstructionResult::SelfDestruct,
        ];

        for result in ok_results {
            assert!(result.is_ok());
            assert!(!result.is_revert());
            assert!(!result.is_error());
        }

        let revert_results = vec![
            InstructionResult::Revert,
            InstructionResult::CallTooDeep,
            InstructionResult::OutOfFunds,
        ];

        for result in revert_results {
            assert!(!result.is_ok());
            assert!(result.is_revert());
            assert!(!result.is_error());
        }

        let error_results = vec![
            InstructionResult::OutOfGas,
            InstructionResult::MemoryOOG,
            InstructionResult::MemoryLimitOOG,
            InstructionResult::PrecompileOOG,
            InstructionResult::InvalidOperandOOG,
            InstructionResult::OpcodeNotFound,
            InstructionResult::CallNotAllowedInsideStatic,
            InstructionResult::StateChangeDuringStaticCall,
            InstructionResult::InvalidFEOpcode,
            InstructionResult::InvalidJump,
            InstructionResult::NotActivated,
            InstructionResult::StackUnderflow,
            InstructionResult::StackOverflow,
            InstructionResult::OutOfOffset,
            InstructionResult::CreateCollision,
            InstructionResult::OverflowPayment,
            InstructionResult::PrecompileError,
            InstructionResult::NonceOverflow,
            InstructionResult::CreateContractSizeLimit,
            InstructionResult::CreateContractStartingWithEF,
            InstructionResult::CreateInitCodeSizeLimit,
            InstructionResult::FatalExternalError,
        ];

        for result in error_results {
            assert!(!result.is_ok());
            assert!(!result.is_revert());
            assert!(result.is_error());
        }
    }
}
