use crate::primitives::{Eval, Halt, OutOfGasError};

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

    // revert codes
    Revert = 0x10, // revert opcode
    CallTooDeep,
    OutOfFund,

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
}

impl From<Eval> for InstructionResult {
    fn from(value: Eval) -> Self {
        match value {
            Eval::Return => InstructionResult::Return,
            Eval::Stop => InstructionResult::Stop,
            Eval::SelfDestruct => InstructionResult::SelfDestruct,
        }
    }
}

impl From<Halt> for InstructionResult {
    fn from(value: Halt) -> Self {
        match value {
            Halt::OutOfGas(OutOfGasError::BasicOutOfGas) => Self::OutOfGas,
            Halt::OutOfGas(OutOfGasError::InvalidOperand) => Self::InvalidOperandOOG,
            Halt::OutOfGas(OutOfGasError::Memory) => Self::MemoryOOG,
            Halt::OutOfGas(OutOfGasError::MemoryLimit) => Self::MemoryLimitOOG,
            Halt::OutOfGas(OutOfGasError::Precompile) => Self::PrecompileOOG,
            Halt::OpcodeNotFound => Self::OpcodeNotFound,
            Halt::InvalidFEOpcode => Self::InvalidFEOpcode,
            Halt::InvalidJump => Self::InvalidJump,
            Halt::NotActivated => Self::NotActivated,
            Halt::StackOverflow => Self::StackOverflow,
            Halt::StackUnderflow => Self::StackUnderflow,
            Halt::OutOfOffset => Self::OutOfOffset,
            Halt::CreateCollision => Self::CreateCollision,
            Halt::PrecompileError => Self::PrecompileError,
            Halt::NonceOverflow => Self::NonceOverflow,
            Halt::CreateContractSizeLimit => Self::CreateContractSizeLimit,
            Halt::CreateContractStartingWithEF => Self::CreateContractStartingWithEF,
            Halt::CreateInitCodeSizeLimit => Self::CreateInitCodeSizeLimit,
            Halt::OverflowPayment => Self::OverflowPayment,
            Halt::StateChangeDuringStaticCall => Self::StateChangeDuringStaticCall,
            Halt::CallNotAllowedInsideStatic => Self::CallNotAllowedInsideStatic,
            Halt::OutOfFund => Self::OutOfFund,
            Halt::CallTooDeep => Self::CallTooDeep,
            #[cfg(feature = "optimism")]
            Halt::FailedDeposit => Self::FatalExternalError,
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
    };
}

#[macro_export]
macro_rules! return_revert {
    () => {
        InstructionResult::Revert | InstructionResult::CallTooDeep | InstructionResult::OutOfFund
    };
}

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
    };
}

impl InstructionResult {
    /// Returns whether the result is a success.
    #[inline]
    pub fn is_ok(self) -> bool {
        matches!(self, crate::return_ok!())
    }

    /// Returns whether the result is a revert.
    #[inline]
    pub fn is_revert(self) -> bool {
        matches!(self, crate::return_revert!())
    }

    /// Returns whether the result is an error.
    #[inline]
    pub fn is_error(self) -> bool {
        matches!(self, return_error!())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SuccessOrHalt {
    Success(Eval),
    Revert,
    Halt(Halt),
    FatalExternalError,
    /// Internal instruction that signals Interpreter should continue running.
    InternalContinue,
    /// Internal instruction that signals subcall.
    InternalCallOrCreate,
}

impl SuccessOrHalt {
    /// Returns true if the transaction returned successfully without halts.
    #[inline]
    pub fn is_success(self) -> bool {
        matches!(self, SuccessOrHalt::Success(_))
    }

    /// Returns the [Eval] value if this a successful result
    #[inline]
    pub fn to_success(self) -> Option<Eval> {
        match self {
            SuccessOrHalt::Success(eval) => Some(eval),
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

    /// Returns the [Halt] value the EVM has experienced an exceptional halt
    #[inline]
    pub fn to_halt(self) -> Option<Halt> {
        match self {
            SuccessOrHalt::Halt(halt) => Some(halt),
            _ => None,
        }
    }
}

impl From<InstructionResult> for SuccessOrHalt {
    fn from(result: InstructionResult) -> Self {
        match result {
            InstructionResult::Continue => Self::InternalContinue, // used only in interpreter loop
            InstructionResult::Stop => Self::Success(Eval::Stop),
            InstructionResult::Return => Self::Success(Eval::Return),
            InstructionResult::SelfDestruct => Self::Success(Eval::SelfDestruct),
            InstructionResult::Revert => Self::Revert,
            InstructionResult::CallOrCreate => Self::InternalCallOrCreate, // used only in interpreter loop
            InstructionResult::CallTooDeep => Self::Halt(Halt::CallTooDeep), // not gonna happen for first call
            InstructionResult::OutOfFund => Self::Halt(Halt::OutOfFund), // Check for first call is done separately.
            InstructionResult::OutOfGas => Self::Halt(Halt::OutOfGas(
                revm_primitives::OutOfGasError::BasicOutOfGas,
            )),
            InstructionResult::MemoryLimitOOG => {
                Self::Halt(Halt::OutOfGas(revm_primitives::OutOfGasError::MemoryLimit))
            }
            InstructionResult::MemoryOOG => {
                Self::Halt(Halt::OutOfGas(revm_primitives::OutOfGasError::Memory))
            }
            InstructionResult::PrecompileOOG => {
                Self::Halt(Halt::OutOfGas(revm_primitives::OutOfGasError::Precompile))
            }
            InstructionResult::InvalidOperandOOG => Self::Halt(Halt::OutOfGas(
                revm_primitives::OutOfGasError::InvalidOperand,
            )),
            InstructionResult::OpcodeNotFound => Self::Halt(Halt::OpcodeNotFound),
            InstructionResult::CallNotAllowedInsideStatic => {
                Self::Halt(Halt::CallNotAllowedInsideStatic)
            } // first call is not static call
            InstructionResult::StateChangeDuringStaticCall => {
                Self::Halt(Halt::StateChangeDuringStaticCall)
            }
            InstructionResult::InvalidFEOpcode => Self::Halt(Halt::InvalidFEOpcode),
            InstructionResult::InvalidJump => Self::Halt(Halt::InvalidJump),
            InstructionResult::NotActivated => Self::Halt(Halt::NotActivated),
            InstructionResult::StackUnderflow => Self::Halt(Halt::StackUnderflow),
            InstructionResult::StackOverflow => Self::Halt(Halt::StackOverflow),
            InstructionResult::OutOfOffset => Self::Halt(Halt::OutOfOffset),
            InstructionResult::CreateCollision => Self::Halt(Halt::CreateCollision),
            InstructionResult::OverflowPayment => Self::Halt(Halt::OverflowPayment), // Check for first call is done separately.
            InstructionResult::PrecompileError => Self::Halt(Halt::PrecompileError),
            InstructionResult::NonceOverflow => Self::Halt(Halt::NonceOverflow),
            InstructionResult::CreateContractSizeLimit => Self::Halt(Halt::CreateContractSizeLimit),
            InstructionResult::CreateContractStartingWithEF => {
                Self::Halt(Halt::CreateContractSizeLimit)
            }
            InstructionResult::CreateInitCodeSizeLimit => Self::Halt(Halt::CreateInitCodeSizeLimit),
            InstructionResult::FatalExternalError => Self::FatalExternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InstructionResult;

    #[test]
    fn all_results_are_covered() {
        let result = InstructionResult::Continue;
        match result {
            return_error!() => {}
            return_revert!() => (),
            return_ok!() => {}
            InstructionResult::CallOrCreate => (),
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
            InstructionResult::OutOfFund,
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
