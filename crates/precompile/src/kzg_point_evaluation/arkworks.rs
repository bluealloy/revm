//! KZG point evaluation precompile using Arkworks BLS12-381 implementation.
use crate::bls12_381::arkworks::pairing_check;
use crate::bls12_381_const::TRUSTED_SETUP_TAU_G2_BYTES;
use crate::PrecompileError;
use ark_bls12_381::{Fr, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::CanonicalDeserialize;
use core::ops::Neg;

/// Verify KZG proof using BLS12-381 implementation.
///
/// <https://github.com/ethereum/EIPs/blob/4d2a00692bb131366ede1a16eced2b0e25b1bf99/EIPS/eip-4844.md?plain=1#L203>
/// <https://github.com/ethereum/consensus-specs/blob/master/specs/deneb/polynomial-commitments.md#verify_kzg_proof_impl>
#[inline]
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    // Parse the commitment (G1 point)
    let Ok(commitment_point) = parse_g1_compressed(commitment) else {
        return false;
    };

    // Parse the proof (G1 point)
    let Ok(proof_point) = parse_g1_compressed(proof) else {
        return false;
    };

    // Parse z and y as field elements (Fr, scalar field)
    // We expect 32-byte big-endian scalars that must be canonical
    let Ok(z_fr) = read_scalar_canonical(z) else {
        return false;
    };
    let Ok(y_fr) = read_scalar_canonical(y) else {
        return false;
    };

    // Get the trusted setup G2 point [τ]₂
    let tau_g2 = get_trusted_setup_g2();

    // Get generators
    let g1 = get_g1_generator();
    let g2 = get_g2_generator();

    // Compute P_minus_y = commitment - [y]G₁
    let y_g1 = p1_scalar_mul(&g1, &y_fr);
    let p_minus_y = p1_sub_affine(&commitment_point, &y_g1);

    // Compute X_minus_z = [τ]G₂ - [z]G₂
    let z_g2 = p2_scalar_mul(&g2, &z_fr);
    let x_minus_z = p2_sub_affine(&tau_g2, &z_g2);

    // Verify: P - y = Q * (X - z)
    // Using pairing check: e(P - y, -G₂) * e(proof, X - z) == 1
    let neg_g2 = p2_neg(&g2);

    pairing_check(&[(p_minus_y, neg_g2), (proof_point, x_minus_z)])
}

/// Get the trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> G2Affine {
    // Parse the compressed G2 point using unchecked deserialization since we trust this point
    // This should never fail since we're using a known valid point from the trusted setup
    G2Affine::deserialize_compressed_unchecked(&TRUSTED_SETUP_TAU_G2_BYTES[..])
        .expect("Failed to parse trusted setup G2 point")
}

/// Parse a G1 point from compressed format (48 bytes)
fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<G1Affine, PrecompileError> {
    G1Affine::deserialize_compressed(&bytes[..]).map_err(|_| PrecompileError::KzgInvalidG1Point)
}

/// Read a scalar field element from bytes and verify it's canonical
fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<Fr, PrecompileError> {
    let fr = Fr::from_be_bytes_mod_order(bytes);

    // Check if the field element is canonical by serializing back and comparing
    let bytes_roundtrip = fr.into_bigint().to_bytes_be();

    if bytes_roundtrip.as_slice() != bytes {
        return Err(PrecompileError::NonCanonicalFp);
    }

    Ok(fr)
}

/// Get G1 generator point
#[inline]
fn get_g1_generator() -> G1Affine {
    G1Affine::generator()
}

/// Get G2 generator point
#[inline]
fn get_g2_generator() -> G2Affine {
    G2Affine::generator()
}

/// Scalar multiplication for G1 points
#[inline]
fn p1_scalar_mul(point: &G1Affine, scalar: &Fr) -> G1Affine {
    point.mul_bigint(scalar.into_bigint()).into_affine()
}

/// Scalar multiplication for G2 points
#[inline]
fn p2_scalar_mul(point: &G2Affine, scalar: &Fr) -> G2Affine {
    point.mul_bigint(scalar.into_bigint()).into_affine()
}

/// Subtract two G1 points in affine form
#[inline]
fn p1_sub_affine(a: &G1Affine, b: &G1Affine) -> G1Affine {
    (a.into_group() - b.into_group()).into_affine()
}

/// Subtract two G2 points in affine form
#[inline]
fn p2_sub_affine(a: &G2Affine, b: &G2Affine) -> G2Affine {
    (a.into_group() - b.into_group()).into_affine()
}

/// Negate a G2 point
#[inline]
fn p2_neg(p: &G2Affine) -> G2Affine {
    p.neg()
}
