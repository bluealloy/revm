use super::MemoryTr;
use core::{
    cell::{Ref, RefCell, RefMut},
    cmp::min,
    fmt,
    ops::Range,
};
use primitives::{hex, B256, U256};
use std::{rc::Rc, vec::Vec};

trait RefcellExt<T> {
    fn dbg_borrow(&self) -> Ref<'_, T>;
    fn dbg_borrow_mut(&self) -> RefMut<'_, T>;
}

impl<T> RefcellExt<T> for RefCell<T> {
    #[inline]
    fn dbg_borrow(&self) -> Ref<'_, T> {
        match self.try_borrow() {
            Ok(b) => b,
            Err(e) => debug_unreachable!("{e}"),
        }
    }

    #[inline]
    fn dbg_borrow_mut(&self) -> RefMut<'_, T> {
        match self.try_borrow_mut() {
            Ok(b) => b,
            Err(e) => debug_unreachable!("{e}"),
        }
    }
}

/// A sequential memory shared between calls, which uses
/// a `Vec` for internal representation.
/// A [SharedMemory] instance should always be obtained using
/// the `new` static method to ensure memory safety.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SharedMemory {
    /// The underlying buffer.
    buffer: Option<Rc<RefCell<Vec<u8>>>>,
    /// Memory checkpoints for each depth.
    /// Invariant: these are always in bounds of `data`.
    my_checkpoint: usize,
    /// Child checkpoint that we need to free context to.
    child_checkpoint: Option<usize>,
    /// Memory limit. See [`Cfg`](context_interface::Cfg).
    #[cfg(feature = "memory_limit")]
    memory_limit: u64,
}

impl fmt::Debug for SharedMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedMemory")
            .field("current_len", &self.len())
            .field("context_memory", &hex::encode(&*self.context_memory()))
            .finish_non_exhaustive()
    }
}

impl Default for SharedMemory {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTr for SharedMemory {
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

    fn slice(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        self.slice_range(range)
    }

    fn local_memory_offset(&self) -> usize {
        self.my_checkpoint
    }

    fn set_data_from_global(
        &mut self,
        memory_offset: usize,
        data_offset: usize,
        len: usize,
        data_range: Range<usize>,
    ) {
        self.global_to_local_set_data(memory_offset, data_offset, len, data_range);
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an out-of-bounds range triggers undefined
    /// behavior. Callers must ensure that the range is within the bounds of the buffer.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn global_slice(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| match b.get(range) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: range; len: {}", self.len()),
        })
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
        self.my_checkpoint
            .saturating_add(offset)
            .saturating_add(len) as u64
            > self.memory_limit
    }
}

impl SharedMemory {
    /// Creates a new memory instance that can be shared between calls.
    ///
    /// The default initial capacity is 4KiB.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(4 * 1024) // from evmone
    }

    /// Creates a new invalid memory instance.
    #[inline]
    pub fn invalid() -> Self {
        Self {
            buffer: None,
            my_checkpoint: 0,
            child_checkpoint: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: 0,
        }
    }

    /// Creates a new memory instance with a given shared buffer.
    pub fn new_with_buffer(buffer: Rc<RefCell<Vec<u8>>>) -> Self {
        Self {
            buffer: Some(buffer),
            my_checkpoint: 0,
            child_checkpoint: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: u64::MAX,
        }
    }

    /// Creates a new memory instance that can be shared between calls with the given `capacity`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Some(Rc::new(RefCell::new(Vec::with_capacity(capacity)))),
            my_checkpoint: 0,
            child_checkpoint: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: u64::MAX,
        }
    }

    /// Creates a new memory instance that can be shared between calls,
    /// with `memory_limit` as upper bound for allocation size.
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
    pub fn set_memory_limit(&mut self, limit: u64) {
        #[cfg(feature = "memory_limit")]
        {
            self.memory_limit = limit;
        }
        let _ = limit;
    }

    #[inline]
    fn buffer(&self) -> &Rc<RefCell<Vec<u8>>> {
        debug_assert!(self.buffer.is_some(), "cannot use SharedMemory::empty");
        unsafe { self.buffer.as_ref().unwrap_unchecked() }
    }

    #[inline]
    fn buffer_ref(&self) -> Ref<'_, Vec<u8>> {
        self.buffer().dbg_borrow()
    }

    #[inline]
    fn buffer_ref_mut(&self) -> RefMut<'_, Vec<u8>> {
        self.buffer().dbg_borrow_mut()
    }

    /// Prepares the shared memory for a new child context.
    ///
    /// # Panics
    ///
    /// Panics if this function was already called without freeing child context.
    #[inline]
    pub fn new_child_context(&mut self) -> SharedMemory {
        if self.child_checkpoint.is_some() {
            panic!("new_child_context was already called without freeing child context");
        }
        let new_checkpoint = self.full_len();
        self.child_checkpoint = Some(new_checkpoint);
        SharedMemory {
            buffer: Some(self.buffer().clone()),
            my_checkpoint: new_checkpoint,
            // child_checkpoint is same as my_checkpoint
            child_checkpoint: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: self.memory_limit,
        }
    }

    /// Prepares the shared memory for returning from child context. Do nothing if there is no child context.
    #[inline]
    pub fn free_child_context(&mut self) {
        let Some(child_checkpoint) = self.child_checkpoint.take() else {
            return;
        };
        unsafe {
            self.buffer_ref_mut().set_len(child_checkpoint);
        }
    }

    /// Returns the length of the current memory range.
    #[inline]
    pub fn len(&self) -> usize {
        self.full_len() - self.my_checkpoint
    }

    fn full_len(&self) -> usize {
        self.buffer_ref().len()
    }

    /// Returns `true` if the current memory range is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the memory in-place so that `len` is equal to `new_len`.
    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        self.buffer()
            .dbg_borrow_mut()
            .resize(self.my_checkpoint + new_size, 0);
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_len(&self, offset: usize, size: usize) -> Ref<'_, [u8]> {
        self.slice_range(offset..offset + size)
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an out-of-bounds range triggers undefined
    /// behavior. Callers must ensure that the range is within the bounds of the memory (i.e.,
    /// `range.end <= self.len()`).
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_range(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| {
            match b.get(range.start + self.my_checkpoint..range.end + self.my_checkpoint) {
                Some(slice) => slice,
                None => debug_unreachable!("slice OOB: range; len: {}", self.len()),
            }
        })
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an out-of-bounds range triggers undefined
    /// behavior. Callers must ensure that the range is within the bounds of the buffer.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn global_slice_range(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| match b.get(range) {
            Some(slice) => slice,
            None => debug_unreachable!("slice OOB: range; len: {}", self.len()),
        })
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with out-of-bounds parameters triggers undefined
    /// behavior. Callers must ensure that `offset + size` does not exceed the length of the
    /// memory.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_mut(&mut self, offset: usize, size: usize) -> RefMut<'_, [u8]> {
        let buffer = self.buffer_ref_mut();
        RefMut::map(buffer, |b| {
            match b.get_mut(self.my_checkpoint + offset..self.my_checkpoint + offset + size) {
                Some(slice) => slice,
                None => debug_unreachable!("slice OOB: {offset}..{}", offset + size),
            }
        })
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
        (*self.slice_len(offset, 32)).try_into().unwrap()
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
        let mut dst = self.context_memory_mut();
        unsafe { set_data(dst.as_mut(), data, memory_offset, data_offset, len) };
    }

    /// Set data from global memory to local memory. If global range is smaller than len, zeroes the rest.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn global_to_local_set_data(
        &mut self,
        memory_offset: usize,
        data_offset: usize,
        len: usize,
        data_range: Range<usize>,
    ) {
        let mut buffer = self.buffer_ref_mut();
        let (src, dst) = buffer.split_at_mut(self.my_checkpoint);
        let src = if data_range.is_empty() {
            &mut []
        } else {
            src.get_mut(data_range).unwrap()
        };
        unsafe { set_data(dst, src, memory_offset, data_offset, len) };
    }

    /// Copies elements from one part of the memory to another part of itself.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
        self.context_memory_mut().copy_within(src..src + len, dst);
    }

    /// Returns a reference to the memory of the current context, the active memory.
    ///
    /// # Panics
    ///
    /// Panics if the checkpoint is invalid in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an invalid checkpoint triggers undefined
    /// behavior. The checkpoint must be within the bounds of the buffer.
    #[inline]
    pub fn context_memory(&self) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| match b.get(self.my_checkpoint..) {
            Some(slice) => slice,
            None => debug_unreachable!("Context memory should be always valid"),
        })
    }

    /// Returns a mutable reference to the memory of the current context.
    ///
    /// # Panics
    ///
    /// Panics if the checkpoint is invalid in debug builds only.
    ///
    /// # Safety
    ///
    /// In release builds, calling this method with an invalid checkpoint triggers undefined
    /// behavior. The checkpoint must be within the bounds of the buffer.
    #[inline]
    pub fn context_memory_mut(&mut self) -> RefMut<'_, [u8]> {
        let buffer = self.buffer_ref_mut();
        RefMut::map(buffer, |b| match b.get_mut(self.my_checkpoint..) {
            Some(slice) => slice,
            None => debug_unreachable!("Context memory should be always valid"),
        })
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
    len.saturating_add(31) / 32
}

/// Performs EVM memory resize.
#[inline]
#[must_use]
pub fn resize_memory<Memory: MemoryTr>(
    gas: &mut crate::Gas,
    memory: &mut Memory,
    offset: usize,
    len: usize,
) -> bool {
    let new_num_words = num_words(offset.saturating_add(len));
    if new_num_words > gas.memory().words_num {
        resize_memory_cold(gas, memory, new_num_words)
    } else {
        true
    }
}

#[cold]
#[inline(never)]
fn resize_memory_cold<Memory: MemoryTr>(
    gas: &mut crate::Gas,
    memory: &mut Memory,
    new_num_words: usize,
) -> bool {
    let cost = unsafe {
        gas.memory_mut()
            .record_new_len(new_num_words)
            .unwrap_unchecked()
    };
    if !gas.record_cost(cost) {
        return false;
    }
    memory.resize(new_num_words * 32);
    true
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
        assert_eq!(num_words(usize::MAX), usize::MAX / 32);
    }

    #[test]
    fn new_free_child_context() {
        let mut sm1 = SharedMemory::new();

        assert_eq!(sm1.buffer_ref().len(), 0);
        assert_eq!(sm1.my_checkpoint, 0);

        unsafe { sm1.buffer_ref_mut().set_len(32) };
        assert_eq!(sm1.len(), 32);
        let mut sm2 = sm1.new_child_context();

        assert_eq!(sm2.buffer_ref().len(), 32);
        assert_eq!(sm2.my_checkpoint, 32);
        assert_eq!(sm2.len(), 0);

        unsafe { sm2.buffer_ref_mut().set_len(96) };
        assert_eq!(sm2.len(), 64);
        let mut sm3 = sm2.new_child_context();

        assert_eq!(sm3.buffer_ref().len(), 96);
        assert_eq!(sm3.my_checkpoint, 96);
        assert_eq!(sm3.len(), 0);

        unsafe { sm3.buffer_ref_mut().set_len(128) };
        let sm4 = sm3.new_child_context();
        assert_eq!(sm4.buffer_ref().len(), 128);
        assert_eq!(sm4.my_checkpoint, 128);
        assert_eq!(sm4.len(), 0);

        // Free contexts
        drop(sm4);
        sm3.free_child_context();
        assert_eq!(sm3.buffer_ref().len(), 128);
        assert_eq!(sm3.my_checkpoint, 96);
        assert_eq!(sm3.len(), 32);

        sm2.free_child_context();
        assert_eq!(sm2.buffer_ref().len(), 96);
        assert_eq!(sm2.my_checkpoint, 32);
        assert_eq!(sm2.len(), 64);

        sm1.free_child_context();
        assert_eq!(sm1.buffer_ref().len(), 32);
        assert_eq!(sm1.my_checkpoint, 0);
        assert_eq!(sm1.len(), 32);
    }

    #[test]
    fn resize() {
        let mut sm1 = SharedMemory::new();
        sm1.resize(32);
        assert_eq!(sm1.buffer_ref().len(), 32);
        assert_eq!(sm1.len(), 32);
        assert_eq!(sm1.buffer_ref().get(0..32), Some(&[0_u8; 32] as &[u8]));

        let mut sm2 = sm1.new_child_context();
        sm2.resize(96);
        assert_eq!(sm2.buffer_ref().len(), 128);
        assert_eq!(sm2.len(), 96);
        assert_eq!(sm2.buffer_ref().get(32..128), Some(&[0_u8; 96] as &[u8]));

        sm1.free_child_context();
        assert_eq!(sm1.buffer_ref().len(), 32);
        assert_eq!(sm1.len(), 32);
        assert_eq!(sm1.buffer_ref().get(0..32), Some(&[0_u8; 32] as &[u8]));
    }
}
