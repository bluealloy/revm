use crate::{alloc::vec::Vec, error::ExitError};
use primitive_types::H256;

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack {
    data: Vec<H256>,
    limit: usize,
}

impl Stack {
    /// Create a new stack with given limit.
    pub fn new(limit: usize) -> Self {
        Self {
            data: Vec::with_capacity(1024),
            limit,
        }
    }

    #[inline]
    /// Stack limit.
    pub fn limit(&self) -> usize {
        self.limit
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
        self.data.get_unchecked(len).clone()
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push(&mut self, value: H256) -> Result<(), ExitError> {
        if self.data.len() + 1 > self.limit {
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
