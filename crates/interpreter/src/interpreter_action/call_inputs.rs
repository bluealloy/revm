use crate::interpreter_types::MemoryTr;
use context_interface::{ContextTr, LocalContextTr};
use core::ops::Range;
use primitives::{Address, Bytes, B256, U256};
use state::Bytecode;

/// Input enum for a call.
///
/// As CallInput uses shared memory buffer it can get overridden if not used directly when call happens.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallInput {
    /// Bytes of the call data.
    Bytes(Bytes),
    /// The Range points to the SharedMemory buffer. Buffer can be found in [`context_interface::LocalContextTr::shared_memory_buffer_slice`] function.
    /// And can be accessed with `evm.ctx().local().shared_memory_buffer()`
    ///
    /// # Warning
    ///
    /// Use it with caution, CallInput shared buffer can be overridden if context from child call is returned so
    /// recommendation is to fetch buffer at first Inspector call and clone it from [`context_interface::LocalContextTr::shared_memory_buffer_slice`] function.
    SharedBuffer(Range<usize>),
}

impl CallInput {
    /// Returns the length of the call input.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::SharedBuffer(range) => range.len(),
        }
    }

    /// Returns `true` if the call input is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the bytes of the call input from the given context.
    #[inline]
    pub fn as_bytes<'a, CTX: ContextTr>(
        &'a self,
        ctx: &'a CTX,
    ) -> impl core::ops::Deref<Target = [u8]> + 'a {
        self.as_bytes_local(ctx.local())
    }

    /// Returns the bytes of the call input from the given local context.
    #[inline]
    pub fn as_bytes_local<'a, L: LocalContextTr>(
        &'a self,
        local: &'a L,
    ) -> impl core::ops::Deref<Target = [u8]> + 'a {
        match self {
            Self::Bytes(bytes) => CallInputRef::Bytes(bytes.as_ref()),
            Self::SharedBuffer(range) => {
                CallInputRef::SharedBuffer(local.shared_memory_buffer_slice(range.clone()))
            }
        }
    }

    /// Returns the bytes of the call input from the given memory.
    #[inline]
    pub fn as_bytes_memory<'a, M: MemoryTr>(
        &'a self,
        memory: &'a M,
    ) -> impl core::ops::Deref<Target = [u8]> + 'a {
        match self {
            Self::Bytes(bytes) => CallInputRef::Bytes(bytes.as_ref()),
            Self::SharedBuffer(range) => {
                CallInputRef::SharedBuffer(Some(memory.global_slice(range.clone())))
            }
        }
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
    pub fn bytes<CTX: ContextTr>(&self, ctx: &CTX) -> Bytes {
        self.bytes_local(ctx.local())
    }

    /// Returns the bytes of the call input from the given local context.
    ///
    /// # Allocation
    ///
    /// If the call input is a `SharedBuffer`, this function will allocate a new `Bytes` object.
    #[inline]
    pub fn bytes_local<L: LocalContextTr>(&self, local: &L) -> Bytes {
        match self {
            CallInput::Bytes(bytes) => bytes.clone(),
            CallInput::SharedBuffer(range) => local
                .shared_memory_buffer_slice(range.clone())
                .map(|b| Bytes::from(b.to_vec()))
                .unwrap_or_default(),
        }
    }
}

impl Default for CallInput {
    #[inline]
    fn default() -> Self {
        CallInput::Bytes(Bytes::new())
    }
}

enum CallInputRef<'a> {
    Bytes(&'a [u8]),
    SharedBuffer(Option<core::cell::Ref<'a, [u8]>>),
}

impl core::ops::Deref for CallInputRef<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Bytes(x) => x,
            Self::SharedBuffer(x) => x.as_deref().unwrap_or_default(),
        }
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
    /// State gas reservoir (EIP-8037). Passed from parent frame to child frame.
    pub reservoir: u64,
    /// The account address of bytecode that is going to be executed.
    ///
    /// Previously `context.code_address`.
    pub bytecode_address: Address,
    /// Known bytecode and its hash.
    /// If None, bytecode will be loaded from the account at `bytecode_address`.
    /// If Some((hash, bytecode)), the provided bytecode and hash will be used.
    pub known_bytecode: (B256, Bytecode),
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
}

impl CallScheme {
    /// Returns true if it is `CALL`.
    pub const fn is_call(&self) -> bool {
        matches!(self, Self::Call)
    }

    /// Returns true if it is `CALLCODE`.
    pub const fn is_call_code(&self) -> bool {
        matches!(self, Self::CallCode)
    }

    /// Returns true if it is `DELEGATECALL`.
    pub const fn is_delegate_call(&self) -> bool {
        matches!(self, Self::DelegateCall)
    }

    /// Returns true if it is `STATICCALL`.
    pub const fn is_static_call(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use std::rc::Rc;

    struct TestLocalContext {
        buffer: Rc<RefCell<Vec<u8>>>,
    }

    impl TestLocalContext {
        fn new(data: Vec<u8>) -> Self {
            Self {
                buffer: Rc::new(RefCell::new(data)),
            }
        }
    }

    impl LocalContextTr for TestLocalContext {
        fn shared_memory_buffer(&self) -> &Rc<RefCell<Vec<u8>>> {
            &self.buffer
        }

        fn clear(&mut self) {
            self.buffer.borrow_mut().clear();
        }

        fn set_precompile_error_context(&mut self, _output: String) {}

        fn take_precompile_error_context(&mut self) -> Option<String> {
            None
        }
    }

    #[test]
    fn as_bytes_local_with_bytes_variant() {
        let input = CallInput::Bytes(Bytes::from_static(b"hello"));
        let local = TestLocalContext::new(vec![]);
        let result = input.as_bytes_local(&local);
        assert_eq!(&*result, b"hello");
    }

    #[test]
    fn as_bytes_local_with_shared_buffer() {
        let input = CallInput::SharedBuffer(1..4);
        let local = TestLocalContext::new(vec![0, 1, 2, 3, 4]);
        let result = input.as_bytes_local(&local);
        assert_eq!(&*result, &[1, 2, 3]);
    }

    #[test]
    fn as_bytes_local_with_out_of_range_buffer() {
        let input = CallInput::SharedBuffer(10..20);
        let local = TestLocalContext::new(vec![0, 1, 2]);
        let result = input.as_bytes_local(&local);
        // Out of range returns empty via unwrap_or_default on the Option<Ref>
        assert_eq!(&*result, &[] as &[u8]);
    }

    #[test]
    fn bytes_local_with_bytes_variant() {
        let input = CallInput::Bytes(Bytes::from_static(b"world"));
        let local = TestLocalContext::new(vec![]);
        let result = input.bytes_local(&local);
        assert_eq!(result, Bytes::from_static(b"world"));
    }

    #[test]
    fn bytes_local_with_shared_buffer() {
        let input = CallInput::SharedBuffer(0..3);
        let local = TestLocalContext::new(vec![10, 20, 30, 40]);
        let result = input.bytes_local(&local);
        assert_eq!(result, Bytes::from(vec![10, 20, 30]));
    }

    #[test]
    fn bytes_local_with_out_of_range_buffer() {
        let input = CallInput::SharedBuffer(5..10);
        let local = TestLocalContext::new(vec![0]);
        let result = input.bytes_local(&local);
        assert_eq!(result, Bytes::new());
    }

    #[test]
    fn bytes_local_with_empty_range() {
        let input = CallInput::SharedBuffer(2..2);
        let local = TestLocalContext::new(vec![0, 1, 2, 3]);
        let result = input.bytes_local(&local);
        assert_eq!(result, Bytes::new());
    }
}
