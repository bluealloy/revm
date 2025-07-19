//! C-KZG implementation for KZG operations

/// Verify KZG proof using C-KZG library.
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    let kzg_settings = c_kzg::ethereum_kzg_settings(8);
    kzg_settings
        .verify_kzg_proof(
            super::as_bytes48(commitment),
            super::as_bytes32(z),
            super::as_bytes32(y),
            super::as_bytes48(proof),
        )
        .unwrap_or(false)
}
