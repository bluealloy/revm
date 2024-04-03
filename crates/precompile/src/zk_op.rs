use std::sync::OnceLock;

use once_cell::{race::OnceBox, sync::Lazy};
use revm_primitives::Bytes;
use crate::Error;

pub static ZKVM_OPERATIONS:  Lazy<OnceBox<Vec<Operation>>>  = Lazy::new(OnceBox::<Vec::<Operation>>::new);
pub static ZKVM_OPERATOR: OnceLock<Box<dyn ZkvmOperator>> = OnceLock::new();

pub fn contains_operation(op: &Operation) -> bool {
    ZKVM_OPERATIONS
        .get()
        .expect("ZKVM_OPERATIONS unset")
        .contains(&op)
}


#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Bn128Add,
    Bn128Mul,
    Bn128Pairing,
}

pub trait ZkvmOperator: Send + Sync {
    fn bn128_run_add(&self, input: &[u8]) -> Result<Bytes, Error>;
    fn bn128_run_mul(&self, input: &[u8]) -> Result<Bytes, Error>;
    fn bn128_run_pairing(&self, input: &[u8]) -> Result<bool, Error>;

}