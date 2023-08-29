use revm_primitives::U256;

use crate::alloc::vec;
use crate::alloc::vec::Vec;
use core::{
    cmp::min,
    ops::{BitAnd, Not},
};

/// A sequential memory shared between calls, which uses
/// a `Vec` for internal representation.
/// A [SharedMemory] instance should always be obtained using
/// the `new` static method to ensure memory safety.
pub struct SharedMemory {
    /// Shared buffer
    data: Vec<u8>,
    /// Memory checkpoints for each depth
    checkpoints: Vec<usize>,
    /// Amount of memory left for assignment
    pub limit: u64,
    /// Raw pointer used for the portion of memory used
    /// by the current context
    current_slice: *mut [u8],
    /// How much memory has been used in the current context
    current_len: usize,
}

impl SharedMemory {
    /// Allocate memory to be shared between calls.
    /// Memory size is estimated using https://2π.com/22/eth-max-mem
    /// using transaction [gas_limit];
    pub fn new(gas_limit: u64, _memory_limit: Option<u64>) -> Self {
        let upper_bound = 4096 * (2_f64 * gas_limit as f64).sqrt() as usize;
        // let max_alloc_size = isize::MAX as usize;
        let max_alloc_size = u32::MAX as usize;
        let size = min(upper_bound, max_alloc_size);

        let mut data = vec![0; size];
        let checkpoints = Vec::with_capacity(1024);
        let current_slice: *mut [u8] = &mut data[..];
        SharedMemory {
            data,
            limit: u64::MAX,
            checkpoints,
            current_slice,
            current_len: 0,
        }
    }
    /// Prepares the shared memory for a new context
    pub fn new_context_memory(&mut self) {
        let base_offset = self.checkpoints.last().unwrap_or(&0);
        let new_checkpoint = base_offset + self.current_len;

        self.checkpoints.push(new_checkpoint);

        self.current_slice = &mut self.data[new_checkpoint..];
        self.current_len = 0;
    }

    /// Prepares the shared memory for returning to the previous context
    pub fn free_context_memory(&mut self) {
        if let Some(old_checkpoint) = self.checkpoints.pop() {
            let last = *self.checkpoints.last().unwrap_or(&0);
            self.current_slice = &mut self.data[last..];
            self.current_len = old_checkpoint - last;
            self.update_limit();
        }
    }

    /// Get the length of the current memory range.
    pub fn len(&self) -> usize {
        self.current_len
    }

    /// Return true if current effective memory range is zero.
    pub fn is_empty(&self) -> bool {
        self.current_len == 0
    }

    /// Return the full memory.
    pub fn data(&self) -> &[u8] {
        self.get_current_slice()
    }

    /// Resize the memory. assume that we already checked if:
    /// - we have enought gas to resize this vector
    /// - we made new_size as multiply of 32
    /// - [new_size] is greater than `self.len()`
    pub fn resize(&mut self, new_size: usize) {
        // extend with zeros
        let range = self.current_len..new_size;

        self.get_current_slice_mut()[range]
            .iter_mut()
            .for_each(|byte| *byte = 0);
        self.current_len = new_size;
        self.update_limit();
    }

    /// Get memory region at given offset. Dont check offset and size
    #[inline(always)]
    pub fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.get_current_slice()[offset..offset + size]
    }

    /// Set memory region at given offset
    ///
    /// # Safety
    /// The caller is responsible for checking the offset and value
    #[inline(always)]
    pub unsafe fn set_byte(&mut self, index: usize, byte: u8) {
        self.get_current_slice_mut()[index] = byte;
    }

    #[inline(always)]
    pub fn set_u256(&mut self, index: usize, value: U256) {
        self.get_current_slice_mut()[index..index + 32]
            .copy_from_slice(&value.to_be_bytes::<{ U256::BYTES }>());
    }

    /// Set memory region at given offset. The offset and value are already checked
    #[inline(always)]
    pub fn set(&mut self, offset: usize, value: &[u8]) {
        if !value.is_empty() {
            self.get_current_slice_mut()[offset..(value.len() + offset)].copy_from_slice(value);
        }
    }

    /// Set memory from data. Our memory offset+len is expected to be correct but we
    /// are doing bound checks on data/data_offeset/len and zeroing parts that is not copied.
    #[inline(always)]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        if data_offset >= data.len() {
            // nulify all memory slots
            for i in &mut self.get_current_slice_mut()[memory_offset..memory_offset + len] {
                *i = 0;
            }
            return;
        }
        let data_end = min(data_offset + len, data.len());
        let memory_data_end = memory_offset + (data_end - data_offset);
        self.get_current_slice_mut()[memory_offset..memory_data_end]
            .copy_from_slice(&data[data_offset..data_end]);

        // nulify rest of memory slots
        // Safety: Memory is assumed to be valid. And it is commented where that assumption is made
        for i in &mut self.get_current_slice_mut()[memory_data_end..memory_offset + len] {
            *i = 0;
        }
    }

    /// In memory copy given a src, dst, and length
    ///
    /// # Safety
    /// The caller is responsible to check that we resized memory properly.
    #[inline(always)]
    pub fn copy(&mut self, dst: usize, src: usize, length: usize) {
        self.get_current_slice_mut()
            .copy_within(src..src + length, dst);
    }

    /// Get a refernce to the memory of the current context
    #[inline(always)]
    fn get_current_slice(&self) -> &[u8] {
        // Safety: if it is a valid pointer to a slice of `self.data`
        unsafe { &*self.current_slice }
    }

    /// Get a mutable refernce to the memory of the current context
    #[inline(always)]
    fn get_current_slice_mut(&mut self) -> &mut [u8] {
        // Safety: it is a valid pointer to a slice of `self.data`
        unsafe { &mut *self.current_slice }
    }

    /// Update the amount of memory left for usage
    #[inline(always)]
    fn update_limit(&mut self) {
        self.limit =
            (self.data.len() - *self.checkpoints.last().unwrap_or(&0) - self.current_len) as u64;
    }
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}
