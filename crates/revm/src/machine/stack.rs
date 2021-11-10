use crate::{alloc::vec::Vec, error::ExitError};
use primitive_types::H256;

pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack {
    data: Vec<H256>,
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
    pub fn data(&self) -> &Vec<H256> {
        &self.data
    }

    #[inline]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop(&mut self) -> Result<H256, ExitError> {
        self.data.pop().ok_or(ExitError::StackUnderflow)
    }

    #[inline(always)]
    /**** SAFETY ********
     * caller is responsible to check length of array
     */
    pub unsafe fn pop_unsafe(&mut self) -> H256 {
        let mut len = self.data.len();
        len -= 1;
        self.data.set_len(len);
        *self.data.get_unchecked(len)
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push(&mut self, value: H256) -> Result<(), ExitError> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(ExitError::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    #[inline]
    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<H256, ExitError> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(ExitError::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn dup<const N: usize>(&mut self) -> Result<(), ExitError> {
        let len = self.data.len();
        if len < N {
            Err(ExitError::StackUnderflow)
        } else if len + 1 > STACK_LIMIT {
            Err(ExitError::StackOverflow)
        } else {
            unsafe {
                let new_len = len + 1;
                self.data.set_len(new_len);
                *self.data.get_unchecked_mut(len) = *self.data.get_unchecked(len - N);
            }
            Ok(())
        }
    }

    #[inline(always)]
    pub fn swap<const N: usize>(&mut self) -> Result<(), ExitError> {
        let len = self.data.len();
        if len <= N {
            return Err(ExitError::StackUnderflow);
        }
        // SAFETY: length is checked before so we are okay to switch bytes in unsafe way.
        unsafe {
            let pa: *mut H256 = self.data.get_unchecked_mut(len - 1);
            let pb: *mut H256 = self.data.get_unchecked_mut(len - 1 - N);
            core::ptr::swap(pa, pb);
        }
        Ok(())
    }

    /// push slice onto memory it is expected to be max 32 bytes and be contains inside H256
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Result<(), ExitError> {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Err(ExitError::StackOverflow);
        }
        unsafe {
            self.data.set_len(new_len);
            let slot = &mut self.data.get_unchecked_mut(new_len - 1).0;
            *slot = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(slice.as_ptr(), slot[32 - N..].as_mut_ptr(), N);
        }
        Ok(())
    }

    #[inline]
    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: H256) -> Result<(), ExitError> {
        if self.data.len() > no_from_top {
            let len = self.data.len();
            self.data[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(ExitError::StackUnderflow)
        }
    }
}
