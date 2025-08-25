//! KZG point evaluation precompile using BLST BLS12-381 implementation.
use crate::bls12_381::blst::{
    p1_add_or_double, p1_from_affine, p1_scalar_mul, p1_to_affine, p2_add_or_double,
    p2_from_affine, p2_scalar_mul, p2_to_affine, pairing_check,
};
use crate::bls12_381_const::TRUSTED_SETUP_TAU_G2_BYTES;
use crate::PrecompileError;
use ::blst::{
    blst_p1_affine, blst_p1_affine_in_g1, blst_p1_affine_on_curve, blst_p2_affine, blst_scalar,
    blst_scalar_fr_check, blst_scalar_from_bendian,
};

/// Verify KZG proof using BLST BLS12-381 implementation.
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
    let Ok(z_scalar) = read_scalar_canonical(z) else {
        return false;
    };
    let Ok(y_scalar) = read_scalar_canonical(y) else {
        return false;
    };

    // Get the trusted setup G2 point [τ]₂
    let tau_g2 = get_trusted_setup_g2();

    // Get generators
    let g1 = get_g1_generator();
    let g2 = get_g2_generator();

    // Compute P_minus_y = commitment - [y]G₁
    let y_g1 = p1_scalar_mul(&g1, &y_scalar);
    let p_minus_y = p1_sub_affine(&commitment_point, &y_g1);

    // Compute X_minus_z = [τ]G₂ - [z]G₂
    let z_g2 = p2_scalar_mul(&g2, &z_scalar);
    let x_minus_z = p2_sub_affine(&tau_g2, &z_g2);

    // Verify: P - y = Q * (X - z)
    // Using pairing check: e(P - y, -G₂) * e(proof, X - z) == 1
    let neg_g2 = p2_neg(&g2);

    pairing_check(&[(p_minus_y, neg_g2), (proof_point, x_minus_z)])
}

/// Get the trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> blst_p2_affine {
    // For compressed G2, we need to decompress
    let mut g2_affine = blst_p2_affine::default();
    unsafe {
        // The compressed format has x coordinate and a flag bit for y
        // We use deserialize_compressed which handles this automatically
        let result = blst::blst_p2_deserialize(&mut g2_affine, TRUSTED_SETUP_TAU_G2_BYTES.as_ptr());
        if result != blst::BLST_ERROR::BLST_SUCCESS {
            panic!("Failed to deserialize trusted setup G2 point");
        }
    }
    g2_affine
}

/// Get G1 generator point
fn get_g1_generator() -> blst_p1_affine {
    unsafe { ::blst::BLS12_381_G1 }
}

/// Get G2 generator point
fn get_g2_generator() -> blst_p2_affine {
    unsafe { ::blst::BLS12_381_G2 }
}

/// Parse a G1 point from compressed format (48 bytes)
fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<blst_p1_affine, PrecompileError> {
    let mut point = blst_p1_affine::default();
    unsafe {
        let result = blst::blst_p1_deserialize(&mut point, bytes.as_ptr());
        if result != blst::BLST_ERROR::BLST_SUCCESS {
            return Err(PrecompileError::KzgInvalidG1Point);
        }

        // Verify the point is on curve
        if !blst_p1_affine_on_curve(&point) {
            return Err(PrecompileError::KzgG1PointNotOnCurve);
        }

        // Verify the point is in the correct subgroup
        if !blst_p1_affine_in_g1(&point) {
            return Err(PrecompileError::KzgG1PointNotInSubgroup);
        }
    }
    Ok(point)
}

/// Read a scalar field element from bytes and verify it's canonical
fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<blst_scalar, PrecompileError> {
    let mut scalar = blst_scalar::default();

    // Read scalar from big endian bytes
    unsafe {
        blst_scalar_from_bendian(&mut scalar, bytes.as_ptr());
    }

    if unsafe { !blst_scalar_fr_check(&scalar) } {
        return Err(PrecompileError::NonCanonicalFp);
    }

    Ok(scalar)
}

/// Subtract two G1 points in affine form
fn p1_sub_affine(a: &blst_p1_affine, b: &blst_p1_affine) -> blst_p1_affine {
    // Convert first point to Jacobian
    let a_jacobian = p1_from_affine(a);

    // Negate second point
    let neg_b = p1_neg(b);

    // Add a + (-b)
    let result = p1_add_or_double(&a_jacobian, &neg_b);

    p1_to_affine(&result)
}

/// Subtract two G2 points in affine form
fn p2_sub_affine(a: &blst_p2_affine, b: &blst_p2_affine) -> blst_p2_affine {
    // Convert first point to Jacobian
    let a_jacobian = p2_from_affine(a);

    // Negate second point
    let neg_b = p2_neg(b);

    // Add a + (-b)
    let result = p2_add_or_double(&a_jacobian, &neg_b);

    p2_to_affine(&result)
}

/// Negate a G1 point
fn p1_neg(p: &blst_p1_affine) -> blst_p1_affine {
    // Convert to Jacobian, negate, convert back
    let mut p_jacobian = p1_from_affine(p);
    unsafe {
        ::blst::blst_p1_cneg(&mut p_jacobian, true);
    }
    p1_to_affine(&p_jacobian)
}

/// Negate a G2 point
fn p2_neg(p: &blst_p2_affine) -> blst_p2_affine {
    // Convert to Jacobian, negate, convert back
    let mut p_jacobian = p2_from_affine(p);
    unsafe {
        ::blst::blst_p2_cneg(&mut p_jacobian, true);
    }
    p2_to_affine(&p_jacobian)
}
