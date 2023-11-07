use crate::{
    primitives::{B256, U256},
    InstructionResult,
};
use alloc::vec::Vec;
use core::fmt;

/// EVM interpreter stack limit.
pub const STACK_LIMIT: usize = 1024;

/// EVM stack.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stack {
    /// The underlying data of the stack.
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
            // SAFETY: expansion functions assume that capacity is `STACK_LIMIT`.
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
    #[inline]
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        self.data.pop().unwrap_unchecked()
    }

    /// Peeks the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        let len = self.data.len();
        self.data.get_unchecked_mut(len - 1)
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

    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    #[inline]
    pub fn push_b256(&mut self, value: B256) -> Result<(), InstructionResult> {
        self.push(value.into())
    }

    /// Push a new value onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline]
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
    #[inline]
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.data.len() > no_from_top {
            Ok(self.data[self.data.len() - no_from_top - 1])
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Duplicates the `N`th value from the top of the stack.
    #[inline]
    pub fn dup<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        let len = self.data.len();
        if len < N {
            Err(InstructionResult::StackUnderflow)
        } else if len + 1 > STACK_LIMIT {
            Err(InstructionResult::StackOverflow)
        } else {
            // SAFETY: check for out of bounds is done above and it makes this safe to do.
            unsafe {
                let data = self.data.as_mut_ptr();
                core::ptr::copy_nonoverlapping(data.add(len - N), data.add(len), 1);
                self.data.set_len(len + 1);
            }
            Ok(())
        }
    }

    /// Swaps the topmost value with the `N`th value from the top.
    #[inline]
    pub fn swap<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        let len = self.data.len();
        if len <= N {
            return Err(InstructionResult::StackUnderflow);
        }
        let last = len - 1;
        self.data.swap(last, last - N);
        Ok(())
    }

    /// Pushes an arbitrary length slice of bytes onto the stack, padding the last word with zeros
    /// if necessary.
    #[inline]
    pub fn push_slice(&mut self, slice: &[u8]) -> Result<(), InstructionResult> {
        if slice.is_empty() {
            return Ok(());
        }

        let n_words = (slice.len() + 31) / 32;
        let new_len = self.data.len() + n_words;
        if new_len > STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }

        // SAFETY: length checked above.
        unsafe {
            let dst = self.data.as_mut_ptr().add(self.data.len()).cast::<u64>();
            self.data.set_len(new_len);

            let mut i = 0;

            // write full words
            let words = slice.chunks_exact(32);
            let partial_last_word = words.remainder();
            for word in words {
                // Note: we unroll `U256::from_be_bytes` here to write directly into the buffer,
                // instead of creating a 32 byte array on the stack and then copying it over.
                for l in word.rchunks_exact(8) {
                    dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
                    i += 1;
                }
            }

            if partial_last_word.is_empty() {
                return Ok(());
            }

            // write limbs of partial last word
            let limbs = partial_last_word.rchunks_exact(8);
            let partial_last_limb = limbs.remainder();
            for l in limbs {
                dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
                i += 1;
            }

            // write partial last limb by padding with zeros
            if !partial_last_limb.is_empty() {
                let mut tmp = [0u8; 8];
                tmp[8 - partial_last_limb.len()..].copy_from_slice(partial_last_limb);
                dst.add(i).write(u64::from_be_bytes(tmp));
                i += 1;
            }

            debug_assert_eq!((i + 3) / 4, n_words, "wrote too much");

            // zero out upper bytes of last word
            let m = i % 4; // 32 / 8
            if m != 0 {
                dst.add(i).write_bytes(0, 4 - m);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: impl FnOnce(&mut Stack)) {
        let mut stack = Stack::new();
        // fill capacity with non-zero values
        unsafe {
            stack.data.set_len(STACK_LIMIT);
            stack.data.fill(U256::MAX);
            stack.data.set_len(0);
        }
        f(&mut stack);
    }

    #[test]
    fn push_slices() {
        // no-op
        run(|stack| {
            stack.push_slice(b"").unwrap();
            assert_eq!(stack.data, []);
        });

        // one word
        run(|stack| {
            stack.push_slice(&[42]).unwrap();
            assert_eq!(stack.data, [U256::from(42)]);
        });

        let n = 0x1111_2222_3333_4444_5555_6666_7777_8888_u128;
        run(|stack| {
            stack.push_slice(&n.to_be_bytes()).unwrap();
            assert_eq!(stack.data, [U256::from(n)]);
        });

        // more than one word
        run(|stack| {
            let b = [U256::from(n).to_be_bytes::<32>(); 2].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data, [U256::from(n); 2]);
        });

        run(|stack| {
            let b = [&[0; 32][..], &[42u8]].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data, [U256::ZERO, U256::from(42)]);
        });

        run(|stack| {
            let b = [&[0; 32][..], &n.to_be_bytes()].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data, [U256::ZERO, U256::from(n)]);
        });

        run(|stack| {
            let b = [&[0; 64][..], &n.to_be_bytes()].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data, [U256::ZERO, U256::ZERO, U256::from(n)]);
        });
    }
}
