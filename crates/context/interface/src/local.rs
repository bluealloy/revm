use primitives::{Bytes, B256};

/// Local context used for caching initcode from Initcode transactions.
pub trait LocalContextTr {
    /// Get the local context
    fn insert_initcodes(&mut self, initcodes: &[Bytes]);
    /// Get validated initcode by hash. if initcode is not validated it is assumed
    /// that validation is going to be performed inside this function.
    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes>;
    /// Clear the local context.
    fn clear(&mut self);
}
