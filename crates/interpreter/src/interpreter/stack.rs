use crate::{
    primitives::{B256, U256},
    InstructionResult,
};
use alloc::vec::Vec;

pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stack {
    data: Vec<U256>,
}

#[cfg(feature = "std")]
impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.data.is_empty() {
            f.write_str("[]")?;
        } else {
            f.write_str("[")?;
            for i in self.data[..self.data.len() - 1].iter() {
                f.write_str(&i.to_string())?;
                f.write_str(", ")?;
            }
            f.write_str(&self.data.last().unwrap().to_string())?;
            f.write_str("]")?;
        }
        Ok(())
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Create a new stack with given limit.
    pub fn new() -> Self {
        Self {
            // Safety: A lot of functions assumes that capacity is STACK_LIMIT
            data: Vec::with_capacity(STACK_LIMIT),
        }
    }

    #[inline]
    /// Stack length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    /// Stack data.
    pub fn data(&self) -> &Vec<U256> {
        &self.data
    }

    #[inline(always)]
    pub fn reduce_one(&mut self) -> Option<InstructionResult> {
        let len = self.data.len();
        if len < 1 {
            return Some(InstructionResult::StackUnderflow);
        }
        unsafe {
            self.data.set_len(len - 1);
        }
        None
    }

    #[inline]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop(&mut self) -> Result<U256, InstructionResult> {
        self.data.pop().ok_or(InstructionResult::StackUnderflow)
    }

    #[inline(always)]
    /// Pops a value from the stack, returning it.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        let mut len = self.data.len();
        len -= 1;
        self.data.set_len(len);
        *self.data.get_unchecked(len)
    }

    #[inline(always)]
    /// Peeks the top of the stack.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        let len = self.data.len();
        self.data.get_unchecked_mut(len - 1)
    }

    #[inline(always)]
    /// Pop the topmost value, returning the value and the new topmost value.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop_top_unsafe(&mut self) -> (U256, &mut U256) {
        let mut len = self.data.len();
        let pop = *self.data.get_unchecked(len - 1);
        len -= 1;
        self.data.set_len(len);

        (pop, self.data.get_unchecked_mut(len - 1))
    }

    #[inline(always)]
    /// Pops 2 values from the stack and returns them, in addition to the new topmost value.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop2_top_unsafe(&mut self) -> (U256, U256, &mut U256) {
        let mut len = self.data.len();
        let pop1 = *self.data.get_unchecked(len - 1);
        len -= 2;
        let pop2 = *self.data.get_unchecked(len);
        self.data.set_len(len);

        (pop1, pop2, self.data.get_unchecked_mut(len - 1))
    }

    #[inline(always)]
    /// Pops 2 values from the stack.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop2_unsafe(&mut self) -> (U256, U256) {
        let mut len = self.data.len();
        len -= 2;
        self.data.set_len(len);
        (
            *self.data.get_unchecked(len + 1),
            *self.data.get_unchecked(len),
        )
    }

    #[inline(always)]
    /// Pops 3 values from the stack.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop3_unsafe(&mut self) -> (U256, U256, U256) {
        let mut len = self.data.len();
        len -= 3;
        self.data.set_len(len);
        (
            *self.data.get_unchecked(len + 2),
            *self.data.get_unchecked(len + 1),
            *self.data.get_unchecked(len),
        )
    }

    #[inline(always)]
    /// Pops 4 values from the stack.
    ///
    /// # Safety
    /// The caller is responsible to check length of array
    pub unsafe fn pop4_unsafe(&mut self) -> (U256, U256, U256, U256) {
        let mut len = self.data.len();
        len -= 4;
        self.data.set_len(len);
        (
            *self.data.get_unchecked(len + 3),
            *self.data.get_unchecked(len + 2),
            *self.data.get_unchecked(len + 1),
            *self.data.get_unchecked(len),
        )
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push_b256(&mut self, value: B256) -> Result<(), InstructionResult> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }
        self.data.push(U256::from_be_bytes(value.0));
        Ok(())
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push(&mut self, value: U256) -> Result<(), InstructionResult> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    #[inline]
    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn dup<const N: usize>(&mut self) -> Option<InstructionResult> {
        let len = self.data.len();
        if len < N {
            Some(InstructionResult::StackUnderflow)
        } else if len + 1 > STACK_LIMIT {
            Some(InstructionResult::StackOverflow)
        } else {
            // Safety: check for out of bounds is done above and it makes this safe to do.
            unsafe {
                *self.data.get_unchecked_mut(len) = *self.data.get_unchecked(len - N);
                self.data.set_len(len + 1);
            }
            None
        }
    }

    #[inline(always)]
    pub fn swap<const N: usize>(&mut self) -> Option<InstructionResult> {
        let len = self.data.len();
        if len <= N {
            return Some(InstructionResult::StackUnderflow);
        }
        // Safety: length is checked before so we are okay to switch bytes in unsafe way.
        unsafe {
            let pa: *mut U256 = self.data.get_unchecked_mut(len - 1);
            let pb: *mut U256 = self.data.get_unchecked_mut(len - 1 - N);
            core::ptr::swap(pa, pb);
        }
        None
    }

    /// push slice onto memory it is expected to be max 32 bytes and be contains inside B256
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Option<InstructionResult> {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Some(InstructionResult::StackOverflow);
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
        None
    }

    #[inline]
    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
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
