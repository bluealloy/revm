//! Local context that is filled by execution.
use context_interface::{LocalContextTr, SharedMemoryBuffer};
use std::string::String;

/// Local context that is filled by execution.
#[derive(Clone, Debug)]
pub struct LocalContext {
    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    pub shared_memory_buffer: SharedMemoryBuffer,
    /// Optional precompile error message to bubble up.
    pub precompile_error_message: Option<String>,
}

impl Default for LocalContext {
    fn default() -> Self {
        Self {
            shared_memory_buffer: SharedMemoryBuffer::with_capacity(1024 * 4),
            precompile_error_message: None,
        }
    }
}

impl LocalContextTr for LocalContext {
    fn clear(&mut self) {
        // Sets len to 0 but it will not shrink to drop the capacity.
        self.shared_memory_buffer.clear();
        self.precompile_error_message = None;
    }

    fn shared_memory_buffer(&self) -> &SharedMemoryBuffer {
        &self.shared_memory_buffer
    }

    fn set_precompile_error_context(&mut self, output: String) {
        self.precompile_error_message = Some(output);
    }

    fn take_precompile_error_context(&mut self) -> Option<String> {
        self.precompile_error_message.take()
    }
}

impl LocalContext {
    /// Creates a new local context with default values.
    ///
    /// Initializes a shared memory buffer with 4KB capacity and no precompile error message.
    pub fn new() -> Self {
        Self::default()
    }
}
