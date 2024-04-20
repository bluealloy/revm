use std::sync::OnceLock;

use crate::Error;
use once_cell::{race::OnceBox, sync::Lazy};

pub static ZKVM_OPERATIONS: Lazy<OnceBox<Vec<ZkOperation>>> =
    Lazy::new(OnceBox::<Vec<ZkOperation>>::new);
pub static ZKVM_OPERATOR: OnceLock<Box<dyn ZkvmOperator>> = OnceLock::new();

pub fn contains_operation(op: &ZkOperation) -> bool {
    ZKVM_OPERATIONS.get().is_some_and(|ops| ops.contains(op))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZkOperation {
    Bn128Add,
    Bn128Mul,
    Bn128Pairing,
    Blake2,
    Sha256,
    Ripemd160,
    Modexp,
    Secp256k1,
    VerifyKzg,
}

// TODO(Cecilia): figure out best data types for each ZkVM
// endianess etc.
pub trait ZkvmOperator: Send + Sync + 'static {
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
    // fn verify_kzg_proof(
    //     &self,
    //     commitment_bytes: &[u8; 48],
    //     z_bytes: &[u8; 32],
    //     y_bytes: &[u8; 32],
    //     proof_bytes: &[u8; 48],
    //     G1: &[[u32; BYTES_PER_G1_POINT]; NUM_G1_POINTS],
    //     G2: &[[u32; BYTES_PER_G2_POINT]; NUM_G2_POINTS],
    // ) -> Result<bool, Error>;
}

// pub fn kzg_setting_to_points(
//     kzg_settings: &KzgSettings,
// ) -> (
//     [[u32; BYTES_PER_G1_POINT]; NUM_G1_POINTS],
//     [[u32; BYTES_PER_G2_POINT]; NUM_G2_POINTS],
// ) {
//     // TODO(Cecilia): figure out what's the best interface for trusted setup in common ZKVM
//     todo!()
// }
