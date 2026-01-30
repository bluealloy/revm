use core::ptr;
use primitives::U256;
use std::vec::Vec;

/// Pointer-based stack view for hot-path operations.
///
/// This is an unsafe, performance-optimized view into the stack
/// that uses pointer arithmetic instead of length tracking.
///
/// # Safety
///
/// Callers must ensure:
/// - Stack bounds are checked before using StackTop operations
/// - The underlying stack memory remains valid
#[derive(Debug)]
pub struct StackTop {
    /// Pointer to one past the top element (next push location)
    end: *mut U256,
    /// Pointer to the base of the stack (for bounds checking)
    base: *const U256,
}

impl StackTop {
    /// Create a new StackTop from a stack's data pointer and length.
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to an allocation of at least `len` U256s.
    #[inline]
    pub unsafe fn new(base: *mut U256, len: usize) -> Self {
        Self {
            end: base.add(len),
            base,
        }
    }

    /// Get current stack length
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { self.end.offset_from(self.base) as usize }
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Access the top element (index 0 = top)
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack is not empty.
    #[inline]
    pub unsafe fn top(&mut self) -> &mut U256 {
        &mut *self.end.sub(1)
    }

    /// Access element at index from top (0 = top, 1 = second from top, etc.)
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least `index + 1` elements.
    #[inline]
    pub unsafe fn get(&self, index: usize) -> &U256 {
        &*self.end.sub(index + 1)
    }

    /// Access element at index from top mutably
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least `index + 1` elements.
    #[inline]
    pub unsafe fn get_mut(&mut self, index: usize) -> &mut U256 {
        &mut *self.end.sub(index + 1)
    }

    /// Pop the top element and return it
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack is not empty.
    #[inline]
    pub unsafe fn pop(&mut self) -> U256 {
        self.end = self.end.sub(1);
        ptr::read(self.end)
    }

    /// Push a value onto the stack
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has capacity for another element.
    #[inline]
    pub unsafe fn push(&mut self, value: U256) {
        ptr::write(self.end, value);
        self.end = self.end.add(1);
    }

    /// Pop N values and return them as an array
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least N elements.
    #[inline]
    pub unsafe fn popn<const N: usize>(&mut self) -> [U256; N] {
        core::array::from_fn(|_| self.pop())
    }

    /// Pop N values and return mutable reference to new top
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least N + 1 elements.
    #[inline]
    pub unsafe fn popn_top<const N: usize>(&mut self) -> ([U256; N], &mut U256) {
        let popped = self.popn::<N>();
        (popped, self.top())
    }

    /// Swap top with element at index N (1-indexed like SWAP1)
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least `n + 1` elements.
    #[inline]
    pub unsafe fn swap(&mut self, n: usize) {
        let top_ptr = self.end.sub(1);
        let other_ptr = self.end.sub(n + 1);
        ptr::swap_nonoverlapping(top_ptr, other_ptr, 1);
    }

    /// Duplicate element at index N to top (1-indexed like DUP1)
    ///
    /// # Safety
    ///
    /// Caller must ensure the stack has at least `n` elements and
    /// capacity for one more element.
    #[inline]
    pub unsafe fn dup(&mut self, n: usize) {
        let value = ptr::read(self.end.sub(n));
        self.push(value);
    }

    /// Write back the new length to the original Vec
    ///
    /// # Safety
    ///
    /// Caller must ensure:
    /// - The Vec is the same one used to create this StackTop
    /// - The new length does not exceed the Vec's capacity
    #[inline]
    pub unsafe fn write_back_len(&self, stack_data: &mut Vec<U256>) {
        stack_data.set_len(self.len());
    }
}

#[cfg(test)]
mod tests {
    use super::super::stack::Stack;
    use super::*;

    #[test]
    fn test_stack_top_basic_operations() {
        let mut stack = Stack::new();
        let _ = stack.push(U256::from(1));
        let _ = stack.push(U256::from(2));
        let _ = stack.push(U256::from(3));

        unsafe {
            let mut st = stack.as_stack_top();

            assert_eq!(st.len(), 3);
            assert_eq!(*st.top(), U256::from(3));
            assert_eq!(*st.get(0), U256::from(3));
            assert_eq!(*st.get(1), U256::from(2));
            assert_eq!(*st.get(2), U256::from(1));

            let popped = st.pop();
            assert_eq!(popped, U256::from(3));
            assert_eq!(st.len(), 2);

            st.push(U256::from(42));
            assert_eq!(st.len(), 3);
            assert_eq!(*st.top(), U256::from(42));

            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(0).unwrap(), U256::from(42));
    }

    #[test]
    fn test_stack_top_popn() {
        let mut stack = Stack::new();
        for i in 1..=5 {
            let _ = stack.push(U256::from(i));
        }

        unsafe {
            let mut st = stack.as_stack_top();

            let [a, b] = st.popn::<2>();
            assert_eq!(a, U256::from(5));
            assert_eq!(b, U256::from(4));
            assert_eq!(st.len(), 3);

            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.len(), 3);
    }

    #[test]
    fn test_stack_top_popn_top() {
        let mut stack = Stack::new();
        for i in 1..=5 {
            let _ = stack.push(U256::from(i));
        }

        unsafe {
            let mut st = stack.as_stack_top();

            let ([a, b], top) = st.popn_top::<2>();
            assert_eq!(a, U256::from(5));
            assert_eq!(b, U256::from(4));
            assert_eq!(*top, U256::from(3));

            *top = U256::from(100);
            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(0).unwrap(), U256::from(100));
    }

    #[test]
    fn test_stack_top_swap() {
        let mut stack = Stack::new();
        let _ = stack.push(U256::from(1));
        let _ = stack.push(U256::from(2));
        let _ = stack.push(U256::from(3));

        unsafe {
            let mut st = stack.as_stack_top();

            st.swap(1);
            assert_eq!(*st.get(0), U256::from(2));
            assert_eq!(*st.get(1), U256::from(3));
            assert_eq!(*st.get(2), U256::from(1));

            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.peek(0).unwrap(), U256::from(2));
        assert_eq!(stack.peek(1).unwrap(), U256::from(3));
    }

    #[test]
    fn test_stack_top_dup() {
        let mut stack = Stack::new();
        let _ = stack.push(U256::from(1));
        let _ = stack.push(U256::from(2));

        unsafe {
            let mut st = stack.as_stack_top();

            st.dup(2);
            assert_eq!(st.len(), 3);
            assert_eq!(*st.get(0), U256::from(1));
            assert_eq!(*st.get(1), U256::from(2));
            assert_eq!(*st.get(2), U256::from(1));

            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.peek(0).unwrap(), U256::from(1));
    }

    #[test]
    fn test_stack_top_is_empty() {
        let mut stack = Stack::new();

        unsafe {
            let st = stack.as_stack_top();
            assert!(st.is_empty());
        }

        let _ = stack.push(U256::from(1));

        unsafe {
            let st = stack.as_stack_top();
            assert!(!st.is_empty());
        }
    }

    #[test]
    fn test_stack_top_get_mut() {
        let mut stack = Stack::new();
        let _ = stack.push(U256::from(1));
        let _ = stack.push(U256::from(2));

        unsafe {
            let mut st = stack.as_stack_top();

            *st.get_mut(1) = U256::from(100);
            assert_eq!(*st.get(1), U256::from(100));

            st.write_back_len(stack.data_mut());
        }

        assert_eq!(stack.peek(1).unwrap(), U256::from(100));
    }
}
