use revm_primitives::{Eval, Halt};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstructionResult {
    //success codes
    Continue = 0x00,
    Stop = 0x01,
    Return = 0x02,
    SelfDestruct = 0x03,

    // revert code
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

    // Fatal external error. Returned by database.
    FatalExternalError,
}

pub enum SuccessOrHalt {
    Success(Eval),
    Revert,
    Halt(Halt),
    FatalExternalError,
    // this is internal opcode.
    Internal,
}

impl From<InstructionResult> for SuccessOrHalt {
    fn from(result: InstructionResult) -> Self {
        match result {
            InstructionResult::Continue => Self::Internal, // used only in interpreter loop
            InstructionResult::Stop => Self::Success(Eval::Stop),
            InstructionResult::Return => Self::Success(Eval::Return),
            InstructionResult::SelfDestruct => Self::Success(Eval::SelfDestruct),
            InstructionResult::Revert => Self::Revert,
            InstructionResult::CallTooDeep => Self::Internal, // not gonna happen for first call
            InstructionResult::OutOfFund => Self::Internal, // Check for first call is done separately.
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
            InstructionResult::CallNotAllowedInsideStatic => Self::Internal, // first call is not static call
            InstructionResult::StateChangeDuringStaticCall => Self::Internal,
            InstructionResult::InvalidFEOpcode => Self::Halt(Halt::InvalidFEOpcode),
            InstructionResult::InvalidJump => Self::Halt(Halt::InvalidJump),
            InstructionResult::NotActivated => Self::Halt(Halt::NotActivated),
            InstructionResult::StackUnderflow => Self::Halt(Halt::StackUnderflow),
            InstructionResult::StackOverflow => Self::Halt(Halt::StackOverflow),
            InstructionResult::OutOfOffset => Self::Halt(Halt::OutOfOffset),
            InstructionResult::CreateCollision => Self::Halt(Halt::CreateCollision),
            InstructionResult::OverflowPayment => Self::Internal, // Check for first call is done separately.
            InstructionResult::PrecompileError => Self::Halt(Halt::PrecompileError),
            InstructionResult::NonceOverflow => Self::Halt(Halt::NonceOverflow),
            InstructionResult::CreateContractSizeLimit => Self::Halt(Halt::CreateContractSizeLimit),
            InstructionResult::CreateContractStartingWithEF => {
                Self::Halt(Halt::CreateContractSizeLimit)
            }
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
