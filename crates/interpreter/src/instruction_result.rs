use context_interface::{
    journaled_state::TransferError,
    result::{HaltReason, OutOfGasError, SuccessReason},
};
use core::fmt::Debug;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstructionResult {
    /// Encountered a `STOP` opcode
    #[default]
    Stop,
    /// Return from the current call.
    Return,
    /// Self-destruct the current contract.
    SelfDestruct,
    /// Return a contract (used in contract creation).
    ReturnContract,

    // Revert Codes
    /// Revert the transaction.
    Revert = 0x10,
    /// Exceeded maximum call depth.
    CallTooDeep,
    /// Insufficient funds for transfer.
    OutOfFunds,
    /// Revert if `CREATE`/`CREATE2` starts with `0xEF00`.
    CreateInitCodeStartingEF00,
    /// Invalid EVM Object Format (EOF) init code.
    InvalidEOFInitCode,
    /// `ExtDelegateCall` calling a non EOF contract.
    InvalidExtDelegateCallTarget,

    // Error Codes
    /// Out of gas error.
    OutOfGas = 0x50,
    /// Out of gas error encountered during memory expansion.
    MemoryOOG,
    /// The memory limit of the EVM has been exceeded.
    MemoryLimitOOG,
    /// Out of gas error encountered during the execution of a precompiled contract.
    PrecompileOOG,
    /// Out of gas error encountered while calling an invalid operand.
    InvalidOperandOOG,
    /// Out of gas error encountered while checking for reentrancy sentry.
    ReentrancySentryOOG,
    /// Unknown or invalid opcode.
    OpcodeNotFound,
    /// Invalid `CALL` with value transfer in static context.
    CallNotAllowedInsideStatic,
    /// Invalid state modification in static call.
    StateChangeDuringStaticCall,
    /// An undefined bytecode value encountered during execution.
    InvalidFEOpcode,
    /// Invalid jump destination. Dynamic jumps points to invalid not jumpdest opcode.
    InvalidJump,
    /// The feature or opcode is not activated in this version of the EVM.
    NotActivated,
    /// Attempting to pop a value from an empty stack.
    StackUnderflow,
    /// Attempting to push a value onto a full stack.
    StackOverflow,
    /// Invalid memory or storage offset.
    OutOfOffset,
    /// Address collision during contract creation.
    CreateCollision,
    /// Payment amount overflow.
    OverflowPayment,
    /// Error in precompiled contract execution.
    PrecompileError,
    /// Nonce overflow.
    NonceOverflow,
    /// Exceeded contract size limit during creation.
    CreateContractSizeLimit,
    /// Created contract starts with invalid bytes (`0xEF`).
    CreateContractStartingWithEF,
    /// Exceeded init code size limit (EIP-3860:  Limit and meter initcode).
    CreateInitCodeSizeLimit,
    /// Fatal external error. Returned by database.
    FatalExternalError,
    /// `RETURNCONTRACT` called outside init EOF code.
    ReturnContractInNotInitEOF,
    /// Legacy contract is calling opcode that is enabled only in EOF.
    EOFOpcodeDisabledInLegacy,
    /// Stack overflow in EOF subroutine function calls.
    SubRoutineStackOverflow,
    /// Aux data overflow, new aux data is larger than `u16` max size.
    EofAuxDataOverflow,
    /// Aux data is smaller than already present data size.
    EofAuxDataTooSmall,
    /// `EXT*CALL` target address needs to be padded with 0s.
    InvalidEXTCALLTarget,
}

impl From<TransferError> for InstructionResult {
    fn from(e: TransferError) -> Self {
        match e {
            TransferError::OutOfFunds => InstructionResult::OutOfFunds,
            TransferError::OverflowPayment => InstructionResult::OverflowPayment,
            TransferError::CreateCollision => InstructionResult::CreateCollision,
        }
    }
}

impl From<SuccessReason> for InstructionResult {
    fn from(value: SuccessReason) -> Self {
        match value {
            SuccessReason::Return => InstructionResult::Return,
            SuccessReason::Stop => InstructionResult::Stop,
            SuccessReason::SelfDestruct => InstructionResult::SelfDestruct,
            SuccessReason::EofReturnContract => InstructionResult::ReturnContract,
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
                OutOfGasError::ReentrancySentry => Self::ReentrancySentryOOG,
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
            HaltReason::EofAuxDataOverflow => Self::EofAuxDataOverflow,
            HaltReason::EofAuxDataTooSmall => Self::EofAuxDataTooSmall,
            HaltReason::SubRoutineStackOverflow => Self::SubRoutineStackOverflow,
            HaltReason::InvalidEXTCALLTarget => Self::InvalidEXTCALLTarget,
        }
    }
}

#[macro_export]
macro_rules! return_ok {
    () => {
        $crate::InstructionResult::Stop
            | $crate::InstructionResult::Return
            | $crate::InstructionResult::SelfDestruct
            | $crate::InstructionResult::ReturnContract
    };
}

#[macro_export]
macro_rules! return_revert {
    () => {
        $crate::InstructionResult::Revert
            | $crate::InstructionResult::CallTooDeep
            | $crate::InstructionResult::OutOfFunds
            | $crate::InstructionResult::InvalidEOFInitCode
            | $crate::InstructionResult::CreateInitCodeStartingEF00
            | $crate::InstructionResult::InvalidExtDelegateCallTarget
    };
}

#[macro_export]
macro_rules! return_error {
    () => {
        $crate::InstructionResult::OutOfGas
            | $crate::InstructionResult::MemoryOOG
            | $crate::InstructionResult::MemoryLimitOOG
            | $crate::InstructionResult::PrecompileOOG
            | $crate::InstructionResult::InvalidOperandOOG
            | $crate::InstructionResult::ReentrancySentryOOG
            | $crate::InstructionResult::OpcodeNotFound
            | $crate::InstructionResult::CallNotAllowedInsideStatic
            | $crate::InstructionResult::StateChangeDuringStaticCall
            | $crate::InstructionResult::InvalidFEOpcode
            | $crate::InstructionResult::InvalidJump
            | $crate::InstructionResult::NotActivated
            | $crate::InstructionResult::StackUnderflow
            | $crate::InstructionResult::StackOverflow
            | $crate::InstructionResult::OutOfOffset
            | $crate::InstructionResult::CreateCollision
            | $crate::InstructionResult::OverflowPayment
            | $crate::InstructionResult::PrecompileError
            | $crate::InstructionResult::NonceOverflow
            | $crate::InstructionResult::CreateContractSizeLimit
            | $crate::InstructionResult::CreateContractStartingWithEF
            | $crate::InstructionResult::CreateInitCodeSizeLimit
            | $crate::InstructionResult::FatalExternalError
            | $crate::InstructionResult::ReturnContractInNotInitEOF
            | $crate::InstructionResult::EOFOpcodeDisabledInLegacy
            | $crate::InstructionResult::SubRoutineStackOverflow
            | $crate::InstructionResult::EofAuxDataTooSmall
            | $crate::InstructionResult::EofAuxDataOverflow
            | $crate::InstructionResult::InvalidEXTCALLTarget
    };
}

impl InstructionResult {
    /// Returns whether the result is a success.
    #[inline]
    pub const fn is_ok(self) -> bool {
        matches!(self, crate::return_ok!())
    }

    #[inline]
    pub const fn is_ok_or_revert(self) -> bool {
        matches!(self, crate::return_ok!() | crate::return_revert!())
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

/// Internal results that are not exposed externally
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InternalResult {
    /// Internal CREATE/CREATE starts with 0xEF00
    CreateInitCodeStartingEF00,
    /// Internal to ExtDelegateCall
    InvalidExtDelegateCallTarget,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SuccessOrHalt<HaltReasonTr> {
    Success(SuccessReason),
    Revert,
    Halt(HaltReasonTr),
    FatalExternalError,
    Internal(InternalResult),
}

impl<HaltReasonTr> SuccessOrHalt<HaltReasonTr> {
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
    pub fn to_halt(self) -> Option<HaltReasonTr> {
        match self {
            SuccessOrHalt::Halt(reason) => Some(reason),
            _ => None,
        }
    }
}

impl<HALT: From<HaltReason>> From<HaltReason> for SuccessOrHalt<HALT> {
    fn from(reason: HaltReason) -> Self {
        SuccessOrHalt::Halt(reason.into())
    }
}

impl<HaltReasonTr: From<HaltReason>> From<InstructionResult> for SuccessOrHalt<HaltReasonTr> {
    fn from(result: InstructionResult) -> Self {
        match result {
            InstructionResult::Stop => Self::Success(SuccessReason::Stop),
            InstructionResult::Return => Self::Success(SuccessReason::Return),
            InstructionResult::SelfDestruct => Self::Success(SuccessReason::SelfDestruct),
            InstructionResult::Revert => Self::Revert,
            InstructionResult::CreateInitCodeStartingEF00 => Self::Revert,
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
            InstructionResult::ReentrancySentryOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::ReentrancySentry).into())
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
            // TODO : (EOF) Add proper Revert subtype.
            InstructionResult::InvalidEOFInitCode => Self::Revert,
            InstructionResult::FatalExternalError => Self::FatalExternalError,
            InstructionResult::EOFOpcodeDisabledInLegacy => {
                Self::Halt(HaltReason::OpcodeNotFound.into())
            }
            InstructionResult::SubRoutineStackOverflow => {
                Self::Halt(HaltReason::SubRoutineStackOverflow.into())
            }
            InstructionResult::ReturnContract => Self::Success(SuccessReason::EofReturnContract),
            InstructionResult::EofAuxDataOverflow => {
                Self::Halt(HaltReason::EofAuxDataOverflow.into())
            }
            InstructionResult::EofAuxDataTooSmall => {
                Self::Halt(HaltReason::EofAuxDataTooSmall.into())
            }
            InstructionResult::InvalidEXTCALLTarget => {
                Self::Halt(HaltReason::InvalidEXTCALLTarget.into())
            }
            InstructionResult::InvalidExtDelegateCallTarget => {
                Self::Internal(InternalResult::InvalidExtDelegateCallTarget)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InstructionResult;

    #[test]
    fn all_results_are_covered() {
        match InstructionResult::Stop {
            return_error!() => {}
            return_revert!() => {}
            return_ok!() => {}
        }
    }

    #[test]
    fn test_results() {
        let ok_results = vec![
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
