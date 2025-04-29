use core::{
    cell::{Ref, RefCell},
    ops::Range,
};
use primitives::{Address, Bytes, U256};
use std::rc::Rc;

/// Input enum for a call.
///
/// Rc<RefCell<..>> buffer introduces some UI restrictions. We can't have a function
/// where input is returned as slice as RefCell does not allow for this. If you need
/// a new variable you can use [`Self::bytes`] function.
///
/// # Note
///
/// As CallInput uses shared memory buffer it can get overriden if not used directly when call happens.
/// Best option if input is needed would be to clone inputs with [`Self::bytes`] function.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallInput {
    /// The Range of the call data to be taken from SharedMemory
    SharedBuffer {
        /// The range that points to the buffer.
        range: Range<usize>,
        /// The buffer used in Interpreter SharedMemory that is used as input.
        buffer: Rc<RefCell<Vec<u8>>>,
    },
    /// Bytes of the call data.
    Bytes(Bytes),
}

impl CallInput {
    /// Return a slice from Shared Buffer, in case this is [`CallInput::Bytes`] variant return None.
    ///
    /// In case of borrow failure or invalid range, return None.
    ///
    /// # Note
    ///
    /// CallInput shared buffer can be overriden when used in later calls.
    pub fn shared_buffer_slice(&self) -> Option<Ref<'_, [u8]>> {
        match self {
            Self::SharedBuffer { range, buffer } => {
                let borrow = buffer.try_borrow().ok()?;
                // check that range is valid
                borrow.get(range.clone())?;
                Some(Ref::map(borrow, |b| {
                    b.get(range.clone()).unwrap_or_default()
                }))
            }
            Self::Bytes(i) => None,
        }
    }

    /// Returns the bytes of the call inputs, or none if range input can't be obtained.
    ///
    /// # Note
    ///
    /// If option `Range` is used, the returned bytes are copied from the buffer, this can be expensive opperation
    /// if used in the loop.
    ///
    /// In case that buffer can't be obtained or if the range is invalid, the returned bytes are empty. If buffer is
    /// overridden with next call invalid data will be returned.
    pub fn bytes(&self) -> Option<Bytes> {
        match self {
            Self::SharedBuffer { range, buffer } => Some(
                buffer
                    .try_borrow()
                    .ok()?
                    .get(range.clone())?
                    .to_vec()
                    .into(),
            ),
            Self::Bytes(bytes) => Some(bytes.clone()),
        }
    }
}

impl Default for CallInput {
    /// Returns a default `CallInput` with an empty `Bytes`.
    #[inline]
    fn default() -> Self {
        CallInput::Bytes(Bytes::default())
    }
}

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The call data of the call.
    pub input: CallInput,
    /// The return memory offset where the output of the call is written.
    ///
    /// In EOF, this range is invalid as EOF calls do not write output to memory.
    pub return_memory_offset: Range<usize>,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The account address of bytecode that is going to be executed.
    ///
    /// Previously `context.code_address`.
    pub bytecode_address: Address,
    /// Target address, this account storage is going to be modified.
    ///
    /// Previously `context.address`.
    pub target_address: Address,
    /// This caller is invoking the call.
    ///
    /// Previously `context.caller`.
    pub caller: Address,
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
    /// Whether the call is initiated from EOF bytecode.
    pub is_eof: bool,
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
    pub const fn transfer_from(&self) -> Address {
        self.caller
    }

    /// Returns the address of the transfer target account.
    ///
    /// This is only meaningful if `transfers_value` is `true`.
    #[inline]
    pub const fn transfer_to(&self) -> Address {
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
    /// `EXTCALL`
    ExtCall,
    /// `EXTSTATICCALL`
    ExtStaticCall,
    /// `EXTDELEGATECALL`
    ExtDelegateCall,
}

impl CallScheme {
    /// Returns true if it is EOF EXT*CALL.
    pub fn is_ext(&self) -> bool {
        matches!(
            self,
            Self::ExtCall | Self::ExtStaticCall | Self::ExtDelegateCall
        )
    }

    /// Returns true if it is ExtDelegateCall.
    pub fn is_ext_delegate_call(&self) -> bool {
        matches!(self, Self::ExtDelegateCall)
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
