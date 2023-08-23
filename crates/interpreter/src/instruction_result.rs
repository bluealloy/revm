use crate::primitives::{Eval, Halt};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstructionResult {
    // success codes
    Continue = 0x00,
    Stop = 0x01,
    Return = 0x02,
    SelfDestruct = 0x03,

    // revert codes
    Revert = 0x20, // revert opcode
    CallTooDeep = 0x21,
    OutOfFund = 0x22,

    // error codes
    OutOfGas = 0x50,
    MemoryOOG = 0x51,
    MemoryLimitOOG = 0x52,
    PrecompileOOG = 0x53,
    InvalidOperandOOG = 0x54,
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
    CreateInitcodeSizeLimit,

    /// Fatal external error. Returned by database.
    FatalExternalError,
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
        matches!(
            self,
            Self::OutOfGas
                | Self::MemoryOOG
                | Self::MemoryLimitOOG
                | Self::PrecompileOOG
                | Self::InvalidOperandOOG
                | Self::OpcodeNotFound
                | Self::CallNotAllowedInsideStatic
                | Self::StateChangeDuringStaticCall
                | Self::InvalidFEOpcode
                | Self::InvalidJump
                | Self::NotActivated
                | Self::StackUnderflow
                | Self::StackOverflow
                | Self::OutOfOffset
                | Self::CreateCollision
                | Self::OverflowPayment
                | Self::PrecompileError
                | Self::NonceOverflow
                | Self::CreateContractSizeLimit
                | Self::CreateContractStartingWithEF
                | Self::CreateInitcodeSizeLimit
                | Self::FatalExternalError
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuccessOrHalt {
    Success(Eval),
    Revert,
    Halt(Halt),
    FatalExternalError,
    // this is internal opcode.
    InternalContinue,
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
            InstructionResult::CreateInitcodeSizeLimit => Self::Halt(Halt::CreateInitcodeSizeLimit),
            InstructionResult::FatalExternalError => Self::FatalExternalError,
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
