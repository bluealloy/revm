use crate::{alloc::vec::Vec, Return};
use primitive_types::{H256, U256};

pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub fn reduce_one(&mut self) -> Return {
        let len = self.data.len();
        if len < 1 {
            return Return::StackUnderflow;
        }
        unsafe {
            self.data.set_len(len - 1);
        }
        Return::Continue
    }

    #[inline]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop(&mut self) -> Result<U256, Return> {
        self.data.pop().ok_or(Return::StackUnderflow)
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
    pub fn push_h256(&mut self, value: H256) -> Result<(), Return> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }
        self.data.push(U256::from_big_endian(value.as_ref()));
        Ok(())
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push(&mut self, value: U256) -> Result<(), Return> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }
        self.data.push(value);
        Ok(())
    }

    #[inline]
    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn peek(&self, no_from_top: usize) -> Result<U256, Return> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(Return::StackUnderflow)
        }
    }

    #[inline(always)]
    pub fn dup<const N: usize>(&mut self) -> Return {
        let len = self.data.len();
        if len < N {
            Return::StackUnderflow
        } else if len + 1 > STACK_LIMIT {
            Return::StackOverflow
        } else {
            // Safety: check for out of bounds is done above and it makes this safe to do.
            unsafe {
                *self.data.get_unchecked_mut(len) = *self.data.get_unchecked(len - N);
                self.data.set_len(len + 1);
            }
            Return::Continue
        }
    }

    #[inline(always)]
    pub fn swap<const N: usize>(&mut self) -> Return {
        let len = self.data.len();
        if len <= N {
            return Return::StackUnderflow;
        }
        // Safety: length is checked before so we are okay to switch bytes in unsafe way.
        unsafe {
            let pa: *mut U256 = self.data.get_unchecked_mut(len - 1);
            let pb: *mut U256 = self.data.get_unchecked_mut(len - 1 - N);
            core::ptr::swap(pa, pb);
        }
        Return::Continue
    }

    /// push slice onto memory it is expected to be max 32 bytes and be contains inside H256
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Return {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Return::StackOverflow;
        }

        let slot;
        // Safety: check above ensures us that we are okey in increment len.
        unsafe {
            self.data.set_len(new_len);
            slot = self.data.get_unchecked_mut(new_len - 1);
        }

        slot.0 = [0u64; 4];
        let mut dangling = [0u8; 8];
        if N < 8 {
            dangling[8 - N..].copy_from_slice(slice);
            slot.0[0] = u64::from_be_bytes(dangling);
        } else if N < 16 {
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            if N != 8 {
                dangling[8 * 2 - N..].copy_from_slice(&slice[..N - 8]);
                slot.0[1] = u64::from_be_bytes(dangling);
            }
        } else if N < 24 {
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            slot.0[1] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 16, 8));
            if N != 16 {
                dangling[8 * 3 - N..].copy_from_slice(&slice[..N - 16]);
                slot.0[2] = u64::from_be_bytes(dangling);
            }
        } else {
            // M<32
            slot.0[0] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 8, 8));
            slot.0[1] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 16, 8));
            slot.0[2] = u64::from_be_bytes(*arrayref::array_ref!(slice, N - 24, 8));
            if N == 32 {
                slot.0[3] = u64::from_be_bytes(*arrayref::array_ref!(slice, 0, 8));
            } else if N != 24 {
                dangling[8 * 4 - N..].copy_from_slice(&slice[..N - 24]);
                slot.0[3] = u64::from_be_bytes(dangling);
            }
        }
        Return::Continue
    }

    #[inline]
    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), Return> {
        if self.data.len() > no_from_top {
            let len = self.data.len();
            self.data[len - no_from_top - 1] = val;
            Ok(())
        } else {
            Err(Return::StackUnderflow)
        }
    }
}
