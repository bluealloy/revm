use crate::{
    primitives::{B256, U256},
    InstructionResult,
};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

/// EVM interpreter stack limit.
pub(crate) const STACK_LIMIT: usize = 1024;

const PAGE_SIZE: usize = 4 * STACK_LIMIT;
type Buffer = Vec<U256>;
type Checkpoints = Vec<usize>;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Page {
    /// The underlying buffers shared between calls
    buffer: Buffer,
    /// Stack checkpoints for each depth
    ///
    /// Invariant: these are always in bounds of `buffer`.
    checkpoints: Checkpoints,
}

impl Page {
    #[inline]
    fn new() -> Self {
        let checkpoints = Vec::with_capacity(32);
        Self {
            buffer: Vec::with_capacity(PAGE_SIZE),
            checkpoints,
        }
    }
}

/// A sequential stack shared between calls, which uses
/// a vector of "pages" (buffers) or internal representation.
///
/// Each page includes a buffer of size `PAGE_SIZE` which
/// is shared between calls, and when there is no more space left
/// we move to a new page.
///
/// A [SharedStack] instance should always be obtained using
/// the `new` static method to ensure memory safety.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedStack {
    /// The underlying data divided in pages,
    /// where each page has size `PAGE_SIZE`
    pages: Vec<Page>,
    /// Keeps track of the page used for the current context
    /// Invariant: is a valid index of `self.pages`
    page_idx: usize,
    /// The pointer to the buffer used by the current page.
    /// Needed for better performance and to avoid double
    /// heap lookup for basic stack operations.
    ///
    /// Invariant: it a valid pointer to `self.pages[self.page_idx].buffer`
    buffer: *mut Buffer,
    /// Keeps track of the length of the stack in the current context.
    /// Needed for better performance and to avoid double
    /// heap lookup for basic stack operations.
    ///
    /// Invariant: it a valid pointer to `self.pages[self.page_idx].buffer`
    context_len: usize,
}

impl fmt::Display for SharedStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, x) in self.context_stack().iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{x}")?;
        }
        f.write_str("]")
    }
}

impl Default for SharedStack {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SharedStack {
    /// Instantiate a new shared stack with a single page of size `PAGE_SIZE`
    #[inline]
    pub fn new() -> Self {
        let mut pages = vec![Page::new()];
        let buffer = unsafe { &mut pages.get_unchecked_mut(0).buffer };
        Self {
            buffer,
            pages,
            page_idx: 0,
            context_len: 0,
        }
    }

    /// Prepares the shared stack for a new context
    #[inline]
    pub fn new_context(&mut self) {
        // check memory left for the current page
        let page = self.page_mut();
        let memory_left = page.buffer.capacity() - page.buffer.len();

        if memory_left < STACK_LIMIT {
            // we need to move to the next page
            self.page_idx += 1;
            if self.pages.len() == self.page_idx {
                // new page needed
                self.pages.push(Page::new());
            }
            let page = self.page_mut();
            page.checkpoints.push(0);
            self.buffer = &mut page.buffer
        } else {
            page.checkpoints.push(page.buffer.len())
        }
        self.context_len = 0;
    }

    /// Prepares the shared stack for returning to the previous context
    #[inline]
    pub fn free_context(&mut self) {
        let page = self.page_mut();
        // SAFETY: `new_context` logic always guarantees at least the zero checkpoint in it
        let old_checkpoint = unsafe { page.checkpoints.pop().unwrap_unchecked() };
        if let Some(last_checkpoint) = page.checkpoints.last() {
            // SAFETY: checkpoints are always in bound of buffers
            unsafe {
                page.buffer.set_len(old_checkpoint);
            };
            self.context_len = old_checkpoint - last_checkpoint;
        } else if self.page_idx > 0 {
            // no more checkpoints means we need to move to the previous page
            self.page_idx -= 1;
            self.buffer = &mut self.page_mut().buffer;
            // SAFETY: Going back to previous page implies non-empty checkpoints list
            self.context_len = unsafe {
                self.buffer().len() - self.page().checkpoints.last().cloned().unwrap_unchecked()
            };
        } else {
            self.context_len = 0;
        }
    }

    /// Returns the length of the stack in words.
    #[inline]
    pub fn len(&self) -> usize {
        self.context_len
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the underlying data of the current context stack.
    #[inline]
    pub fn data(&self) -> &[U256] {
        self.context_stack()
    }

    /// Removes the topmost element from the stack and returns it,
    /// or `StackUnderflow` if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Result<U256, InstructionResult> {
        if self.is_empty() {
            Err(InstructionResult::StackUnderflow)
        } else {
            self.context_len -= 1;
            // SAFETY: `self.len()` <= `self.buffer.len()` by construction,
            // and `self.len()` > 0 thanks to the check above
            Ok(unsafe { self.buffer_mut().pop().unwrap_unchecked() })
        }
    }

    /// Peek a value at given index for the stack, where the top of
    /// the stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        if self.len() > no_from_top {
            let buffer = self.buffer();
            // SAFETY: `0 < no_from_top + 1` <= self.len()` <= `self.buffer.len()`
            // Therefore, this index is bounded between 0 and `self.buffer.len()`
            let val = unsafe { *buffer.get_unchecked(buffer.len() - no_from_top - 1) };
            Ok(val)
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    /// Peeks the top of the stack.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn top_unsafe(&mut self) -> &mut U256 {
        let buffer = self.buffer_mut();
        let buf_len = buffer.len();
        buffer.get_unchecked_mut(buf_len - 1)
    }

    /// Removes the topmost element from the stack and returns it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for checking the length of the stack.
    #[inline]
    pub unsafe fn pop_unsafe(&mut self) -> U256 {
        self.context_len -= 1;
        self.buffer_mut().pop().unwrap_unchecked()
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

    /// Push a new value onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline]
    pub fn push(&mut self, value: U256) -> Result<(), InstructionResult> {
        if self.len() >= STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }
        let buffer = self.buffer_mut();
        let buf_len = buffer.len();
        // SAFETY: the check above and the `new_context` method
        // guarantee we have enough capacity
        unsafe {
            *buffer.get_unchecked_mut(buf_len) = value;
            buffer.set_len(buf_len + 1);
        }
        self.context_len += 1;
        Ok(())
    }

    /// Push a new value into the stack. If it will exceed the stack limit,
    /// returns `StackOverflow` error and leaves the stack unchanged.
    #[inline]
    pub fn push_b256(&mut self, value: B256) -> Result<(), InstructionResult> {
        self.push(value.into())
    }

    /// Duplicates the `N`th value from the top of the stack, with `N` >= 1
    #[inline]
    pub fn dup<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        if self.len() < N {
            Err(InstructionResult::StackUnderflow)
        } else if self.len() >= STACK_LIMIT {
            Err(InstructionResult::StackOverflow)
        } else {
            let buffer = self.buffer_mut();
            let buf_len = buffer.len();
            // SAFETY: the check above and the `new_context`
            // method guarantee we have enough capacity
            unsafe {
                let val = *buffer.get_unchecked(buf_len - N);
                *buffer.get_unchecked_mut(buf_len) = val;
                buffer.set_len(buf_len + 1);
            };
            self.context_len += 1;
            Ok(())
        }
    }

    /// Swaps the topmost value with the `N`th value from the top.
    #[inline]
    pub fn swap<const N: usize>(&mut self) -> Result<(), InstructionResult> {
        if self.len() <= N {
            return Err(InstructionResult::StackUnderflow);
        }
        let buffer = self.buffer_mut();
        let last = buffer.len() - 1;
        buffer.swap(last, last - N);
        Ok(())
    }

    /// Push a slice of bytes of `N` length onto the stack.
    ///
    /// If it will exceed the stack limit, returns `StackOverflow` error and leaves the stack
    /// unchanged.
    #[inline]
    pub fn push_slice<const N: usize>(&mut self, slice: &[u8]) -> Result<(), InstructionResult> {
        if self.len() >= STACK_LIMIT {
            return Err(InstructionResult::StackOverflow);
        }

        let mut slot = U256::ZERO;

        unsafe {
            let mut dangling = [0u8; 8];
            if N < 8 {
                dangling[8 - N..].copy_from_slice(slice);
                slot.as_limbs_mut()[0] = u64::from_be_bytes(dangling);
            } else if N < 16 {
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                if N != 8 {
                    dangling[8 * 2 - N..].copy_from_slice(&slice[..N - 8]);
                    slot.as_limbs_mut()[1] = u64::from_be_bytes(dangling);
                }
            } else if N < 24 {
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                slot.as_limbs_mut()[1] =
                    u64::from_be_bytes(slice[N - 16..N - 8].try_into().expect("Infallible"));
                if N != 16 {
                    dangling[8 * 3 - N..].copy_from_slice(&slice[..N - 16]);
                    slot.as_limbs_mut()[2] = u64::from_be_bytes(dangling);
                }
            } else {
                // M<32
                slot.as_limbs_mut()[0] =
                    u64::from_be_bytes(slice[N - 8..N].try_into().expect("Infallible"));
                slot.as_limbs_mut()[1] =
                    u64::from_be_bytes(slice[N - 16..N - 8].try_into().expect("Infallible"));
                slot.as_limbs_mut()[2] =
                    u64::from_be_bytes(slice[N - 24..N - 16].try_into().expect("Infallible"));
                if N == 32 {
                    slot.as_limbs_mut()[3] =
                        u64::from_be_bytes(slice[..N - 24].try_into().expect("Infallible"));
                } else if N != 24 {
                    dangling[8 * 4 - N..].copy_from_slice(&slice[..N - 24]);
                    slot.as_limbs_mut()[3] = u64::from_be_bytes(dangling);
                }
            }

            let buffer = self.buffer_mut();
            let buf_len = buffer.len();
            // SAFETY: the check above and the `new_context`
            // method guarantee we have enough capacity

            *buffer.get_unchecked_mut(buf_len) = slot;
            buffer.set_len(buf_len + 1);
            self.context_len += 1;
        }
        Ok(())
    }

    /// Set a value at given index for the stack, where the top of the
    /// stack is at index `0`. If the index is too large,
    /// `StackError::Underflow` is returned.
    #[inline]
    pub fn set(&mut self, no_from_top: usize, val: U256) -> Result<(), InstructionResult> {
        if self.len() > no_from_top {
            let buffer = self.buffer_mut();
            let buf_len = buffer.len();
            // SAFETY: `0 < no_from_top + 1` <= self.len()` <= `self.buffer.len()`.
            // Therefore, this index is bounded between 0 and `self.buffer.len()`
            unsafe { *buffer.get_unchecked_mut(buf_len - no_from_top - 1) = val };
            Ok(())
        } else {
            Err(InstructionResult::StackUnderflow)
        }
    }

    #[inline]
    fn page(&self) -> &Page {
        // SAFETY: by construction, `self.page_idx` is a valid
        // index of `self.pages`
        unsafe { self.pages.get_unchecked(self.page_idx) }
    }

    #[inline]
    fn page_mut(&mut self) -> &mut Page {
        // SAFETY: by construction, `self.page_idx` is a valid
        // index of `self.pages`
        unsafe { self.pages.get_unchecked_mut(self.page_idx) }
    }

    #[inline]
    fn buffer(&self) -> &Buffer {
        // SAFETY: by construction, `self.buffer` is a valid
        // pointer to the buffer of the current page
        unsafe { &*self.buffer }
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut Buffer {
        // SAFETY: by construction, `self.buffer` is a valid
        // pointer to the buffer of the current page
        unsafe { &mut *self.buffer }
    }

    /// Get a reference to the stack of the current context
    #[inline]
    fn context_stack(&self) -> &[U256] {
        let buffer = self.buffer();
        // SAFETY: range is bounded between 0 and buffer length
        unsafe { buffer.get_unchecked(buffer.len() - self.context_len..buffer.len()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_free() {
        let mut shared_stack = SharedStack::new();

        shared_stack.new_context();
        assert_eq!(shared_stack.page_idx, 0);
        assert_eq!(shared_stack.pages.len(), 1);
        assert_eq!(shared_stack.page().checkpoints.len(), 1);
        assert_eq!(shared_stack.page().checkpoints[0], 0);
        assert_eq!(shared_stack.context_len, 0);

        let new_len = STACK_LIMIT / 2;
        shared_stack.context_len = new_len;
        unsafe { shared_stack.buffer_mut().set_len(new_len) };

        shared_stack.new_context();
        assert_eq!(shared_stack.page_idx, 0);
        assert_eq!(shared_stack.pages.len(), 1);
        assert_eq!(shared_stack.page().checkpoints.len(), 2);
        assert_eq!(shared_stack.page().checkpoints[1], new_len);
        assert_eq!(shared_stack.context_len, 0);

        // first free in the same context
        shared_stack.free_context();
        assert_eq!(shared_stack.page_idx, 0);
        assert_eq!(shared_stack.pages.len(), 1);
        assert_eq!(shared_stack.page().checkpoints.len(), 1);
        assert_eq!(shared_stack.page().checkpoints[0], 0);
        assert_eq!(shared_stack.context_len, new_len);

        // reset
        shared_stack.free_context();
        assert_eq!(shared_stack.page_idx, 0);
        assert_eq!(shared_stack.pages.len(), 1);
        assert_eq!(shared_stack.page().checkpoints.len(), 0);
        assert_eq!(shared_stack.context_len, 0);

        // fill current page
        for i in 0..7 {
            shared_stack.new_context();
            assert_eq!(shared_stack.context_len, 0);
            assert_eq!(shared_stack.pages.len(), 1);
            assert_eq!(shared_stack.page().checkpoints.len(), i + 1);
            assert_eq!(
                shared_stack.page().checkpoints.last().cloned().unwrap(),
                shared_stack.buffer().len()
            );

            let new_len = STACK_LIMIT / 2;
            unsafe { shared_stack.buffer_mut().set_len(new_len * (i + 1)) };
            shared_stack.context_len = new_len;
            println!("buffer len = {}", shared_stack.buffer().len());
        }

        // a new page should be created
        shared_stack.new_context();
        assert_eq!(shared_stack.page_idx, 1);
        assert_eq!(shared_stack.pages.len(), 2);
        assert_eq!(shared_stack.page().checkpoints.len(), 1);
        assert_eq!(shared_stack.page().checkpoints[0], 0);
        assert_eq!(shared_stack.context_len, 0);

        // go back to previous page
        shared_stack.free_context();
        assert_eq!(shared_stack.page_idx, 0);
        assert_eq!(shared_stack.pages.len(), 2);
        assert_eq!(shared_stack.page().checkpoints.len(), 7);
        assert_eq!(shared_stack.page().checkpoints[6], STACK_LIMIT * 3);
        assert_eq!(shared_stack.context_len, STACK_LIMIT / 2);

        // go to new page without creating it
        shared_stack.new_context();
        assert_eq!(shared_stack.page_idx, 1);
        assert_eq!(shared_stack.pages.len(), 2);
        assert_eq!(shared_stack.page().checkpoints.len(), 1);
        assert_eq!(shared_stack.page().checkpoints[0], 0);
        assert_eq!(shared_stack.context_len, 0);
    }

    #[test]
    fn new_consecutive() {
        let mut shared_stack = SharedStack::new();
        for i in 0..3 {
            for j in 0..7 {
                println!("i = {}, j = {}", i, j);
                shared_stack.new_context();
                assert_eq!(shared_stack.page_idx, i);
                assert_eq!(shared_stack.pages.len(), i + 1);
                assert_eq!(shared_stack.context_len, 0);
                assert_eq!(shared_stack.page().checkpoints.len(), j + 1);
                assert_eq!(
                    shared_stack.page().checkpoints.last().cloned().unwrap(),
                    shared_stack.buffer().len()
                );

                let new_len = STACK_LIMIT / 2;
                unsafe { shared_stack.buffer_mut().set_len(new_len * (j + 1)) };
                shared_stack.context_len = new_len;
                println!("buffer len = {}", shared_stack.buffer().len());
            }
        }

        // println!("Free");
        //
        // for i in (0..3).rev() {
        //     for j in (0..7).rev() {
        //         println!("i = {}, j = {}", i, j);
        //         assert_eq!(shared_stack.page_idx, i);
        //         assert_eq!(shared_stack.pages.len(), i + 1);
        //         assert_eq!(shared_stack.context_len, STACK_LIMIT / 2);
        //         assert_eq!(shared_stack.page().checkpoints.len(), j + 1);
        //         assert_eq!(
        //             shared_stack.page().checkpoints.last().cloned().unwrap(),
        //             shared_stack.buffer().len()
        //         );
        //         println!("buffer len = {}", shared_stack.buffer().len());
        //         shared_stack.free_context();
        //     }
        // }
    }

    // #[test]
    // fn new_free_context_stack() {
    //     let mut shared_stack = SharedStack::new();
    //     shared_stack.new_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 0);
    //     assert_eq!(shared_stack.checkpoints.len(), 1);
    //
    //     unsafe { shared_stack.buffer.set_len(16) }
    //     shared_stack.new_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 16);
    //     assert_eq!(shared_stack.checkpoints.len(), 2);
    //     assert_eq!(shared_stack.len(), 0);
    //
    //     unsafe { shared_stack.buffer.set_len(48) }
    //     shared_stack.new_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 48);
    //     assert_eq!(shared_stack.checkpoints.len(), 3);
    //     assert_eq!(shared_stack.len(), 0);
    //     assert_eq!(shared_stack.buffer.len(), 48);
    //
    //     // free contexts
    //     shared_stack.free_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 16);
    //     assert_eq!(shared_stack.buffer.len(), 48);
    //     assert_eq!(shared_stack.checkpoints.len(), 2);
    //     assert_eq!(shared_stack.len(), 32);
    //
    //     shared_stack.free_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 0);
    //     assert_eq!(shared_stack.checkpoints.len(), 1);
    //     assert_eq!(shared_stack.len(), 16);
    //     assert_eq!(shared_stack.buffer.len(), 16);
    //
    //     shared_stack.free_context();
    //     assert_eq!(shared_stack.last_checkpoint(), 0);
    //     assert_eq!(shared_stack.checkpoints.len(), 0);
    //     assert_eq!(shared_stack.len(), 0);
    //     assert_eq!(shared_stack.buffer.len(), 0);
    // }

    #[test]
    fn pop() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        shared_stack.buffer_mut().push(U256::from(1));
        assert_eq!(shared_stack.buffer()[0], U256::from(1));
        shared_stack.context_len += 1;
        assert_eq!(shared_stack.pop(), Ok(U256::from(1)));
    }

    #[test]
    fn pop_underflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(shared_stack.pop(), Err(InstructionResult::StackUnderflow));

        // pop underflow in a new empty context
        unsafe { shared_stack.page_mut().buffer.set_len(STACK_LIMIT / 2) }
        shared_stack.new_context();
        assert_eq!(shared_stack.pop(), Err(InstructionResult::StackUnderflow));
    }

    #[test]
    fn push() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        let one = U256::from(1);
        let res = shared_stack.push(one);
        assert_eq!(res, Ok(()));
        assert_eq!(shared_stack.page_mut().buffer[0], one);

        let two = U256::from(2);
        let res = shared_stack.push(two);
        assert_eq!(res, Ok(()));
        assert_eq!(shared_stack.page_mut().buffer[1], two);
    }

    #[test]
    fn push_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        for _ in 1..=STACK_LIMIT {
            assert_eq!(shared_stack.push(U256::ZERO), Ok(()));
        }

        assert_eq!(shared_stack.page_mut().buffer.len(), STACK_LIMIT);
        assert_eq!(shared_stack.len(), STACK_LIMIT);
        assert_eq!(
            shared_stack.push(U256::ZERO),
            Err(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn push_slice() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(shared_stack.push_slice::<1>(&[1]), Ok(()));
        assert_eq!(shared_stack.page_mut().buffer[0], U256::from(1));
        assert_eq!(shared_stack.len(), 1);
    }

    #[test]
    fn push_slice_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        for _ in 1..=STACK_LIMIT {
            assert_eq!(shared_stack.push_slice::<1>(&[0]), Ok(()));
        }
        assert_eq!(shared_stack.len(), STACK_LIMIT);
        assert_eq!(shared_stack.page_mut().buffer.len(), STACK_LIMIT);
        assert_eq!(
            shared_stack.push_slice::<1>(&[0]),
            Err(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn dup() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(shared_stack.push(U256::from(1)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(2)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(3)), Ok(()));
        assert_eq!(shared_stack.push(U256::from(4)), Ok(()));
        assert_eq!(shared_stack.len(), 4);
        assert_eq!(shared_stack.page_mut().buffer[3], U256::from(4));

        assert_eq!(shared_stack.dup::<1>(), Ok(()));
        assert_eq!(shared_stack.len(), 5);
        assert_eq!(shared_stack.page_mut().buffer[4], U256::from(4));

        assert_eq!(shared_stack.dup::<3>(), Ok(()));
        assert_eq!(shared_stack.len(), 6);
        assert_eq!(shared_stack.page_mut().buffer[5], U256::from(3));
    }

    #[test]
    fn dup_stack_underflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackUnderflow)
        );

        unsafe { shared_stack.page_mut().buffer.set_len(STACK_LIMIT / 2) }
        shared_stack.new_context();

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackUnderflow)
        );
    }

    #[test]
    fn dup_stack_overflow() {
        let mut shared_stack = SharedStack::new();
        shared_stack.new_context();

        shared_stack.page_mut().buffer.push(U256::ZERO);
        shared_stack.context_len += 1;

        for _ in 2..=STACK_LIMIT {
            assert_eq!(shared_stack.dup::<1>(), Ok(()))
        }

        assert_eq!(shared_stack.page_mut().buffer.len(), STACK_LIMIT);
        assert_eq!(shared_stack.len(), STACK_LIMIT);

        assert_eq!(
            shared_stack.dup::<1>(),
            Err(InstructionResult::StackOverflow)
        );
    }
}
