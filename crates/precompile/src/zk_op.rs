use revm_primitives::Bytes;
use crate::Error;

pub trait ZkvmOperator: Send + Sync {
    fn bn128_run_add(&self, input: &[u8]) -> Result<Bytes, Error>;
}