use std::sync::OnceLock;

use crate::Error;
use once_cell::{race::OnceBox, sync::Lazy};
use revm_primitives::Bytes;

pub static ZKVM_OPERATIONS: Lazy<OnceBox<Vec<Operation>>> =
    Lazy::new(OnceBox::<Vec<Operation>>::new);
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
    Blake2,
    Sha256,
    Ripemd160,
    Modexp,
    Secp256k1,
}

pub trait ZkvmOperator: Send + Sync {
    fn bn128_run_add(&self, input: &[u8]) -> Result<[u8; 64], Error>;
    fn bn128_run_mul(&self, input: &[u8]) -> Result<[u8; 64], Error>;
    fn bn128_run_pairing(&self, input: &[u8]) -> Result<bool, Error>;
    fn blake2_run(&self, input: &[u8]) -> Result<[u8; 64], Error>;
    fn sha256_run(&self, input: &[u8]) -> Result<[u8; 32], Error>;
    fn ripemd160_run(&self, input: &[u8]) -> Result<[u8; 32], Error>;
    fn modexp_run(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, Error>;
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], Error>;
}
