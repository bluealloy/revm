use crate::{SharedMemory, SharedStack, EMPTY_SHARED_MEMORY, EMPTY_SHARED_STACK};

pub const EMPTY_SHARED_CONTEXT: SharedContext = SharedContext {
    stack: EMPTY_SHARED_STACK,
    memory: EMPTY_SHARED_MEMORY,
};

#[derive(Debug)]
pub struct SharedContext {
    pub stack: SharedStack,
    pub memory: SharedMemory,
}

impl Default for SharedContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            stack: SharedStack::new(),
            memory: SharedMemory::new(),
        }
    }

    pub fn new_with_memory_limit(_memory_limit: u64) -> Self {
        Self {
            stack: SharedStack::new(),
            // memory: SharedMemory::new_with_memory_limit(memory_limit),
            memory: SharedMemory::new(),
        }
    }

    pub fn new_context(&mut self) {
        self.memory.new_context();
        self.stack.new_context();
    }

    pub fn free_context(&mut self) {
        self.memory.free_context();
        self.stack.free_context();
    }
}
