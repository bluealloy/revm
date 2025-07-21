//! KZG (Kate-Zaverucha-Goldberg) point evaluation

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        use c_kzg::{Bytes32, Bytes48};
    } else if #[cfg(feature = "kzg-rs")] {
        use kzg_rs::{Bytes32, Bytes48, KzgProof};
    }
}

/// Verify KZG proof.
#[inline]
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    cfg_if::cfg_if! {
        if #[cfg(feature = "c-kzg")] {
            let kzg_settings = c_kzg::ethereum_kzg_settings(8);
            kzg_settings.verify_kzg_proof(
                &Bytes48::from(*commitment), 
                &Bytes32::from(*z), 
                &Bytes32::from(*y), 
                &Bytes48::from(*proof)
            ).unwrap_or(false)
        } else if #[cfg(feature = "kzg-rs")] {
            let env = kzg_rs::EnvKzgSettings::default();
            let kzg_settings = env.get();
            KzgProof::verify_kzg_proof(
                Bytes48::from(*commitment),
                Bytes32::from(*z),
                Bytes32::from(*y),
                Bytes48::from(*proof),
                kzg_settings
            ).unwrap_or(false)
        }
    }
}