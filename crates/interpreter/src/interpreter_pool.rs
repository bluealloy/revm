//! Interpreter pooling by call depth for reduced allocation overhead.
//!
//! This module provides [`InterpreterPool`], which pools and reuses [`Interpreter`]
//! instances by call depth. This optimization is inspired by evmone's ExecutionState
//! reuse pattern, avoiding repeated allocation/initialization for nested calls.
//!
//! # Example
//!
//! ```ignore
//! use revm_interpreter::{InterpreterPool, Interpreter, EthInterpreter};
//!
//! let mut pool = InterpreterPool::<EthInterpreter>::new();
//!
//! // Get or create an interpreter at depth 0
//! let interp = pool.get_or_create(0);
//! // Use the interpreter...
//!
//! // Later, at depth 1
//! let interp = pool.get_or_create(1);
//! // The interpreter at depth 0 is still available
//! ```

use crate::interpreter::{EthInterpreter, Interpreter};
use crate::interpreter_types::InterpreterTypes;
use std::vec::Vec;

/// Maximum EVM call depth plus one for the initial frame.
pub const MAX_CALL_DEPTH_POOL_SIZE: usize = 1025;

/// A pool of interpreters indexed by call depth.
///
/// This structure pools [`Interpreter`] instances to avoid repeated allocations
/// during nested EVM calls. Each call depth level has its own interpreter slot
/// that can be reused across calls.
///
/// The pool grows lazily as deeper call depths are reached, but capacity is
/// pre-reserved to avoid reallocation during execution.
pub struct InterpreterPool<WIRE: InterpreterTypes = EthInterpreter> {
    /// Pool of interpreters, indexed by call depth.
    /// `None` indicates the slot hasn't been used yet.
    pool: Vec<Option<Interpreter<WIRE>>>,
}

impl<WIRE: InterpreterTypes> core::fmt::Debug for InterpreterPool<WIRE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InterpreterPool")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}

impl<WIRE: InterpreterTypes> Default for InterpreterPool<WIRE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<WIRE: InterpreterTypes> Clone for InterpreterPool<WIRE>
where
    Interpreter<WIRE>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl<WIRE: InterpreterTypes> InterpreterPool<WIRE> {
    /// Creates a new empty interpreter pool with pre-reserved capacity.
    ///
    /// The pool reserves capacity for [`MAX_CALL_DEPTH_POOL_SIZE`] interpreters
    /// to avoid reallocation during execution.
    #[inline]
    pub fn new() -> Self {
        Self {
            pool: Vec::with_capacity(MAX_CALL_DEPTH_POOL_SIZE),
        }
    }

    /// Creates a new interpreter pool with a custom capacity.
    ///
    /// Use this when you know the maximum call depth will be less than the
    /// default to save memory.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of initialized interpreters in the pool.
    #[inline]
    pub fn len(&self) -> usize {
        self.pool.iter().filter(|slot| slot.is_some()).count()
    }

    /// Returns true if no interpreters have been initialized.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pool.iter().all(|slot| slot.is_none())
    }

    /// Returns the current capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }

    /// Ensures the pool has at least `depth + 1` slots.
    ///
    /// This resizes the internal vector if necessary, similar to evmone's
    /// `get_execution_state` pattern.
    #[inline]
    fn ensure_depth(&mut self, depth: usize) {
        if self.pool.len() <= depth {
            self.pool.resize_with(depth + 1, || None);
        }
    }

    /// Returns a mutable reference to the interpreter slot at the given depth.
    ///
    /// Returns `None` if no interpreter has been placed at this depth yet.
    #[inline]
    pub fn get(&mut self, depth: usize) -> Option<&mut Interpreter<WIRE>> {
        self.ensure_depth(depth);
        self.pool[depth].as_mut()
    }

    /// Takes the interpreter from the given depth, leaving `None` in its place.
    ///
    /// This is useful when you need to move the interpreter elsewhere.
    #[inline]
    pub fn take(&mut self, depth: usize) -> Option<Interpreter<WIRE>> {
        if depth < self.pool.len() {
            self.pool[depth].take()
        } else {
            None
        }
    }

    /// Places an interpreter at the given depth.
    ///
    /// Returns the previous interpreter at that depth, if any.
    #[inline]
    pub fn put(
        &mut self,
        depth: usize,
        interpreter: Interpreter<WIRE>,
    ) -> Option<Interpreter<WIRE>> {
        self.ensure_depth(depth);
        self.pool[depth].replace(interpreter)
    }

    /// Clears all interpreters from the pool without deallocating.
    ///
    /// After calling this, all slots will be `None` but capacity is preserved.
    #[inline]
    pub fn clear(&mut self) {
        for slot in &mut self.pool {
            *slot = None;
        }
    }
}

impl<EXT: Default> InterpreterPool<EthInterpreter<EXT>> {
    /// Gets an existing interpreter at the given depth or creates a new one.
    ///
    /// If an interpreter already exists at this depth, returns a mutable reference
    /// to it. Otherwise, creates a new interpreter using [`Interpreter::default_ext`]
    /// and stores it in the pool.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut pool = InterpreterPool::<EthInterpreter>::new();
    /// let interp = pool.get_or_create(0);
    /// // interp is ready for use, either reused or newly created
    /// ```
    #[inline]
    pub fn get_or_create(&mut self, depth: usize) -> &mut Interpreter<EthInterpreter<EXT>> {
        self.ensure_depth(depth);
        self.pool[depth].get_or_insert_with(Interpreter::default_ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::EthInterpreter;

    #[test]
    fn test_new_pool() {
        let pool = InterpreterPool::<EthInterpreter>::new();
        assert!(pool.is_empty());
        assert_eq!(pool.capacity(), MAX_CALL_DEPTH_POOL_SIZE);
    }

    #[test]
    fn test_get_or_create() {
        let mut pool = InterpreterPool::<EthInterpreter>::new();

        // First access creates a new interpreter
        let _interp = pool.get_or_create(0);
        assert_eq!(pool.len(), 1);

        // Second access at same depth reuses
        let _interp = pool.get_or_create(0);
        assert_eq!(pool.len(), 1);

        // Access at new depth creates new interpreter
        let _interp = pool.get_or_create(5);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_get_returns_existing() {
        let mut pool = InterpreterPool::<EthInterpreter>::new();

        // No interpreter at depth 0 yet
        assert!(pool.get(0).is_none());

        // Create one
        let _interp = pool.get_or_create(0);

        // Now it exists
        assert!(pool.get(0).is_some());
    }

    #[test]
    fn test_take_and_put() {
        let mut pool = InterpreterPool::<EthInterpreter>::new();

        // Create at depth 0
        let _interp = pool.get_or_create(0);
        assert!(pool.get(0).is_some());

        // Take it
        let taken = pool.take(0);
        assert!(taken.is_some());
        assert!(pool.get(0).is_none());

        // Put it back
        pool.put(0, taken.unwrap());
        assert!(pool.get(0).is_some());
    }

    #[test]
    fn test_clear() {
        let mut pool = InterpreterPool::<EthInterpreter>::new();

        // Create several interpreters
        let _interp = pool.get_or_create(0);
        let _interp = pool.get_or_create(1);
        let _interp = pool.get_or_create(2);
        assert_eq!(pool.len(), 3);

        // Clear
        pool.clear();
        assert!(pool.is_empty());

        // Capacity preserved
        assert_eq!(pool.capacity(), MAX_CALL_DEPTH_POOL_SIZE);
    }

    #[test]
    fn test_deep_depth() {
        let mut pool = InterpreterPool::<EthInterpreter>::new();

        // Access at a deep depth
        let _interp = pool.get_or_create(100);
        assert_eq!(pool.len(), 1);

        // Pool should have resized appropriately
        assert!(pool.pool.len() > 100);
    }
}
