use crate::{SharedMemory, SharedStack, EMPTY_SHARED_MEMORY, EMPTY_SHARED_STACK};

pub const EMPTY_SHARED_CONTEXT: SharedContext = SharedContext {
    stack: EMPTY_SHARED_STACK,
    memory: EMPTY_SHARED_MEMORY,
};

/// The shared data between contexts.
/// Wraps [SharedStack] and [SharedMemory] wrapped in a struct
#[derive(Debug)]
pub struct SharedContext {
    /// Shared stack
    pub stack: SharedStack,
    /// Shared memory
    pub memory: SharedMemory,
}

impl Default for SharedContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SharedContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            stack: SharedStack::new(),
            memory: SharedMemory::new(),
        }
    }

    #[cfg(feature = "memory_limit")]
    #[inline]
    pub fn new_with_memory_limit(memory_limit: u64) -> Self {
        Self {
            stack: SharedStack::new(),
            memory: SharedMemory::new_with_memory_limit(memory_limit),
        }
    }

    /// Prepares the shared data for a new context
    #[inline]
    pub fn new_context(&mut self) {
        self.memory.new_context();
        self.stack.new_context();
    }

    /// Prepares the shared data for the previous context
    #[inline]
    pub fn free_context(&mut self) {
        self.memory.free_context();
        self.stack.free_context();
    }
}
