use context_interface::{ContextTr, LocalContextTr};
use core::ops::Range;
use primitives::{AddressAndId, Bytes, U256};
/// Input enum for a call.
///
/// As CallInput uses shared memory buffer it can get overridden if not used directly when call happens.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallInput {
    /// The Range points to the SharedMemory buffer. Buffer can be found in [`context_interface::LocalContextTr::shared_memory_buffer_slice`] function.
    /// And can be accessed with `evm.ctx().local().shared_memory_buffer()`
    ///
    /// # Warning
    ///
    /// Use it with caution, CallInput shared buffer can be overridden if context from child call is returned so
    /// recommendation is to fetch buffer at first Inspector call and clone it from [`context_interface::LocalContextTr::shared_memory_buffer_slice`] function.
    SharedBuffer(Range<usize>),
    /// Bytes of the call data.
    Bytes(Bytes),
}

impl CallInput {
    /// Returns the length of the call input.
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::SharedBuffer(range) => range.len(),
        }
    }

    /// Returns `true` if the call input is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the bytes of the call input.
    ///
    /// SharedMemory buffer can be shrunked or overwritten if the child call returns the
    /// shared memory context to its parent, the range in `CallInput::SharedBuffer` can show unexpected data.
    ///
    /// # Allocation
    ///
    /// If this `CallInput` is a `SharedBuffer`, the slice will be copied
    /// into a fresh `Bytes` buffer, which can pose a performance penalty.
    pub fn bytes<CTX>(&self, ctx: &mut CTX) -> Bytes
    where
        CTX: ContextTr,
    {
        match self {
            CallInput::Bytes(bytes) => bytes.clone(),
            CallInput::SharedBuffer(range) => ctx
                .local()
                .shared_memory_buffer_slice(range.clone())
                .map(|b| Bytes::from(b.to_vec()))
                .unwrap_or_default(),
        }
    }
}

impl Default for CallInput {
    #[inline]
    fn default() -> Self {
        CallInput::SharedBuffer(0..0)
    }
}

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The call data of the call.
    pub input: CallInput,
    /// The return memory offset where the output of the call is written.
    pub return_memory_offset: Range<usize>,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The account address of bytecode that is going to be executed.
    ///
    /// Previously `context.code_address`.
    pub bytecode_address: AddressAndId,
    /// Target address, this account storage is going to be modified.
    ///
    /// Previously `context.address`.
    pub target_address: AddressAndId,
    /// This caller is invoking the call.
    ///
    /// Previously `context.caller`.
    pub caller: AddressAndId,
    /// Call value.
    ///
    /// **Note**: This value may not necessarily be transferred from caller to callee, see [`CallValue`].
    ///
    /// Previously `transfer.value` or `context.apparent_value`.
    pub value: CallValue,
    /// The call scheme.
    ///
    /// Previously `context.scheme`.
    pub scheme: CallScheme,
    /// Whether the call is a static call, or is initiated inside a static call.
    pub is_static: bool,
}

impl CallInputs {
    /// Returns `true` if the call will transfer a non-zero value.
    #[inline]
    pub fn transfers_value(&self) -> bool {
        self.value.transfer().is_some_and(|x| x > U256::ZERO)
    }

    /// Returns the transfer value.
    ///
    /// This is the value that is transferred from caller to callee, see [`CallValue`].
    #[inline]
    pub const fn transfer_value(&self) -> Option<U256> {
        self.value.transfer()
    }

    /// Returns the **apparent** call value.
    ///
    /// This value is not actually transferred, see [`CallValue`].
    #[inline]
    pub const fn apparent_value(&self) -> Option<U256> {
        self.value.apparent()
    }

    /// Returns the address of the transfer source account.
    ///
    /// This is only meaningful if `transfers_value` is `true`.
    #[inline]
    pub const fn transfer_from(&self) -> AddressAndId {
        self.caller
    }

    /// Returns the address of the transfer target account.
    ///
    /// This is only meaningful if `transfers_value` is `true`.
    #[inline]
    pub const fn transfer_to(&self) -> AddressAndId {
        self.target_address
    }

    /// Returns the call value, regardless of the transfer value type.
    ///
    /// **Note**: This value may not necessarily be transferred from caller to callee, see [`CallValue`].
    #[inline]
    pub const fn call_value(&self) -> U256 {
        self.value.get()
    }
}

/// Call scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallScheme {
    /// `CALL`.
    Call,
    /// `CALLCODE`
    CallCode,
    /// `DELEGATECALL`
    DelegateCall,
    /// `STATICCALL`
    StaticCall,
}

impl CallScheme {
    /// Returns true if it is `CALL`.
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call)
    }

    /// Returns true if it is `CALLCODE`.
    pub fn is_call_code(&self) -> bool {
        matches!(self, Self::CallCode)
    }

    /// Returns true if it is `DELEGATECALL`.
    pub fn is_delegate_call(&self) -> bool {
        matches!(self, Self::DelegateCall)
    }

    /// Returns true if it is `STATICCALL`.
    pub fn is_static_call(&self) -> bool {
        matches!(self, Self::StaticCall)
    }
}

/// Call value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallValue {
    /// Concrete value, transferred from caller to callee at the end of the transaction.
    Transfer(U256),
    /// Apparent value, that is **not** actually transferred.
    ///
    /// Set when in a `DELEGATECALL` call type, and used by the `CALLVALUE` opcode.
    Apparent(U256),
}

impl Default for CallValue {
    #[inline]
    fn default() -> Self {
        CallValue::Transfer(U256::ZERO)
    }
}

impl CallValue {
    /// Returns the call value, regardless of the type.
    #[inline]
    pub const fn get(&self) -> U256 {
        match *self {
            Self::Transfer(value) | Self::Apparent(value) => value,
        }
    }

    /// Returns the transferred value, if any.
    #[inline]
    pub const fn transfer(&self) -> Option<U256> {
        match *self {
            Self::Transfer(transfer) => Some(transfer),
            Self::Apparent(_) => None,
        }
    }

    /// Returns whether the call value will be transferred.
    #[inline]
    pub const fn is_transfer(&self) -> bool {
        matches!(self, Self::Transfer(_))
    }

    /// Returns the apparent value, if any.
    #[inline]
    pub const fn apparent(&self) -> Option<U256> {
        match *self {
            Self::Transfer(_) => None,
            Self::Apparent(apparent) => Some(apparent),
        }
    }

    /// Returns whether the call value is apparent, and not actually transferred.
    #[inline]
    pub const fn is_apparent(&self) -> bool {
        matches!(self, Self::Apparent(_))
    }
}
