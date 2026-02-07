use crate::InstructionResult;
use core::{fmt, ptr};
use primitives::U256;
use std::{sync::Arc, vec::Vec};

use super::StackTr;

/// EVM interpreter stack limit.
pub const STACK_LIMIT: usize = 1024;

/// EVM stack with [STACK_LIMIT] capacity of words.
///
/// The stack uses a cached raw pointer (`ptr`) to the start of its region
/// for fast access on the hot path, backed by an `Arc<Vec<U256>>` that keeps
/// the memory alive. Multiple stacks can share the same backing `Vec` when
/// created via a [`StackArena`](crate::interpreter::StackArena).
#[derive(Debug)]
pub struct Stack {
    /// Cached pointer to the first slot of this stack's region.
    /// SAFETY: Valid as long as `_backing` is alive; the Vec is never reallocated.
    ptr: *mut U256,
    /// Current number of items on the stack.
    len: usize,
    /// Shared ownership of the backing buffer.
    _backing: Arc<Vec<U256>>,
}

// SAFETY: Stack is Send + Sync because:
// - `ptr` always points into the Vec inside `_backing` (an Arc<Vec<U256>>).
// - The Vec is never reallocated (pre-allocated to capacity).
// - Non-overlapping regions are guaranteed when arena is shared.
unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        let data = self.data();
        for (i, x) in data.iter().enumerate() {
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

impl Clone for Stack {
    fn clone(&self) -> Self {
        let mut new_stack = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(self.ptr, new_stack.ptr, self.len);
            new_stack.len = self.len;
        }
        new_stack
    }
}

impl PartialEq for Stack {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data()
    }
}

impl Eq for Stack {}

impl core::hash::Hash for Stack {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data().hash(state);
    }
}

impl StackTr for Stack {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn data(&self) -> &[U256] {
        self.data()
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    fn popn<const N: usize>(&mut self) -> Option<[U256; N]> {
        if self.len < N {
            return None;
        }
        // SAFETY: Stack length is checked above.
        Some(unsafe { self.popn::<N>() })
    }

    #[inline]
    fn popn_top<const POPN: usize>(&mut self) -> Option<([U256; POPN], &mut U256)> {
        if self.len < POPN + 1 {
            return None;
        }
        // SAFETY: Stack length is checked above.
        Some(unsafe { self.popn_top::<POPN>() })
    }

    #[inline]
    fn exchange(&mut self, n: usize, m: usize) -> bool {
        self.exchange(n, m)
    }

    #[inline]
    fn dup(&mut self, n: usize) -> bool {
        self.dup(n)
    }

    #[inline]
    fn push(&mut self, value: U256) -> bool {
        self.push(value)
    }

    #[inline]
    fn push_slice(&mut self, slice: &[u8]) -> bool {
        self.push_slice_(slice)
    }
}

impl Stack {
    /// Instantiate a new stack with the [default stack limit][STACK_LIMIT].
    #[inline]
    pub fn new() -> Self {
        let backing = Arc::new(std::vec![U256::ZERO; STACK_LIMIT]);
        let ptr = backing.as_ptr().cast_mut();
        Self {
            ptr,
            len: 0,
            _backing: backing,
        }
    }

    /// Instantiate a new stack backed by a shared arena at the given frame offset.
    ///
    /// The arena must have been pre-allocated with at least
    /// `(frame_index + 1) * STACK_LIMIT` elements.
    #[inline]
    pub fn new_with_arena(arena: Arc<Vec<U256>>, frame_index: usize) -> Self {
        debug_assert!(arena.len() >= (frame_index + 1) * STACK_LIMIT);
        let ptr = unsafe { arena.as_ptr().cast_mut().add(frame_index * STACK_LIMIT) };
        Self {
            ptr,
            len: 0,
            _backing: arena,
        }
    }

    /// Reinitialize this stack to use a different frame slot in the arena.
    ///
    /// This avoids dropping and reallocating - just updates the pointer and length.
    /// The arena must have been pre-allocated with at least
    /// `(frame_index + 1) * STACK_LIMIT` elements.
    #[inline]
    pub fn reinit_with_arena(&mut self, arena: Arc<Vec<U256>>, frame_index: usize) {
        debug_assert!(arena.len() >= (frame_index + 1) * STACK_LIMIT);
        self.ptr = unsafe { arena.as_ptr().cast_mut().add(frame_index * STACK_LIMIT) };
        self.len = 0;
        self._backing = arena;
    }

    /// Instantiate a new invalid Stack.
    #[inline]
    pub fn invalid() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            len: 0,
            _backing: Arc::new(Vec::new()),
        }
    }

    /// Returns whether this stack was created via [`Stack::invalid`].
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.ptr.is_null()
    }

    /// Returns the length of the stack in words.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the underlying data buffer as a slice.
    #[inline]
    pub fn data(&self) -> &[U256] {
        unsafe {
            let ptr = self.ptr;
            core::slice::from_raw_parts(ptr, self.len)
        }
    }

    /// Returns a mutable reference to the underlying data buffer as a slice.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [U256] {
        unsafe {
            let ptr = self.ptr;
            core::slice::from_raw_parts_mut(ptr, self.len)
        }
    }

    /// Consumes the stack and returns the underlying data buffer.
    #[inline]
    pub fn into_data(self) -> Vec<U256> {
        // Convert arena slice to owned Vec
        self.data().to_vec()
    }

    /// Removes the topmost element from the stack and returns it, or `StackUnderflow` if it is
    /// empty.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn pop(&mut self) -> Result<U256, InstructionResult> {
        if primitives::hints_util::unlikely(self.len == 0) {
            Err(InstructionResult::StackUnderflow)
        } else {
            unsafe {
                let ptr = self.ptr;
                self.len -= 1;
                Ok(core::ptr::read(ptr.add(self.len)))
            }
        }
    }

    /// Removes the topmost element from the stack and returns it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        assume!(self.len > 0);
        let ptr = self.ptr;
        self.len -= 1;
        core::ptr::read(ptr.add(self.len))
    }

    /// Peeks the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        assume!(self.len > 0);
        let ptr = self.ptr;
        &mut *ptr.add(self.len - 1)
    }

    /// Pops `N` values from the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn popn<const N: usize>(&mut self) -> [U256; N] {
        assume!(self.len >= N);
        core::array::from_fn(|_| unsafe { self.pop_unsafe() })
    }

    /// Pops `N` values from the stack and returns the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub unsafe fn popn_top<const POPN: usize>(&mut self) -> ([U256; POPN], &mut U256) {
        let result = self.popn::<POPN>();
        let top = self.top_unsafe();
        (result, top)
    }

    /// Push a new value onto the stack.
    ///
    /// If it will exceed the stack limit, returns false and leaves the stack
    /// unchanged.
    #[inline]
    #[must_use]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn push(&mut self, value: U256) -> bool {
        if self.len == STACK_LIMIT {
            return false;
        }
        unsafe {
            let ptr = self.ptr;
            core::ptr::write(ptr.add(self.len), value);
            self.len += 1;
        }
        true
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.len > no_from_top {
            unsafe {
                let ptr = self.ptr;
                Ok(ptr::read(ptr.add(self.len - no_from_top - 1)))
            }
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Duplicates the `N`th value from the top of the stack.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    #[inline]
    #[must_use]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn dup(&mut self, n: usize) -> bool {
        assume!(n > 0, "attempted to dup 0");
        if self.len < n || self.len + 1 > STACK_LIMIT {
            false
        } else {
            // SAFETY: Check for out of bounds is done above and it makes this safe to do.
            unsafe {
                let ptr = self.ptr;
                let top = ptr.add(self.len);
                ptr::copy_nonoverlapping(top.sub(n), top, 1);
                self.len += 1;
            }
            true
        }
    }

    /// Swaps the topmost value with the `N`th value from the top.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn swap(&mut self, n: usize) -> bool {
        self.exchange(0, n)
    }

    /// Exchange two values on the stack.
    ///
    /// `n` is the first index, and the second index is calculated as `n + m`.
    ///
    /// # Panics
    ///
    /// Panics if `m` is zero.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn exchange(&mut self, n: usize, m: usize) -> bool {
        assume!(m > 0, "overlapping exchange");
        let n_m_index = n + m;
        if n_m_index >= self.len {
            return false;
        }
        // SAFETY: `n` and `n_m` are checked to be within bounds, and they don't overlap.
        unsafe {
            // Note: `ptr::swap_nonoverlapping` is more efficient than `slice::swap` or `ptr::swap`
            // because it operates under the assumption that the pointers do not overlap,
            // eliminating an intermediate copy,
            // which is a condition we know to be true in this context.
            let ptr = self.ptr;
            let top = ptr.add(self.len - 1);
            core::ptr::swap_nonoverlapping(top.sub(n), top.sub(n_m_index), 1);
        }
        true
    }

    /// Pushes an arbitrary length slice of bytes onto the stack, padding the last word with zeros
    /// if necessary.
    #[inline]
    pub fn push_slice(&mut self, slice: &[u8]) -> Result<(), InstructionResult> {
        if self.push_slice_(slice) {
            Ok(())
        } else {
            Err(InstructionResult::StackOverflow)
        }
    }

    /// Pushes an arbitrary length slice of bytes onto the stack, padding the last word with zeros
    /// if necessary.
    #[inline]
    fn push_slice_(&mut self, slice: &[u8]) -> bool {
        if slice.is_empty() {
            return true;
        }

        let n_words = slice.len().div_ceil(32);
        let new_len = self.len + n_words;
        if new_len > STACK_LIMIT {
            return false;
        }

        // SAFETY: Length checked above.
        unsafe {
            let ptr = self.ptr;
            let dst = ptr.add(self.len).cast::<u64>();
            self.len = new_len;

            let mut i = 0;

            // Write full words
            let words = slice.chunks_exact(32);
            let partial_last_word = words.remainder();
            for word in words {
                // Note: We unroll `U256::from_be_bytes` here to write directly into the buffer,
                // instead of creating a 32 byte array on the stack and then copying it over.
                for l in word.rchunks_exact(8) {
                    dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
                    i += 1;
                }
            }

            if partial_last_word.is_empty() {
                return true;
            }

            // Write limbs of partial last word
            let limbs = partial_last_word.rchunks_exact(8);
            let partial_last_limb = limbs.remainder();
            for l in limbs {
                dst.add(i).write(u64::from_be_bytes(l.try_into().unwrap()));
                i += 1;
            }

            // Write partial last limb by padding with zeros
            if !partial_last_limb.is_empty() {
                let mut tmp = [0u8; 8];
                tmp[8 - partial_last_limb.len()..].copy_from_slice(partial_last_limb);
                dst.add(i).write(u64::from_be_bytes(tmp));
                i += 1;
            }

            debug_assert_eq!(i.div_ceil(4), n_words, "wrote too much");

            // Zero out upper bytes of last word
            let m = i % 4; // 32 / 8
            if m != 0 {
                dst.add(i).write_bytes(0, 4 - m);
            }
        }

        true
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), InstructionResult> {
        if self.len > no_from_top {
            unsafe {
                let ptr = self.ptr;
                ptr::write(ptr.add(self.len - no_from_top - 1), val);
            }
            Ok(())
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Stack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Stack", 1)?;
        state.serialize_field("data", self.data())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Stack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct StackSerde {
            data: Vec<U256>,
        }

        let stack_serde = StackSerde::deserialize(deserializer)?;
        if stack_serde.data.len() > STACK_LIMIT {
            return Err(serde::de::Error::custom(std::format!(
                "stack size exceeds limit: {} > {}",
                stack_serde.data.len(),
                STACK_LIMIT
            )));
        }
        let mut stack = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(stack_serde.data.as_ptr(), stack.ptr, stack_serde.data.len());
            stack.len = stack_serde.data.len();
        }
        Ok(stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: impl FnOnce(&mut Stack)) {
        let mut stack = Stack::new();
        // Fill backing region with non-zero values, then reset len
        unsafe {
            for i in 0..STACK_LIMIT {
                stack.ptr.add(i).write(U256::MAX);
            }
            stack.len = 0;
        }
        f(&mut stack);
    }

    #[test]
    fn push_slices() {
        // No-op
        run(|stack| {
            stack.push_slice(b"").unwrap();
            assert!(stack.is_empty());
        });

        // One word
        run(|stack| {
            stack.push_slice(&[42]).unwrap();
            assert_eq!(stack.data(), [U256::from(42)]);
        });

        let n = 0x1111_2222_3333_4444_5555_6666_7777_8888_u128;
        run(|stack| {
            stack.push_slice(&n.to_be_bytes()).unwrap();
            assert_eq!(stack.data(), [U256::from(n)]);
        });

        // More than one word
        run(|stack| {
            let b = [U256::from(n).to_be_bytes::<32>(); 2].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data(), [U256::from(n); 2]);
        });

        run(|stack| {
            let b = [&[0; 32][..], &[42u8]].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data(), [U256::ZERO, U256::from(42)]);
        });

        run(|stack| {
            let b = [&[0; 32][..], &n.to_be_bytes()].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data(), [U256::ZERO, U256::from(n)]);
        });

        run(|stack| {
            let b = [&[0; 64][..], &n.to_be_bytes()].concat();
            stack.push_slice(&b).unwrap();
            assert_eq!(stack.data(), [U256::ZERO, U256::ZERO, U256::from(n)]);
        });
    }

    #[test]
    fn stack_clone() {
        // Test cloning an empty stack
        let empty_stack = Stack::new();
        let cloned_empty = empty_stack.clone();
        assert_eq!(empty_stack, cloned_empty);
        assert_eq!(cloned_empty.len(), 0);

        // Test cloning a partially filled stack
        let mut partial_stack = Stack::new();
        for i in 0..10 {
            assert!(partial_stack.push(U256::from(i)));
        }
        let mut cloned_partial = partial_stack.clone();
        assert_eq!(partial_stack, cloned_partial);
        assert_eq!(cloned_partial.len(), 10);

        // Test that modifying the clone doesn't affect the original
        assert!(cloned_partial.push(U256::from(100)));
        assert_ne!(partial_stack, cloned_partial);
        assert_eq!(partial_stack.len(), 10);
        assert_eq!(cloned_partial.len(), 11);

        // Test cloning a full stack
        let mut full_stack = Stack::new();
        for i in 0..STACK_LIMIT {
            assert!(full_stack.push(U256::from(i)));
        }
        let mut cloned_full = full_stack.clone();
        assert_eq!(full_stack, cloned_full);
        assert_eq!(cloned_full.len(), STACK_LIMIT);

        // Test push to the full original or cloned stack should return StackOverflow
        assert!(!full_stack.push(U256::from(100)));
        assert!(!cloned_full.push(U256::from(100)));
    }
}
