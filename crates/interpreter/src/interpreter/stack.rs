use crate::{
    primitives::{B256, U256},
    InstructionResult,
};
use alloc::vec::Vec;
use core::fmt;

/// EVM interpreter stack limit.
pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stack {
    data: Vec<U256>,
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, x) in self.data.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{x}")?;
        }
        f.write_str("]")
    }
}

impl Default for Stack {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Instantiate a new stack with the [default stack limit][STACK_LIMIT].
    #[inline]
    pub fn new() -> Self {
        Self {
            // Safety: [`Self::push`] assumes that capacity is STACK_LIMIT
            data: Vec::with_capacity(STACK_LIMIT),
        }
    }

    /// Returns the length of the stack in words.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the underlying data of the stack.
    #[inline]
    pub fn data(&self) -> &Vec<U256> {
        &self.data
    }

    /// Removes the topmost element from the stack and returns it, or `StackUnderflow` if it is
    /// empty.
    #[inline]
    pub fn pop(&mut self) -> Result<U256, InstructionResult> {
        self.data.pop().ok_or(InstructionResult::StackUnderflow)
    }

    /// Removes the topmost element from the stack and returns it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline(always)]
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        self.data.pop().unwrap_unchecked()
    }

    /// Peeks the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline(always)]
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        let len = self.data.len();
        self.data.get_unchecked_mut(len - 1)
    }

    /// Pop the topmost value, returning the value and the new topmost value.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    pub unsafe fn pop4_unsafe(&mut self) -> (U256, U256, U256, U256) {
        let pop1 = self.pop_unsafe();
        let pop2 = self.pop_unsafe();
        let pop3 = self.pop_unsafe();
        let pop4 = self.pop_unsafe();

        (pop1, pop2, pop3, pop4)
    }

    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    #[inline(always)]
    pub fn push_b256(&mut self, value: B256) -> Result<(), InstructionResult> {
        self.push(value.into())
    }

    /// Push a new value onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline(always)]
    pub fn push(&mut self, value: U256) -> Result<(), InstructionResult> {
        // allows the compiler to optimize out the `Vec::push` capacity check
        assume!(self.data.capacity() == STACK_LIMIT);
        if self.data.len() == STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline(always)]
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Duplicates the `N`th value from the top of the stack.
    #[inline(always)]
    pub fn dup<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        let len = self.data.len();
        if len < N {
            Err(InstructionResult::StackUnderflow)
        } else if len + 1 > STACK_LIMIT {
            Err(InstructionResult::StackOverflow)
        } else {
            // Safety: check for out of bounds is done above and it makes this safe to do.
            unsafe {
                *self.data.get_unchecked_mut(len) = *self.data.get_unchecked(len - N);
                self.data.set_len(len + 1);
            }
            Ok(())
        }
    }

    /// Swaps the topmost value with the `N`th value from the top.
    #[inline(always)]
    pub fn swap<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        let len = self.data.len();
        if len <= N {
            return Err(InstructionResult::StackUnderflow);
        }
        let last = len - 1;
        self.data.swap(last, last - N);
        Ok(())
    }

    /// Push a slice of bytes of `N` length onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Result<(), InstructionResult> {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }

        let slot;
        // Safety: check above ensures us that we are okay in increment len.
        unsafe {
            self.data.set_len(new_len);
            slot = self.data.get_unchecked_mut(new_len - 1);
        }

        unsafe {
            *slot.as_limbs_mut() = [0u64; 4];
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
        Ok(())
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), InstructionResult> {
        if self.data.len() > no_from_top {
            let len = self.data.len();
            self.data[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }
}
