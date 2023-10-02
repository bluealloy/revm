use crate::primitives::U256;
use alloc::vec::Vec;
use core::{
    cmp::min,
    fmt,
    ops::{BitAnd, Not},
};

/// A sequential memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Memory {
    data: Vec<u8>,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("data", &crate::primitives::hex::encode(&self.data))
            .finish()
    }
}

impl Default for Memory {
    #[inline]
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(4 * 1024), // took it from evmone
        }
    }
}

impl Memory {
    /// Create a new memory with the given limit.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(4 * 1024), // took it from evmone
        }
    }

    #[deprecated = "Use `len` instead"]
    #[doc(hidden)]
    #[inline]
    pub fn effective_len(&self) -> usize {
        self.len()
    }

    /// Returns the length of the current memory range.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if current memory range length is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a reference to the full memory.
    #[inline]
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Consumes the type and returns the full memory.
    #[inline]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// Shrinks the capacity of the data buffer as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    /// Resizes the stack in-place so that then length is equal to `new_size`.
    ///
    /// `new_size` should be a multiple of 32.
    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice(&self, offset: usize, size: usize) -> &[u8] {
        match self.data.get(offset..offset + size) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: {offset}..{size}; len: {}", self.len()),
        }
    }

    #[deprecated = "use `slice` instead"]
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        self.slice(offset, size)
    }

    /// Returns a mutable byte slice of the memory region at the given offset.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        let _len = self.len();
        match self.data.get_mut(offset..offset + size) {
            Some(slice) => slice,
            None => debug_unreachable!("slice_mut OOB: {offset}..{size}; len: {_len}"),
        }
    }

    /// Sets the `byte` at the given `index`.
    ///
    /// Panics when `index` is out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_byte(&mut self, index: usize, byte: u8) {
        match self.data.get_mut(index) {
            Some(b) => *b = byte,
            None => debug_unreachable!("set_byte OOB: {index}; len: {}", self.len()),
        }
    }

    /// Sets the given `value` to the memory region at the given `offset`.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_u256(&mut self, offset: usize, value: U256) {
        self.set(offset, &value.to_be_bytes::<32>());
    }

    /// Set memory region at given `offset`.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&mut self, offset: usize, value: &[u8]) {
        if !value.is_empty() {
            self.slice_mut(offset, value.len()).copy_from_slice(value);
        }
    }

    /// Set memory from data. Our memory offset+len is expected to be correct but we
    /// are doing bound checks on data/data_offeset/len and zeroing parts that is not copied.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        if data_offset >= data.len() {
            // nullify all memory slots
            self.slice_mut(memory_offset, len).fill(0);
            return;
        }
        let data_end = min(data_offset + len, data.len());
        let data_len = data_end - data_offset;
        debug_assert!(data_offset < data.len() && data_end <= data.len());
        let data = unsafe { data.get_unchecked(data_offset..data_end) };
        self.slice_mut(memory_offset, data_len)
            .copy_from_slice(data);

        // nullify rest of memory slots
        // Safety: Memory is assumed to be valid. And it is commented where that assumption is made
        self.slice_mut(memory_offset + data_len, len - data_len)
            .fill(0);
    }

    /// Copies elements from one part of the memory to another part of itself.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
        self.data.copy_within(src..src + len, dst);
    }
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}

#[cfg(test)]
mod tests {
    use super::next_multiple_of_32;
    use crate::Memory;

    #[test]
    fn test_copy() {
        // Create a sample memory instance
        let mut memory = Memory::new();

        // Set up initial memory data
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        memory.resize(data.len());
        memory.set_data(0, 0, data.len(), &data);

        // Perform a copy operation
        memory.copy(5, 0, 4);

        // Verify the copied data
        let copied_data = memory.slice(5, 4);
        assert_eq!(copied_data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_next_multiple_of_32() {
        // next_multiple_of_32 returns x when it is a multiple of 32
        for i in 0..32 {
            let x = i * 32;
            assert_eq!(Some(x), next_multiple_of_32(x));
        }

        // next_multiple_of_32 rounds up to the nearest multiple of 32 when `x % 32 != 0`
        for x in 0..1024 {
            if x % 32 == 0 {
                continue;
            }
            let next_multiple = x + 32 - (x % 32);
            assert_eq!(Some(next_multiple), next_multiple_of_32(x));
        }
    }
}
