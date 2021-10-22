use crate::collection::Cow;

/// Exit reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitReason {
    /// Machine has succeeded.
    Succeed(ExitSucceed),
    /// Machine returns a normal EVM error.
    Error(ExitError),
    /// Machine encountered an explict revert.
    Revert(ExitRevert),
    /// Machine encountered an error that is not supposed to be normal EVM
    /// errors, such as requiring too much memory to execute.
    Fatal(ExitFatal),
}

impl ExitReason {
    /// Whether the exit is succeeded.
    pub fn is_succeed(&self) -> bool {
        matches!(self, Self::Succeed(_))
    }

    /// Whether the exit is error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Whether the exit is revert.
    pub fn is_revert(&self) -> bool {
        matches!(self, Self::Revert(_))
    }

    /// Whether the exit is fatal.
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }
}

/// Exit succeed reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitSucceed {
    /// Machine encountered an explict stop.
    Stopped,
    /// Machine encountered an explict return.
    Returned,
    /// Machine encountered an explict selfdestruct.
    SelfDestructed,
}

impl From<ExitSucceed> for ExitReason {
    fn from(s: ExitSucceed) -> Self {
        Self::Succeed(s)
    }
}

/// Exit revert reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitRevert {
    /// Machine encountered an explict revert.
    Reverted,
    /// Account does not have balance, revert it.
    OutOfFund,
    /// Hit call stack limit
    CallTooDeep,
}

impl From<ExitRevert> for ExitReason {
    fn from(s: ExitRevert) -> Self {
        Self::Revert(s)
    }
}

/// Exit error reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitError {
    /// Trying to pop from an empty stack.
    StackUnderflow,
    /// Trying to push into a stack over stack limit.
    StackOverflow,
    /// Jump destination is invalid.
    InvalidJump,
    /// An opcode accesses memory region, but the region is invalid.
    InvalidRange,
    /// Encountered the designated invalid opcode.
    DesignatedInvalid,
    /// Create opcode encountered collision (runtime).
    CreateCollision,
    /// Create init code exceeds limit (runtime).
    CreateContractLimit,
    /// Create contract that begins with EF
    CreateContractWithEF,

    /// An opcode accesses external information, but the request is off offset
    /// limit (runtime).
    OutOfOffset,
    /// Execution runs out of gas (runtime).
    OutOfGas,
    /// Not enough fund to start the execution (runtime).
    OutOfFund,

    /// PC underflowed (unused).
    PCUnderflow,
    /// Attempt to create an empty account (runtime, unused).
    CreateEmpty,

    /// opcode not found,
    OpcodeNotFound,

    /// calling CALL inside static call
    CallNotAllowedInsideStatic,

    /// Other normal errors.
    Other(Cow<'static, str>),
}

impl From<ExitError> for ExitReason {
    fn from(s: ExitError) -> Self {
        Self::Error(s)
    }
}

/// Exit fatal reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitFatal {
    /// The operation is not supported.
    NotSupported,
    /// The environment explictly set call errors as fatal error.
    CallErrorAsFatal(ExitError),

    /// Other fatal errors.
    Other(Cow<'static, str>),
}

impl From<ExitFatal> for ExitReason {
    fn from(s: ExitFatal) -> Self {
        Self::Fatal(s)
    }
}
