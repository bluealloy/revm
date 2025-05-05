use core::{
    cell::{Ref, RefCell},
    ops::Range,
};
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
}
