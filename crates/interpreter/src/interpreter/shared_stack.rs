use crate::{
    primitives::{B256, U256},
    InstructionResult,
};
use alloc::vec::Vec;
use core::fmt;

/// EVM interpreter stack limit.
pub(crate) const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SharedStack {
    /// Shared buffer
    pub buffer: Vec<U256>,
    /// Stack checkpoints for each depth
    pub checkpoints: Vec<usize>,
    /// How much stack has been used in the current context
    pub last_checkpoint: usize,
}

impl fmt::Display for SharedStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, x) in self.context_stack().iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{x}")?;
        }
        f.write_str("]")
    }
}

impl Default for SharedStack {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SharedStack {
    /// Instantiate a new stack with the [default stack limit][STACK_LIMIT].
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(STACK_LIMIT),
            checkpoints: Vec::with_capacity(32),
            last_checkpoint: 0,
        }
    }

    /// Prepares the shared stack for a new context
    #[inline]
    pub fn new_context(&mut self) {
        let buf_len = self.buffer.len();
        self.checkpoints.push(buf_len);
        self.last_checkpoint = buf_len;
    }

    /// Prepares the shared stack for returning to the previous context
    #[inline]
    pub fn free_context(&mut self) {
        if let Some(old_checkpoint) = self.checkpoints.pop() {
            self.last_checkpoint = self.last_checkpoint();
            unsafe { self.buffer.set_len(old_checkpoint) }
        }
    }

    /// Returns the length of the stack in words.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len() - self.last_checkpoint
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the underlying data of the current context stack.
    #[inline]
    pub fn data(&self) -> &[U256] {
        self.context_stack()
    }

    /// Removes the topmost element from the stack and returns it, or `StackUnderflow` if it is
    /// empty.
    #[inline]
    pub fn pop(&mut self) -> Result<U256, InstructionResult> {
        if self.is_empty() {
            Err(InstructionResult::StackUnderflow)
        } else {
            Ok(unsafe { self.buffer.pop().unwrap_unchecked() })
        }
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.len() > no_from_top {
            Ok(unsafe {
                *self
                    .buffer
                    .get_unchecked(self.buffer.len() - no_from_top - 1)
            })
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Peeks the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        let len = self.buffer.len();
        self.buffer.get_unchecked_mut(len - 1)
    }

    /// Removes the topmost element from the stack and returns it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        self.buffer.pop().unwrap_unchecked()
    }

    /// Pop the topmost value, returning the value and the new topmost value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop_top_unsafe(&mut self) -> (U256, &mut U256) {
        let pop = self.pop_unsafe();
        let top = self.top_unsafe();
        (pop, top)
    }

    /// Pops 2 values from the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop2_unsafe(&mut self) -> (U256, U256) {
        let pop1 = self.pop_unsafe();
        let pop2 = self.pop_unsafe();
        (pop1, pop2)
    }

    /// Pops 2 values from the stack and returns them, in addition to the new topmost value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop2_top_unsafe(&mut self) -> (U256, U256, &mut U256) {
        let pop1 = self.pop_unsafe();
        let pop2 = self.pop_unsafe();
        let top = self.top_unsafe();

        (pop1, pop2, top)
    }

    /// Pops 3 values from the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop3_unsafe(&mut self) -> (U256, U256, U256) {
        let pop1 = self.pop_unsafe();
        let pop2 = self.pop_unsafe();
        let pop3 = self.pop_unsafe();

        (pop1, pop2, pop3)
    }

    /// Pops 4 values from the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop4_unsafe(&mut self) -> (U256, U256, U256, U256) {
        let pop1 = self.pop_unsafe();
        let pop2 = self.pop_unsafe();
        let pop3 = self.pop_unsafe();
        let pop4 = self.pop_unsafe();

        (pop1, pop2, pop3, pop4)
    }

    /// Push a new value onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline]
    pub fn push(&mut self, value: U256) -> Result<(), InstructionResult> {
        // allows the compiler to optimize out the `Vec::push` capacity check
        if self.len() == STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }
        self.buffer.push(value);
        Ok(())
    }

    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    #[inline]
    pub fn push_b256(&mut self, value: B256) -> Result<(), InstructionResult> {
        self.push(value.into())
    }

    /// Duplicates the `N`th value from the top of the stack, with `N` >= 1
    #[inline]
    pub fn dup<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        let current_len = self.len();
        if current_len < N {
            Err(InstructionResult::StackUnderflow)
        } else if current_len >= STACK_LIMIT {
            Err(InstructionResult::StackOverflow)
        } else {
            let buf_len = self.buffer.len();
            // Safety: check for out of bounds is done above and it makes this safe to do.
            let val = unsafe { *self.buffer.get_unchecked(buf_len - N) };
            self.buffer.push(val);
            Ok(())
        }
    }

    /// Swaps the topmost value with the `N`th value from the top.
    #[inline]
    pub fn swap<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        if self.len() <= N {
            return Err(InstructionResult::StackUnderflow);
        }
        let last = self.buffer.len() - 1;
        self.buffer.swap(last, last - N);
        Ok(())
    }

    /// Push a slice of bytes of `N` length onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Result<(), InstructionResult> {
        if self.len() >= STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }

        let mut slot = U256::ZERO;

        unsafe {
            let mut dangling = [0u8; 8];
            if N < 8 {
                dangling[8 - N..].copy_from_slice(slice);
                slot.as_limbs_mut()[0] = u64::from_be_bytes(dangling);
            } else if N < 16 {
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                if N != 8 {
                    dangling[8 * 2 - N..].copy_from_slice(&slice[..N - 8]);
                    slot.as_limbs_mut()[1] = u64::from_be_bytes(dangling);
                }
            } else if N < 24 {
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                slot.as_limbs_mut()[1] =
                    u64::from_be_bytes(slice[N - 16..N - 8].try_into().expect("Infallible"));
                if N != 16 {
                    dangling[8 * 3 - N..].copy_from_slice(&slice[..N - 16]);
                    slot.as_limbs_mut()[2] = u64::from_be_bytes(dangling);
                }
            } else {
                // M<32
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                slot.as_limbs_mut()[1] =
                    u64::from_be_bytes(slice[N - 16..N - 8].try_into().expect("Infallible"));
                slot.as_limbs_mut()[2] =
                    u64::from_be_bytes(slice[N - 24..N - 16].try_into().expect("Infallible"));
                if N == 32 {
                    slot.as_limbs_mut()[3] =
                        u64::from_be_bytes(slice[..N - 24].try_into().expect("Infallible"));
                } else if N != 24 {
                    dangling[8 * 4 - N..].copy_from_slice(&slice[..N - 24]);
                    slot.as_limbs_mut()[3] = u64::from_be_bytes(dangling);
                }
            }
        }

        self.buffer.push(slot);
        Ok(())
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), InstructionResult> {
        if self.len() > no_from_top {
            let buf_len = self.buffer.len();
            unsafe { *self.buffer.get_unchecked_mut(buf_len - no_from_top - 1) = val };
            Ok(())
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Get a reference to the stack of the current context
    #[inline]
    fn context_stack(&self) -> &[U256] {
        unsafe { self.buffer.get_unchecked(self.last_checkpoint..self.len()) }
    }

    /// Get the last stack checkpoint
    #[inline]
    fn last_checkpoint(&self) -> usize {
        self.checkpoints.last().cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_free_context_stack() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();
        assert_eq!(shared_stack.last_checkpoint(), 0);
        assert_eq!(shared_stack.checkpoints.len(), 1);

        unsafe { shared_stack.buffer.set_len(16) }
        shared_stack.new_context();
        assert_eq!(shared_stack.last_checkpoint(), 16);
        assert_eq!(shared_stack.checkpoints.len(), 2);
        assert_eq!(shared_stack.len(), 0);

        unsafe { shared_stack.buffer.set_len(48) }
        shared_stack.new_context();
        assert_eq!(shared_stack.last_checkpoint(), 48);
        assert_eq!(shared_stack.checkpoints.len(), 3);
        assert_eq!(shared_stack.len(), 0);
        assert_eq!(shared_stack.buffer.len(), 48);

        // free contexts
        shared_stack.free_context();
        assert_eq!(shared_stack.last_checkpoint(), 16);
        assert_eq!(shared_stack.buffer.len(), 48);
        assert_eq!(shared_stack.checkpoints.len(), 2);
        assert_eq!(shared_stack.len(), 32);

        shared_stack.free_context();
        assert_eq!(shared_stack.last_checkpoint(), 0);
        assert_eq!(shared_stack.checkpoints.len(), 1);
        assert_eq!(shared_stack.len(), 16);
        assert_eq!(shared_stack.buffer.len(), 16);

        shared_stack.free_context();
        assert_eq!(shared_stack.last_checkpoint(), 0);
        assert_eq!(shared_stack.checkpoints.len(), 0);
        assert_eq!(shared_stack.len(), 0);
        assert_eq!(shared_stack.buffer.len(), 0);
    }

    #[test]
    fn pop() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        shared_stack.buffer.push(U256::from(1));
        assert_eq!(shared_stack.pop(), Ok(U256::from(1)));
    }

    #[test]
    fn pop_underflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(shared_stack.pop(), Err(InstructionResult::StackUnderflow));

        // pop underflow in a new empty context
        unsafe { shared_stack.buffer.set_len(STACK_LIMIT / 2) }
        shared_stack.new_context();
        assert_eq!(shared_stack.pop(), Err(InstructionResult::StackUnderflow));
    }

    #[test]
    fn push() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        let one = U256::from(1);
        let res = shared_stack.push(one);
        assert_eq!(res, Ok(()));
        assert_eq!(shared_stack.buffer[0], one);

        let two = U256::from(2);
        let res = shared_stack.push(two);
        assert_eq!(res, Ok(()));
        assert_eq!(shared_stack.buffer[1], two);
    }

    #[test]
    fn push_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        for _ in 1..=STACK_LIMIT {
            assert_eq!(shared_stack.push(U256::ZERO), Ok(()));
        }

        assert_eq!(shared_stack.buffer.len(), STACK_LIMIT);
        assert_eq!(shared_stack.len(), STACK_LIMIT);
        assert_eq!(
            shared_stack.push(U256::ZERO),
            Err(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn push_slice() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(shared_stack.push_slice::<1>(&[1]), Ok(()));
        assert_eq!(shared_stack.buffer[0], U256::from(1));
        assert_eq!(shared_stack.len(), 1);
    }

    #[test]
    fn push_slice_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        for _ in 1..=STACK_LIMIT {
            assert_eq!(shared_stack.push_slice::<1>(&[0]), Ok(()));
        }
        assert_eq!(shared_stack.len(), STACK_LIMIT);
        assert_eq!(shared_stack.buffer.len(), STACK_LIMIT);
        assert_eq!(
            shared_stack.push_slice::<1>(&[0]),
            Err(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn dup() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackUnderflow)
        );

        assert_eq!(shared_stack.push(U256::from(1)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(2)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(3)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(4)), Ok(()));
        assert_eq!(shared_stack.len(), 4);
        assert_eq!(shared_stack.buffer[3], U256::from(4));

        assert_eq!(shared_stack.dup::<1>(), Ok(()));
        assert_eq!(shared_stack.len(), 5);
        assert_eq!(shared_stack.buffer[4], U256::from(4));

        assert_eq!(shared_stack.dup::<3>(), Ok(()));
        assert_eq!(shared_stack.len(), 6);
        assert_eq!(shared_stack.buffer[5], U256::from(3));
    }

    #[test]
    fn dup_stack_underflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackUnderflow)
        );

        unsafe { shared_stack.buffer.set_len(STACK_LIMIT / 2) }
        shared_stack.new_context();

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackUnderflow)
        );
    }

    #[test]
    fn dup_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        shared_stack.buffer.push(U256::ZERO);

        for _ in 2..=STACK_LIMIT {
            assert_eq!(shared_stack.dup::<1>(), Ok(()))
        }

        assert_eq!(shared_stack.buffer.len(), STACK_LIMIT);
        assert_eq!(shared_stack.len(), STACK_LIMIT);

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackOverflow)
        );
    }
}
