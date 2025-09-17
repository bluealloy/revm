//! KZG point evaluation cryptographic operations
//! 
//! This module contains the pure cryptographic implementations for KZG point evaluation precompiles.
//! These functions are called by the Crypto trait.

use crate::PrecompileError;

/// KZG point evaluation proof verification
pub fn verify_kzg_proof(
    z: &[u8; 32],
    y: &[u8; 32],
    commitment: &[u8; 48],
    proof: &[u8; 48],
) -> Result<(), PrecompileError> {
    if !crate::kzg_point_evaluation::verify_kzg_proof(commitment, z, y, proof) {
        return Err(PrecompileError::BlobVerifyKzgProofFailed);
    }

    Ok(())
}
