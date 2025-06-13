//! Local context trait [`LocalContextTr`] and related types.
use core::{
    cell::{Ref, RefCell},
    ops::Range,
};
use primitives::{Bytes, B256};
use std::{rc::Rc, vec::Vec};

/// Non-empty, item-pooling Vec.
#[derive(Clone, Debug)]
pub struct FrameStack<T> {
    stack: Vec<Box<T>>,
    index: usize,
}

impl<T> Default for FrameStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FrameStack<T> {
    /// Creates a new, empty stack. It must be initialized with init before use.
    #[inline]
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1025),
            index: 0,
        }
    }

    /// Initializes the stack with a single item.
    #[inline]
    pub fn start_init(&mut self) -> OutFrame<'_, T> {
        self.index = 0;
        if self.stack.is_empty() {
            self.stack.reserve(1);
        }
        self.out_frame_at(0)
    }

    /// Finishes initialization.
    #[inline]
    pub fn end_init(&mut self, token: FrameToken) {
        token.assert();
        if self.stack.is_empty() {
            unsafe { self.stack.set_len(1) };
        }
    }

    /// Returns the current index of the stack.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Increments the index.
    #[inline]
    pub fn push(&mut self, token: FrameToken) {
        token.assert();
        if self.index + 1 == self.stack.len() {
            unsafe { self.stack.set_len(self.stack.len() + 1) };
            self.stack.reserve(1);
        }
        self.index += 1;
    }

    /// Clears the stack by setting the index to 0.
    /// It does not destroy the stack.
    #[inline]
    pub fn clear(&mut self) {
        self.index = 0;
    }

    /// Decrements the index.
    #[inline]
    pub fn pop(&mut self) {
        self.index -= 1;
    }

    /// Returns the current item.
    #[inline]
    pub fn get(&mut self) -> &mut T {
        debug_assert!(self.stack.capacity() > self.index + 1);
        unsafe { &mut *self.stack.as_mut_ptr().add(self.index) }
    }

    /// Get next uninitialized item.
    #[inline]
    pub fn get_next(&mut self) -> OutFrame<'_, T> {
        self.out_frame_at(self.index + 1)
    }

    fn out_frame_at(&mut self, idx: usize) -> OutFrame<'_, T> {
        unsafe {
            OutFrame::new_maybe_uninit(self.stack.as_mut_ptr().add(idx), idx < self.stack.len())
        }
    }
}

/// A potentially initialized frame. Used when initializing a new frame in the main loop.
#[allow(missing_debug_implementations)]
pub struct OutFrame<'a, T> {
    ptr: *mut Box<T>,
    init: bool,
    lt: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T> OutFrame<'a, T> {
    /// Creates a new initialized `OutFrame` from a mutable reference to a type `T`.
    pub fn new_init(slot: &'a mut Box<T>) -> Self {
        unsafe { Self::new_maybe_uninit(slot, true) }
    }

    /// Creates a new uninitialized `OutFrame` from a mutable reference to a `MaybeUninit<T>`.
    pub fn new_uninit(slot: &'a mut core::mem::MaybeUninit<Box<T>>) -> Self {
        unsafe { Self::new_maybe_uninit(slot.as_mut_ptr(), false) }
    }

    /// Creates a new `OutFrame` from a raw pointer to a type `T`.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes that the pointer is valid and points to a location
    /// where a type `T` can be stored. It also assumes that the `init` flag correctly reflects whether
    /// the type `T` has been initialized or not.
    pub unsafe fn new_maybe_uninit(ptr: *mut Box<T>, init: bool) -> Self {
        Self {
            ptr,
            init,
            lt: Default::default(),
        }
    }

    /// Returns a mutable reference to the type `T`, initializing it if it hasn't been initialized yet.
    pub fn get(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if !self.init {
            self.do_init(f);
        }
        unsafe { &mut *self.ptr }
    }

    #[cold]
    fn do_init(&mut self, f: impl FnOnce() -> T) {
        unsafe {
            self.init = true;
            self.ptr.write(Box::new(f()));
        }
    }

    /// Returns a mutable reference to the type `T`, without checking if it has been initialized.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it assumes that the `OutFrame` has been initialized before use.
    pub unsafe fn get_unchecked(&mut self) -> &mut T {
        debug_assert!(self.init, "OutFrame must be initialized before use");
        unsafe { &mut *self.ptr }
    }

    /// Consumes the `OutFrame`, returning a `FrameToken` that indicates the frame has been initialized.
    pub fn consume(self) -> FrameToken {
        FrameToken(self.init)
    }
}

/// Used to guarantee that a frame is initialized before use.
#[allow(missing_debug_implementations)]
pub struct FrameToken(bool);

impl FrameToken {
    /// Asserts that the frame token is initialized.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn assert(self) {
        assert!(self.0, "FrameToken must be initialized before use");
    }
}

/// Local context used for caching initcode from Initcode transactions.
pub trait LocalContextTr {
    /// Get the local context
    fn insert_initcodes(&mut self, initcodes: &[Bytes]);

    /// Get validated initcode by hash. if initcode is not validated it is assumed
    /// that validation is going to be performed inside this function.
    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes>;

    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    fn shared_memory_buffer(&self) -> &Rc<RefCell<Vec<u8>>>;

    /// Slice of the shared memory buffer returns None if range is not valid or buffer can't be borrowed.
    fn shared_memory_buffer_slice(&self, range: Range<usize>) -> Option<Ref<'_, [u8]>> {
        let buffer = self.shared_memory_buffer();
        buffer.borrow().get(range.clone())?;
        Some(Ref::map(buffer.borrow(), |b| {
            b.get(range).unwrap_or_default()
        }))
    }

    /// Clear the local context.
    fn clear(&mut self);

    /// Frame stack.
    fn frame_stack(&mut self) -> &mut FrameStack<u128>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_stack() {
        let mut stack = FrameStack::new();
        let mut frame = stack.start_init();
        frame.get(|| 1);
        let token = frame.consume();
        stack.end_init(token);

        assert_eq!(stack.index(), 0);
        assert_eq!(stack.stack.len(), 1);

        let a = stack.get();
        assert_eq!(a, &mut 1);
        let mut b = stack.get_next();
        assert!(!b.init);
        assert_eq!(b.get(|| 2), &mut 2);
        let token = b.consume(); // TODO: remove
        stack.push(token);

        assert_eq!(stack.index(), 1);
        assert_eq!(stack.stack.len(), 2);
        let a = stack.get();
        assert_eq!(a, &mut 2);
        let b = stack.get_next();
        assert!(!b.init);

        stack.pop();

        assert_eq!(stack.index(), 0);
        assert_eq!(stack.stack.len(), 2);
        let a = stack.get();
        assert_eq!(a, &mut 1);
        let mut b = stack.get_next();
        assert!(b.init);
        assert_eq!(unsafe { b.get_unchecked() }, &mut 2);
    }
}
