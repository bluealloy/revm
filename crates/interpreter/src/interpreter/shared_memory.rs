use revm_primitives::U256;

use crate::alloc::vec;
use crate::alloc::{boxed::Box, slice, vec::Vec};
use core::{
    cmp::min,
    fmt,
    ops::{BitAnd, Not},
};

/// A sequential memory shared between calls, which uses
/// a `Vec` for internal representation.
/// A [SharedMemory] instance should always be obtained using
/// the `new` static method to ensure memory safety.
pub struct SharedMemory {
    /// Shared buffer
    data: Box<[u8]>,
    /// Memory checkpoints for each depth
    checkpoints: Vec<usize>,
    /// Raw pointer used for the portion of memory used
    /// by the current context
    current_ptr: *mut u8,
    /// How much memory has been used in the current context
    current_len: usize,
    /// Amount of memory left for assignment
    pub limit: usize,
}

impl fmt::Debug for SharedMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedMemory")
            .field(
                "current_slice",
                &crate::primitives::hex::encode(self.current_slice()),
            )
            .finish()
    }
}

impl SharedMemory {
    /// Calculates memory allocation upper bound using
    /// https://2π.com/22/eth-max-mem
    #[inline]
    pub fn calculate_upper_bound(gas_limit: u64) -> u64 {
        4096 * sqrt(2u64.checked_mul(gas_limit).unwrap_or(u64::MAX))
    }

    /// Allocate memory to be shared between calls.
    /// Memory size is estimated using https://2π.com/22/eth-max-mem
    /// which depends on transaction [gas_limit].
    /// Maximum allocation size is 2^32 - 1 bytes;
    pub fn new(gas_limit: u64) -> Self {
        Self::new_with_memory_limit(gas_limit, (u32::MAX - 1) as u64)
    }

    /// Allocate memory to be shared between calls.
    /// Memory size is estimated using https://2π.com/22/eth-max-mem
    /// which depends on transaction [gas_limit].
    /// Uses [memory_limit] as maximum allocation size
    pub fn new_with_memory_limit(gas_limit: u64, memory_limit: u64) -> Self {
        let limit = min(
            Self::calculate_upper_bound(gas_limit) as usize,
            memory_limit as usize,
        );

        let mut data = vec![0; limit].into_boxed_slice();
        let current_slice = data.as_mut_ptr();
        Self {
            data,
            checkpoints: Vec::with_capacity(32),
            current_ptr: current_slice,
            current_len: 0,
            limit,
        }
    }

    /// Prepares the shared memory for a new context
    pub fn new_context_memory(&mut self) {
        let base_offset = self.last_checkpoint();
        let new_checkpoint = base_offset + self.current_len;

        self.checkpoints.push(new_checkpoint);

        self.current_ptr = self.data[new_checkpoint..].as_mut_ptr();
        self.current_len = 0;
    }

    /// Prepares the shared memory for returning to the previous context
    pub fn free_context_memory(&mut self) {
        if let Some(old_checkpoint) = self.checkpoints.pop() {
            let last_checkpoint = self.last_checkpoint();
            self.current_ptr = self.data[last_checkpoint..].as_mut_ptr();
            self.current_len = old_checkpoint - last_checkpoint;
            self.update_limit();
        }
    }

    /// Get the length of the current memory range.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.current_len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.current_len == 0
    }

    /// Resize the memory. assume that we already checked if:
    /// - we have enough gas to resize this vector
    /// - we made new_size as multiply of 32
    /// - [new_size] is greater than `self.len()`
    #[inline(always)]
    pub fn resize(&mut self, new_size: usize) {
        // extend with zeros
        let range = self.current_len..new_size;

        for byte in self.current_slice_mut()[range].iter_mut() {
            *byte = 0;
        }

        self.current_len = new_size;
        self.update_limit();
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice(&self, offset: usize, size: usize) -> &[u8] {
        match self.current_slice().get(offset..offset + size) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: {offset}..{size}; len: {}", self.len()),
        }
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// Panics on out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        let len = self.current_len;
        match self.current_slice_mut().get_mut(offset..offset + size) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: {offset}..{size}; len: {}", len),
        }
    }

    /// Sets the `byte` at the given `index`.
    ///
    /// Panics when `index` is out of bounds.
    #[inline(always)]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_byte(&mut self, index: usize, byte: u8) {
        match self.current_slice_mut().get_mut(index) {
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
        self.current_slice_mut().copy_within(src..src + len, dst);
    }

    /// Get a reference to the memory of the current context
    #[inline(always)]
    fn current_slice(&self) -> &[u8] {
        // Safety: if it is a valid pointer to a slice of `self.data`
        unsafe { slice::from_raw_parts(self.current_ptr, self.limit) }
    }

    /// Get a mutable reference to the memory of the current context
    #[inline(always)]
    fn current_slice_mut(&mut self) -> &mut [u8] {
        // Safety: it is a valid pointer to a slice of `self.data`
        unsafe { slice::from_raw_parts_mut(self.current_ptr, self.limit) }
    }

    /// Update the amount of memory left for usage
    #[inline(always)]
    fn update_limit(&mut self) {
        self.limit = self.data.len() - self.last_checkpoint() - self.current_len;
    }

    /// Get the last memory checkpoint
    #[inline(always)]
    fn last_checkpoint(&self) -> usize {
        *self.checkpoints.last().unwrap_or(&0)
    }
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}

/// Basic sqrt function using Babylonian method
fn sqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n / 2;
    let mut y = (x + n / x) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::next_multiple_of_32;

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
