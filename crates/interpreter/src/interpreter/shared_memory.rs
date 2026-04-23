use super::MemoryTr;
use crate::InstructionResult;
use context_interface::cfg::GasParams;
use core::{cmp::min, fmt, ops::Range};
use primitives::{hex, B256, U256};
use std::vec::Vec;

/// EVM memory.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Memory {
    /// The underlying buffer.
    buffer: Vec<u8>,
    /// Memory limit. See [`Cfg`](context_interface::Cfg).
    #[cfg(feature = "memory_limit")]
    memory_limit: u64,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("len", &self.buffer.len())
            .field("data", &hex::encode(&self.buffer))
            .finish()
    }
}

impl Default for Memory {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTr for Memory {
    fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        self.set_data(memory_offset, data_offset, len, data);
    }

    fn set(&mut self, memory_offset: usize, data: &[u8]) {
        self.set(memory_offset, data);
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn copy(&mut self, destination: usize, source: usize, len: usize) {
        self.copy(destination, source, len);
    }

    fn slice(&self, range: Range<usize>) -> &[u8] {
        self.slice(range)
    }

    fn resize(&mut self, new_size: usize) -> bool {
        self.resize(new_size);
        true
    }

    /// Returns `true` if the `new_size` for the current context memory will
    /// make the shared buffer length exceed the `memory_limit`.
    #[cfg(feature = "memory_limit")]
    #[inline]
    fn limit_reached(&self, offset: usize, len: usize) -> bool {
        offset.saturating_add(len) as u64 > self.memory_limit
    }
}

impl Memory {
    /// Creates a new memory instance.
    ///
    /// The default initial capacity is 4KiB.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(4 * 1024) // from evmone
    }

    /// Creates a new memory instance with a given `capacity`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            #[cfg(feature = "memory_limit")]
            memory_limit: u64::MAX,
        }
    }

    /// Creates a new memory instance with `memory_limit` as upper bound for allocation size.
    ///
    /// The default initial capacity is 4KiB.
    #[cfg(feature = "memory_limit")]
    #[inline]
    pub fn new_with_memory_limit(memory_limit: u64) -> Self {
        Self {
            memory_limit,
            ..Self::new()
        }
    }

    /// Sets the memory limit in bytes.
    #[inline]
    pub const fn set_memory_limit(&mut self, limit: u64) {
        #[cfg(feature = "memory_limit")]
        {
            self.memory_limit = limit;
        }
        // for clippy.
        let _ = limit;
    }

    /// Returns the length of the memory.
    #[inline]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the memory.
    #[inline]
    pub const fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the memory is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Resizes the memory in-place so that `len` is equal to `new_size`.
    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        self.buffer.resize(new_size, 0);
    }

    /// Clears the memory, setting the length to 0 without deallocating.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns a byte slice of the memory region at the given range.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an out-of-bounds range triggers undefined
    /// behavior. Callers must ensure that the range is within the bounds of the memory.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice(&self, range: Range<usize>) -> &[u8] {
        match self.buffer.get(range.clone()) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: {range:?}; len: {}", self.len()),
        }
    }

    /// Returns a byte slice of the memory region at the given offset and size.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_len(&self, offset: usize, size: usize) -> &[u8] {
        self.slice(offset..offset + size)
    }

    /// Returns a mutable byte slice of the memory region at the given offset and size.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn slice_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        match self.buffer.get_mut(offset..offset + size) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: {offset}..{}", offset + size),
        }
    }

    /// Returns the byte at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn get_byte(&self, offset: usize) -> u8 {
        self.slice_len(offset, 1)[0]
    }

    /// Returns a 32-byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn get_word(&self, offset: usize) -> B256 {
        self.slice_len(offset, 32).try_into().unwrap()
    }

    /// Returns a U256 of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn get_u256(&self, offset: usize) -> U256 {
        self.get_word(offset).into()
    }

    /// Sets the `byte` at the given `index`.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_byte(&mut self, offset: usize, byte: u8) {
        self.set(offset, &[byte]);
    }

    /// Sets the given 32-byte `value` to the memory region at the given `offset`.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_word(&mut self, offset: usize, value: &B256) {
        self.set(offset, &value[..]);
    }

    /// Sets the given U256 `value` to the memory region at the given `offset`.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_u256(&mut self, offset: usize, value: U256) {
        self.set(offset, &value.to_be_bytes::<32>());
    }

    /// Set memory region at given `offset`.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&mut self, offset: usize, value: &[u8]) {
        if !value.is_empty() {
            self.slice_mut(offset, value.len()).copy_from_slice(value);
        }
    }

    /// Set memory from data. Our memory offset+len is expected to be correct but we
    /// are doing bound checks on data/data_offeset/len and zeroing parts that is not copied.
    ///
    /// # Panics
    ///
    /// Panics if memory is out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        unsafe { set_data(&mut self.buffer, data, memory_offset, data_offset, len) };
    }

    /// Copies elements from one part of the memory to another part of itself.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
        self.buffer.copy_within(src..src + len, dst);
    }
}

/// Copies data from src to dst taking into account the offsets and len.
///
/// If src does not have enough data, it nullifies the rest of dst that is not copied.
///
/// # Safety
///
/// Assumes that dst has enough space to copy the data.
/// Assumes that src has enough data to copy.
/// Assumes that dst_offset and src_offset are in bounds.
/// Assumes that dst and src are valid.
/// Assumes that dst and src do not overlap.
unsafe fn set_data(dst: &mut [u8], src: &[u8], dst_offset: usize, src_offset: usize, len: usize) {
    if len == 0 {
        return;
    }
    if src_offset >= src.len() {
        // Nullify all memory slots
        dst.get_mut(dst_offset..dst_offset + len).unwrap().fill(0);
        return;
    }
    let src_end = min(src_offset + len, src.len());
    let src_len = src_end - src_offset;
    debug_assert!(src_offset < src.len() && src_end <= src.len());
    let data = unsafe { src.get_unchecked(src_offset..src_end) };
    unsafe {
        dst.get_unchecked_mut(dst_offset..dst_offset + src_len)
            .copy_from_slice(data)
    };

    // Nullify rest of memory slots
    // SAFETY: Memory is assumed to be valid, and it is commented where this assumption is made.
    unsafe {
        dst.get_unchecked_mut(dst_offset + src_len..dst_offset + len)
            .fill(0)
    };
}

/// Returns number of words what would fit to provided number of bytes,
/// i.e. it rounds up the number bytes to number of words.
#[inline]
pub const fn num_words(len: usize) -> usize {
    len.div_ceil(32)
}

/// Performs EVM memory resize.
#[inline]
pub fn resize_memory<Memory: MemoryTr>(
    gas: &mut crate::Gas,
    memory: &mut Memory,
    gas_table: &GasParams,
    offset: usize,
    len: usize,
) -> Result<(), InstructionResult> {
    #[cfg(feature = "memory_limit")]
    if memory.limit_reached(offset, len) {
        return Err(InstructionResult::MemoryLimitOOG);
    }

    let new_num_words = num_words(offset.saturating_add(len));
    if new_num_words > gas.memory().words_num {
        return resize_memory_cold(gas, memory, gas_table, new_num_words);
    }

    Ok(())
}

#[cold]
#[inline(never)]
fn resize_memory_cold<Memory: MemoryTr>(
    gas: &mut crate::Gas,
    memory: &mut Memory,
    gas_table: &GasParams,
    new_num_words: usize,
) -> Result<(), InstructionResult> {
    let cost = gas_table.memory_cost(new_num_words);
    let cost = unsafe {
        gas.memory_mut()
            .set_words_num(new_num_words, cost)
            .unwrap_unchecked()
    };

    if !gas.record_regular_cost(cost) {
        return Err(InstructionResult::MemoryOOG);
    }
    memory.resize(new_num_words * 32);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_words() {
        assert_eq!(num_words(0), 0);
        assert_eq!(num_words(1), 1);
        assert_eq!(num_words(31), 1);
        assert_eq!(num_words(32), 1);
        assert_eq!(num_words(33), 2);
        assert_eq!(num_words(63), 2);
        assert_eq!(num_words(64), 2);
        assert_eq!(num_words(65), 3);
        assert_eq!(num_words(usize::MAX - 31), usize::MAX / 32);
        assert_eq!(num_words(usize::MAX - 30), (usize::MAX / 32) + 1);
        assert_eq!(num_words(usize::MAX), (usize::MAX / 32) + 1);
    }

    #[test]
    fn resize() {
        let mut m = Memory::new();
        m.resize(32);
        assert_eq!(m.len(), 32);
        assert_eq!(&m.buffer[..32], &[0_u8; 32]);

        m.resize(96);
        assert_eq!(m.len(), 96);
        assert_eq!(&m.buffer[..96], &[0_u8; 96]);

        m.clear();
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
    }
}
