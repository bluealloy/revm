use core::cell::RefCell;
use primitives::{Bytes, B256};
use std::{rc::Rc, vec::Vec};

/// Local context used for caching initcode from Initcode transactions.
pub trait LocalContextTr {
    /// Get the local context
    fn insert_initcodes(&mut self, initcodes: &[Bytes]);
    /// Get validated initcode by hash. if initcode is not validated it is assumed
    /// that validation is going to be performed inside this function.
    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes>;
    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    fn shared_memory_buffer(&mut self) -> &Rc<RefCell<Vec<u8>>>;
    /// Clear the local context.
    fn clear(&mut self);
}
