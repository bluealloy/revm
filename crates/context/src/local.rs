//! Local context that is filled by execution.
use context_interface::LocalContextTr;
use core::cell::RefCell;
use std::{rc::Rc, string::String, vec::Vec};

/// Local context that is filled by execution.
#[derive(Clone, Debug)]
pub struct LocalContext {
    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    pub shared_memory_buffer: Rc<RefCell<Vec<u8>>>,
    /// Optional precompile error message to bubble up.
    precompile_error_message: Option<String>,
}

impl Default for LocalContext {
    fn default() -> Self {
        Self {
            shared_memory_buffer: Rc::new(RefCell::new(Vec::with_capacity(1024 * 4))),
            precompile_error_message: None,
        }
    }
}

impl LocalContextTr for LocalContext {
    fn clear(&mut self) {
        // Sets len to 0 but it will not shrink to drop the capacity.
        unsafe { self.shared_memory_buffer.borrow_mut().set_len(0) };
        self.precompile_error_message = None;
    }

    fn shared_memory_buffer(&self) -> &Rc<RefCell<Vec<u8>>> {
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
    /// Creates a new local context, initcodes are hashes and added to the mapping.
    pub fn new() -> Self {
        Self::default()
    }
}
