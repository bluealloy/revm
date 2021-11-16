use crate::{alloc::vec::Vec, util, Return};
use primitive_types::{H256, U256};

pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug)]
pub struct Stack {
    data: Vec<U256>,
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
    pub fn data(&self) -> &Vec<U256> {
        &self.data
    }

    #[inline]
    /// Pop a value from the stack. If the stack is already empty, returns the
    /// `StackUnderflow` error.
    pub fn pop(&mut self) -> Result<U256, Return> {
        self.data.pop().ok_or(Return::StackUnderflow)
    }

    #[inline(always)]
    /**** SAFETY ********
     * caller is responsible to check length of array
     */
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        let mut len = self.data.len();
        len -= 1;
        self.data.set_len(len);
        *self.data.get_unchecked(len)
    }

    #[inline]
    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    pub fn push_h256(&mut self, value: H256) -> Result<(), Return> {
        if self.data.len() + 1 > STACK_LIMIT {
            return Err(Return::StackOverflow);
        }
        self.data.push(util::be_to_u256(&value[..]));
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
            unsafe {
                let new_len = len + 1;
                self.data.set_len(new_len);
                *self.data.get_unchecked_mut(len) = *self.data.get_unchecked(len - N);
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
        // SAFETY: length is checked before so we are okay to switch bytes in unsafe way.
        unsafe {
            let pa: *mut U256 = self.data.get_unchecked_mut(len - 1);
            let pb: *mut U256 = self.data.get_unchecked_mut(len - 1 - N);
            core::ptr::swap(pa, pb);
        }
        Return::Continue
    }
    /*
        /// Converts from big endian representation bytes in memory.
    pub fn from_big_endian(slice: &[u8]) -> Self {
        use $crate::byteorder::{ByteOrder, BigEndian};
        assert!($n_words * 8 >= slice.len());

        let mut padded = [0u8; $n_words * 8];
        padded[$n_words * 8 - slice.len() .. $n_words * 8].copy_from_slice(&slice);

        let mut ret = [0; $n_words];
        for i in 0..$n_words {
            ret[$n_words - i - 1] = BigEndian::read_u64(&padded[8 * i..]);
        }

        0 <- 24..32  _______...
        1 <- 16..24  __________
        2 <- 8..16
        3 <- 0..8


        0 <- 24..32  .........
        1 <- 16..24  _____....
        2 <- 8..16
        3 <- 0..8

        $name(ret)
    }
     */

    /// push slice onto memory it is expected to be max 32 bytes and be contains inside H256
    #[inline(always)]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Return {
        let new_len = self.data.len() + 1;
        if new_len > STACK_LIMIT {
            return Return::StackOverflow;
        }

        unsafe {
            self.data.set_len(new_len);
        }
        let slot = self.data.get_mut(new_len - 1).unwrap();
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
