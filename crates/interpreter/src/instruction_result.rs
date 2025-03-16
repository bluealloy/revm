use crate::primitives::{HaltReason, OutOfGasError, SuccessReason};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstructionResult {
    // Success Codes
    #[default]
    /// Execution should continue to the next one.
    Continue = 0x00,
    /// Encountered a `STOP` opcode
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

    // Action Codes
    /// Indicates a call or contract creation.
    CallOrCreate = 0x20,

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
    EOFFunctionStackOverflow,
    /// Aux data overflow, new aux data is larger than `u16` max size.
    EofAuxDataOverflow,
    /// Aux data is smaller then already present data size.
    EofAuxDataTooSmall,
    /// `EXT*CALL` target address needs to be padded with 0s.
    InvalidEXTCALLTarget,

    // fluentbase error codes
    RootCallOnly = 0x80,
    MalformedBuiltinParams,
    CallDepthOverflow,
    NonNegativeExitCode,
    UnknownError,
    InputOutputOutOfBounds,
    // trap error codes
    UnreachableCodeReached,
    MemoryOutOfBounds,
    TableOutOfBounds,
    IndirectCallToNull,
    IntegerDivisionByZero,
    IntegerOverflow,
    BadConversionToInteger,
    BadSignature,
    OutOfFuel,
    GrowthOperationLimited,
    UnresolvedFunction,
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
            HaltReason::EOFFunctionStackOverflow => Self::EOFFunctionStackOverflow,
            HaltReason::InvalidEXTCALLTarget => Self::InvalidEXTCALLTarget,
            #[cfg(feature = "optimism")]
            HaltReason::FailedDeposit => Self::FatalExternalError,
            HaltReason::RootCallOnly => Self::RootCallOnly,
            HaltReason::MalformedBuiltinParams => Self::MalformedBuiltinParams,
            HaltReason::CallDepthOverflow => Self::CallDepthOverflow,
            HaltReason::NonNegativeExitCode => Self::NonNegativeExitCode,
            HaltReason::UnknownError => Self::UnknownError,
            HaltReason::InputOutputOutOfBounds => Self::InputOutputOutOfBounds,
            HaltReason::UnreachableCodeReached => Self::UnreachableCodeReached,
            HaltReason::MemoryOutOfBounds => Self::MemoryOutOfBounds,
            HaltReason::TableOutOfBounds => Self::TableOutOfBounds,
            HaltReason::IndirectCallToNull => Self::IndirectCallToNull,
            HaltReason::IntegerDivisionByZero => Self::IntegerDivisionByZero,
            HaltReason::IntegerOverflow => Self::IntegerOverflow,
            HaltReason::BadConversionToInteger => Self::BadConversionToInteger,
            HaltReason::BadSignature => Self::BadSignature,
            HaltReason::OutOfFuel => Self::OutOfFuel,
            HaltReason::GrowthOperationLimited => Self::GrowthOperationLimited,
            HaltReason::UnresolvedFunction => Self::UnresolvedFunction,
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
        InstructionResult::Revert
            | InstructionResult::CallTooDeep
            | InstructionResult::OutOfFunds
            | InstructionResult::InvalidEOFInitCode
            | InstructionResult::CreateInitCodeStartingEF00
            | InstructionResult::InvalidExtDelegateCallTarget
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
            | InstructionResult::EofAuxDataTooSmall
            | InstructionResult::EofAuxDataOverflow
            | InstructionResult::InvalidEXTCALLTarget
            | InstructionResult::RootCallOnly
            | InstructionResult::MalformedBuiltinParams
            | InstructionResult::CallDepthOverflow
            | InstructionResult::NonNegativeExitCode
            | InstructionResult::UnknownError
            | InstructionResult::InputOutputOutOfBounds
            | InstructionResult::UnreachableCodeReached
            | InstructionResult::MemoryOutOfBounds
            | InstructionResult::TableOutOfBounds
            | InstructionResult::IndirectCallToNull
            | InstructionResult::IntegerDivisionByZero
            | InstructionResult::IntegerOverflow
            | InstructionResult::BadConversionToInteger
            | InstructionResult::BadSignature
            | InstructionResult::OutOfFuel
            | InstructionResult::GrowthOperationLimited
            | InstructionResult::UnresolvedFunction
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

/// Internal result that are not ex
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InternalResult {
    /// Internal instruction that signals Interpreter should continue running.
    InternalContinue,
    /// Internal instruction that signals call or create.
    InternalCallOrCreate,
    /// Internal CREATE/CREATE starts with 0xEF00
    CreateInitCodeStartingEF00,
    /// Internal to ExtDelegateCall
    InvalidExtDelegateCallTarget,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SuccessOrHalt {
    Success(SuccessReason),
    Revert,
    Halt(HaltReason),
    FatalExternalError,
    Internal(InternalResult),
}

impl SuccessOrHalt {
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
    pub fn to_halt(self) -> Option<HaltReason> {
        match self {
            SuccessOrHalt::Halt(reason) => Some(reason),
            _ => None,
        }
    }
}

impl From<InstructionResult> for SuccessOrHalt {
    fn from(result: InstructionResult) -> Self {
        match result {
            InstructionResult::Continue => Self::Internal(InternalResult::InternalContinue), /* used only in interpreter loop */
            InstructionResult::Stop => Self::Success(SuccessReason::Stop),
            InstructionResult::Return => Self::Success(SuccessReason::Return),
            InstructionResult::SelfDestruct => Self::Success(SuccessReason::SelfDestruct),
            InstructionResult::Revert => Self::Revert,
            InstructionResult::CreateInitCodeStartingEF00 => Self::Revert,
            InstructionResult::CallOrCreate => Self::Internal(InternalResult::InternalCallOrCreate), /* used only in interpreter loop */
            InstructionResult::CallTooDeep => Self::Halt(HaltReason::CallTooDeep), /* not gonna happen for first call */
            InstructionResult::OutOfFunds => Self::Halt(HaltReason::OutOfFunds), /* Check for first call is done separately. */
            InstructionResult::OutOfGas => Self::Halt(HaltReason::OutOfGas(OutOfGasError::Basic)),
            InstructionResult::MemoryLimitOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::MemoryLimit))
            }
            InstructionResult::MemoryOOG => Self::Halt(HaltReason::OutOfGas(OutOfGasError::Memory)),
            InstructionResult::PrecompileOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::Precompile))
            }
            InstructionResult::InvalidOperandOOG => {
                Self::Halt(HaltReason::OutOfGas(OutOfGasError::InvalidOperand))
            }
            InstructionResult::OpcodeNotFound | InstructionResult::ReturnContractInNotInitEOF => {
                Self::Halt(HaltReason::OpcodeNotFound)
            }
            InstructionResult::CallNotAllowedInsideStatic => {
                Self::Halt(HaltReason::CallNotAllowedInsideStatic)
            } // first call is not static call
            InstructionResult::StateChangeDuringStaticCall => {
                Self::Halt(HaltReason::StateChangeDuringStaticCall)
            }
            InstructionResult::InvalidFEOpcode => Self::Halt(HaltReason::InvalidFEOpcode),
            InstructionResult::InvalidJump => Self::Halt(HaltReason::InvalidJump),
            InstructionResult::NotActivated => Self::Halt(HaltReason::NotActivated),
            InstructionResult::StackUnderflow => Self::Halt(HaltReason::StackUnderflow),
            InstructionResult::StackOverflow => Self::Halt(HaltReason::StackOverflow),
            InstructionResult::OutOfOffset => Self::Halt(HaltReason::OutOfOffset),
            InstructionResult::CreateCollision => Self::Halt(HaltReason::CreateCollision),
            InstructionResult::OverflowPayment => Self::Halt(HaltReason::OverflowPayment), /* Check for first call is done separately. */
            InstructionResult::PrecompileError => Self::Halt(HaltReason::PrecompileError),
            InstructionResult::NonceOverflow => Self::Halt(HaltReason::NonceOverflow),
            InstructionResult::CreateContractSizeLimit
            | InstructionResult::CreateContractStartingWithEF => {
                Self::Halt(HaltReason::CreateContractSizeLimit)
            }
            InstructionResult::CreateInitCodeSizeLimit => {
                Self::Halt(HaltReason::CreateInitCodeSizeLimit)
            }
            // TODO (EOF) add proper Revert subtype.
            InstructionResult::InvalidEOFInitCode => Self::Revert,
            InstructionResult::FatalExternalError => Self::FatalExternalError,
            InstructionResult::EOFOpcodeDisabledInLegacy => Self::Halt(HaltReason::OpcodeNotFound),
            InstructionResult::EOFFunctionStackOverflow => {
                Self::Halt(HaltReason::EOFFunctionStackOverflow)
            }
            InstructionResult::ReturnContract => Self::Success(SuccessReason::EofReturnContract),
            InstructionResult::EofAuxDataOverflow => Self::Halt(HaltReason::EofAuxDataOverflow),
            InstructionResult::EofAuxDataTooSmall => Self::Halt(HaltReason::EofAuxDataTooSmall),
            InstructionResult::InvalidEXTCALLTarget => Self::Halt(HaltReason::InvalidEXTCALLTarget),
            InstructionResult::InvalidExtDelegateCallTarget => {
                Self::Internal(InternalResult::InvalidExtDelegateCallTarget)
            }
            InstructionResult::RootCallOnly => Self::Halt(HaltReason::RootCallOnly),
            InstructionResult::MalformedBuiltinParams => {
                Self::Halt(HaltReason::MalformedBuiltinParams)
            }
            InstructionResult::CallDepthOverflow => Self::Halt(HaltReason::CallDepthOverflow),
            InstructionResult::NonNegativeExitCode => Self::Halt(HaltReason::NonNegativeExitCode),
            InstructionResult::UnknownError => Self::Halt(HaltReason::UnknownError),
            InstructionResult::InputOutputOutOfBounds => {
                Self::Halt(HaltReason::InputOutputOutOfBounds)
            }
            InstructionResult::UnreachableCodeReached => {
                Self::Halt(HaltReason::UnreachableCodeReached)
            }
            InstructionResult::MemoryOutOfBounds => Self::Halt(HaltReason::MemoryOutOfBounds),
            InstructionResult::TableOutOfBounds => Self::Halt(HaltReason::TableOutOfBounds),
            InstructionResult::IndirectCallToNull => Self::Halt(HaltReason::IndirectCallToNull),
            InstructionResult::IntegerDivisionByZero => {
                Self::Halt(HaltReason::IntegerDivisionByZero)
            }
            InstructionResult::IntegerOverflow => Self::Halt(HaltReason::IntegerOverflow),
            InstructionResult::BadConversionToInteger => {
                Self::Halt(HaltReason::BadConversionToInteger)
            }
            InstructionResult::BadSignature => Self::Halt(HaltReason::BadSignature),
            InstructionResult::OutOfFuel => Self::Halt(HaltReason::OutOfFuel),
            InstructionResult::GrowthOperationLimited => {
                Self::Halt(HaltReason::GrowthOperationLimited)
            }
            InstructionResult::UnresolvedFunction => Self::Halt(HaltReason::UnresolvedFunction),
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
