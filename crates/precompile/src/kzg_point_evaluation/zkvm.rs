//! zkVM implementation of KZG point evaluation.

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        use c_kzg::{Bytes32, Bytes48};
    } else if #[cfg(feature = "kzg-rs")] {
        use kzg_rs::{Bytes32, Bytes48};
    }
}

extern "C" {
    /// zkVM implementation of KZG proof verification.
    ///
    /// # Arguments
    /// * `commitment_ptr` - Pointer to 48-byte commitment
    /// * `z_ptr` - Pointer to 32-byte evaluation point
    /// * `y_ptr` - Pointer to 32-byte evaluation result  
    /// * `proof_ptr` - Pointer to 48-byte KZG proof
    ///
    /// # Returns
    /// * 1 if proof is valid
    /// * 0 if proof is invalid
    fn zkvm_verify_kzg_proof_impl(
        commitment_ptr: *const u8,
        z_ptr: *const u8,
        y_ptr: *const u8,
        proof_ptr: *const u8,
    ) -> i32;
}

/// Verify KZG proof using zkVM implementation.
///
/// This function provides a hook for zkVM-optimized KZG proof verification.
/// The external implementation should handle polynomial commitment verification
/// and return 1 for valid proofs, 0 for invalid proofs.
pub fn verify_kzg_proof(commitment: &Bytes48, z: &Bytes32, y: &Bytes32, proof: &Bytes48) -> bool {
    let result = unsafe {
        zkvm_verify_kzg_proof_impl(commitment.as_ptr(), z.as_ptr(), y.as_ptr(), proof.as_ptr())
    };

    result == 1
}
