//! KZG-RS implementation for KZG operations

use kzg_rs::KzgProof;

/// Verify KZG proof using KZG-RS library.
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    let env = kzg_rs::EnvKzgSettings::default();
    let kzg_settings = env.get();
    KzgProof::verify_kzg_proof(
        super::as_bytes48(commitment),
        super::as_bytes32(z),
        super::as_bytes32(y),
        super::as_bytes48(proof),
        kzg_settings,
    )
    .unwrap_or(false)
}
