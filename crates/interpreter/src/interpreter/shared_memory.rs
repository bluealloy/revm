use super::MemoryTr;
use crate::InstructionResult;
use context_interface::cfg::GasParams;
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
    /// Memory checkpoint for this context.
    /// This is in bounds during normal context sequencing. Shared aliases or externally
    /// supplied buffers can invalidate it, so consumers validate it before use.
    my_checkpoint: usize,
    /// Child checkpoint used to restore the parent context.
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
    /// Panics on out of bounds access.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn global_slice(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        self.global_slice_range(range)
    }

    fn resize(&mut self, new_size: usize) -> bool {
        self.resize(new_size);
        true
    }

    /// Returns `true` if the `new_words` for the current context memory will
    /// make the shared buffer length exceed the `memory_limit`.
    #[cfg(feature = "memory_limit")]
    #[inline]
    fn limit_reached(&self, new_words: usize) -> bool {
        self.my_checkpoint
            .saturating_add(new_words.saturating_mul(32)) as u64
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
    pub const fn invalid() -> Self {
        Self {
            buffer: None,
            my_checkpoint: 0,
            child_checkpoint: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: 0,
        }
    }

    /// Creates a new memory instance with a given shared buffer.
    pub const fn new_with_buffer(buffer: Rc<RefCell<Vec<u8>>>) -> Self {
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
    pub const fn set_memory_limit(&mut self, limit: u64) {
        #[cfg(feature = "memory_limit")]
        {
            self.memory_limit = limit;
        }
        // for clippy.
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

    /// Returns a byte slice of the backing buffer, applying `base` to `range`.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn slice_range_with_base(&self, range: Range<usize>, base: usize) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| {
            let start = range
                .start
                .checked_add(base)
                .expect("memory read offset overflow");
            let end = range
                .end
                .checked_add(base)
                .expect("memory read length overflow");
            b.get(start..end)
                .unwrap_or_else(|| panic!("slice OOB: {start}..{end}; len: {}", b.len()))
        })
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
    ///
    /// # Panics
    ///
    /// Panics if the shared buffer was truncated below the child checkpoint.
    #[inline]
    pub fn free_child_context(&mut self) {
        let Some(child_checkpoint) = self.child_checkpoint else {
            return;
        };
        {
            let mut buffer = self.buffer_ref_mut();
            assert!(
                buffer.len() >= child_checkpoint,
                "shared memory buffer is below child checkpoint"
            );
            buffer.truncate(child_checkpoint);
        }
        self.child_checkpoint = None;
    }

    /// Returns the length of the current memory range.
    #[inline]
    pub fn len(&self) -> usize {
        self.full_len()
            .checked_sub(self.my_checkpoint)
            .expect("shared memory checkpoint out of bounds")
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
    ///
    /// # Panics
    ///
    /// Panics if the checkpoint plus the new size overflows or if resizing would
    /// invalidate the child checkpoint recorded by this handle.
    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        let new_len = self
            .my_checkpoint
            .checked_add(new_size)
            .expect("memory resize overflow");
        if self
            .child_checkpoint
            .is_some_and(|child_checkpoint| new_len < child_checkpoint)
        {
            panic!("memory resize below child checkpoint");
        }
        self.buffer().dbg_borrow_mut().resize(new_len, 0);
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
    /// Panics on out of bounds access.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_range(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        self.slice_range_with_base(range, self.my_checkpoint)
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn global_slice_range(&self, range: Range<usize>) -> Ref<'_, [u8]> {
        self.slice_range_with_base(range, 0)
    }

    /// Returns a byte slice of the memory region at the given offset.
    ///
    /// # Panics
    ///
    /// Panics on out of bounds access.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn slice_mut(&mut self, offset: usize, size: usize) -> RefMut<'_, [u8]> {
        let start = self
            .my_checkpoint
            .checked_add(offset)
            .expect("memory write offset overflow");
        let end = start
            .checked_add(size)
            .expect("memory write length overflow");
        let buffer = self.buffer_ref_mut();
        RefMut::map(buffer, |b| {
            b.get_mut(start..end).expect("memory write out of bounds")
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

    /// Set memory from data. The destination range is validated, and source bytes outside
    /// `data` are written as zeroes.
    ///
    /// # Panics
    ///
    /// Panics if memory is out of bounds.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        let mut dst = self.context_memory_mut();
        set_data(dst.as_mut(), data, memory_offset, data_offset, len);
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
        set_data(dst, src, memory_offset, data_offset, len);
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
    /// Panics if the checkpoint is invalid.
    #[inline]
    pub fn context_memory(&self) -> Ref<'_, [u8]> {
        let buffer = self.buffer_ref();
        Ref::map(buffer, |b| {
            b.get(self.my_checkpoint..)
                .expect("shared memory checkpoint out of bounds")
        })
    }

    /// Returns a mutable reference to the memory of the current context.
    ///
    /// # Panics
    ///
    /// Panics if the checkpoint is invalid.
    #[inline]
    pub fn context_memory_mut(&mut self) -> RefMut<'_, [u8]> {
        let buffer = self.buffer_ref_mut();
        RefMut::map(buffer, |b| {
            b.get_mut(self.my_checkpoint..)
                .expect("shared memory checkpoint out of bounds")
        })
    }
}

/// Copies data from src to dst taking into account the offsets and len.
///
/// If src does not have enough data, it nullifies the rest of dst that is not copied.
///
fn set_data(dst: &mut [u8], src: &[u8], dst_offset: usize, src_offset: usize, len: usize) {
    if len == 0 {
        return;
    }
    let dst_end = dst_offset
        .checked_add(len)
        .filter(|&end| end <= dst.len())
        .expect("memory write out of bounds");
    if src_offset >= src.len() {
        // Nullify all memory slots
        dst[dst_offset..dst_end].fill(0);
        return;
    }
    let src_end = src_offset + min(len, src.len() - src_offset);
    let src_len = src_end - src_offset;
    debug_assert!(src_end <= src.len());
    let data = unsafe { src.get_unchecked(src_offset..src_end) };
    // SAFETY: `dst_end` was checked against `dst.len()` above, and `src_len <= len`.
    unsafe {
        dst.get_unchecked_mut(dst_offset..dst_offset + src_len)
            .copy_from_slice(data)
    };

    // Nullify rest of memory slots
    // SAFETY: `dst_end` was checked against `dst.len()` above.
    unsafe { dst.get_unchecked_mut(dst_offset + src_len..dst_end).fill(0) };
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
    #[cfg(feature = "memory_limit")]
    if memory.limit_reached(new_num_words) {
        return Err(InstructionResult::MemoryLimitOOG);
    }

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
    #[cfg(feature = "std")]
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[cfg(feature = "std")]
    fn assert_panic_message(result: std::thread::Result<()>, expected: &str) {
        let payload = result.expect_err("operation should panic");
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
        assert_eq!(message, Some(expected));
    }

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

    #[test]
    #[should_panic(expected = "memory resize overflow")]
    fn resize_overflow_panics() {
        let mut parent = SharedMemory::new();
        parent.resize(1);
        parent.new_child_context().resize(usize::MAX);
    }

    #[cfg(feature = "std")]
    #[test]
    fn parent_resize_below_child_checkpoint_is_atomic() {
        let mut parent = SharedMemory::new();
        parent.resize(1);
        parent.set_byte(0, 0xA5);
        let _child = parent.new_child_context();

        let result = catch_unwind(AssertUnwindSafe(|| parent.resize(0)));

        assert_panic_message(result, "memory resize below child checkpoint");
        assert_eq!(parent.buffer_ref().as_slice(), &[0xA5]);
        assert_eq!(parent.child_checkpoint, Some(1));
    }

    #[test]
    #[should_panic(expected = "shared memory checkpoint out of bounds")]
    fn external_buffer_truncation_panics() {
        let buffer = Rc::new(RefCell::new(vec![0]));
        let mut parent = SharedMemory::new_with_buffer(buffer.clone());
        let child = parent.new_child_context();
        buffer.borrow_mut().clear();
        let _ = child.context_memory();
    }

    #[cfg(feature = "std")]
    #[test]
    fn free_child_context_with_invalid_checkpoint_is_retryable() {
        let buffer = Rc::new(RefCell::new(vec![0]));
        let mut parent = SharedMemory::new_with_buffer(buffer.clone());
        let _child = parent.new_child_context();
        *buffer.borrow_mut() = Vec::new();

        let result = catch_unwind(AssertUnwindSafe(|| parent.free_child_context()));

        assert_panic_message(result, "shared memory buffer is below child checkpoint");
        assert_eq!(parent.child_checkpoint, Some(1));
        assert!(buffer.borrow().is_empty());

        buffer.borrow_mut().extend_from_slice(&[0xA5, 0x5A]);
        parent.free_child_context();

        assert_eq!(parent.child_checkpoint, None);
        assert_eq!(buffer.borrow().as_slice(), &[0xA5]);
    }

    #[test]
    #[should_panic(expected = "memory read offset overflow")]
    fn child_slice_start_overflow_panics() {
        let mut parent = SharedMemory::new();
        parent.resize(1);
        let child = parent.new_child_context();
        let _ = child.slice_range(usize::MAX..usize::MAX);
    }

    #[test]
    #[should_panic(expected = "memory read length overflow")]
    fn child_slice_end_overflow_panics() {
        let mut parent = SharedMemory::new();
        parent.resize(1);
        let child = parent.new_child_context();
        let _ = child.slice_range(0..usize::MAX);
    }

    #[test]
    #[should_panic(expected = "shared memory checkpoint out of bounds")]
    fn invalid_checkpoint_len_panics() {
        let buffer = Rc::new(RefCell::new(vec![0]));
        let mut parent = SharedMemory::new_with_buffer(buffer.clone());
        let child = parent.new_child_context();
        buffer.borrow_mut().clear();
        let _ = child.len();
    }

    #[test]
    #[should_panic(expected = "shared memory checkpoint out of bounds")]
    fn invalid_checkpoint_context_memory_mut_panics() {
        let buffer = Rc::new(RefCell::new(vec![0]));
        let mut parent = SharedMemory::new_with_buffer(buffer.clone());
        let mut child = parent.new_child_context();
        buffer.borrow_mut().clear();
        let _ = child.context_memory_mut();
    }

    #[test]
    #[should_panic(expected = "slice OOB")]
    fn slice_out_of_bounds_panics() {
        let mut memory = SharedMemory::new();
        memory.resize(1);
        let _ = memory.slice_range(1..2);
    }

    #[test]
    #[should_panic(expected = "memory write out of bounds")]
    fn set_out_of_bounds_panics() {
        let mut memory = SharedMemory::new();
        memory.resize(1);
        memory.set(1, &[0xFF]);
    }

    #[test]
    #[should_panic(expected = "memory write length overflow")]
    fn set_length_overflow_panics() {
        let mut memory = SharedMemory::new();
        memory.set(usize::MAX, &[0xFF; 8]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn child_set_offset_overflow_is_atomic() {
        let mut parent = SharedMemory::new();
        parent.resize(1);
        parent.set_byte(0, 0xA5);
        let mut child = parent.new_child_context();

        let result = catch_unwind(AssertUnwindSafe(|| child.set(usize::MAX, &[0xFF])));

        assert_eq!(&*parent.context_memory(), &[0xA5]);
        assert_panic_message(result, "memory write offset overflow");
    }

    #[test]
    #[should_panic(expected = "memory write out of bounds")]
    fn set_data_range_overflow_panics() {
        let mut memory = SharedMemory::new();
        memory.set_data(usize::MAX, 0, usize::MAX, &[0xFF; 8]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn set_data_out_of_bounds_is_atomic() {
        let mut memory = SharedMemory::new();
        memory.resize(2);
        memory.set(0, &[0xA5, 0x5A]);

        let result = catch_unwind(AssertUnwindSafe(|| memory.set_data(1, 0, 2, &[0xFF, 0xFF])));

        assert_panic_message(result, "memory write out of bounds");
        assert_eq!(&*memory.context_memory(), &[0xA5, 0x5A]);
    }

    #[cfg(feature = "memory_limit")]
    #[test]
    fn resize_memory_limit() {
        let gas_table = GasParams::default();

        // Limit of 64 bytes allows 2 words.
        let mut memory = SharedMemory::new_with_memory_limit(64);
        let mut gas = crate::Gas::new(100_000);

        // Resize to 1 word (32 bytes) should succeed.
        assert!(resize_memory(&mut gas, &mut memory, &gas_table, 0, 32).is_ok());
        assert_eq!(memory.len(), 32);

        // Resize to 2 words (64 bytes) should succeed.
        assert!(resize_memory(&mut gas, &mut memory, &gas_table, 0, 64).is_ok());
        assert_eq!(memory.len(), 64);

        // Resize to 3 words (96 bytes) should fail with MemoryLimitOOG.
        assert_eq!(
            resize_memory(&mut gas, &mut memory, &gas_table, 0, 96),
            Err(InstructionResult::MemoryLimitOOG),
        );
        // Memory should not have grown.
        assert_eq!(memory.len(), 64);
    }
}
