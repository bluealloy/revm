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
    pub precompile_error_message: Option<String>,
    /// EIP-7702 regular gas refund of the current transaction, set by the
    /// pre-execution phase.
    pub eip7702_refund: u64,
}

impl Default for LocalContext {
    fn default() -> Self {
        Self {
            shared_memory_buffer: Rc::new(RefCell::new(Vec::with_capacity(1024 * 4))),
            precompile_error_message: None,
            eip7702_refund: 0,
        }
    }
}

impl LocalContextTr for LocalContext {
    fn clear(&mut self) {
        // Sets len to 0 but it will not shrink to drop the capacity.
        unsafe { self.shared_memory_buffer.borrow_mut().set_len(0) };
        self.precompile_error_message = None;
        self.eip7702_refund = 0;
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

    fn set_eip7702_refund(&mut self, refund: u64) {
        self.eip7702_refund = refund;
    }

    fn eip7702_refund(&self) -> u64 {
        self.eip7702_refund
    }
}

impl LocalContext {
    /// Creates a new local context with default values.
    ///
    /// Initializes a shared memory buffer with 4KB capacity, no precompile error message
    /// and a zero EIP-7702 refund.
    pub fn new() -> Self {
        Self::default()
    }
}
