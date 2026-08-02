//! KZG point evaluation precompile using BLST BLS12-381 implementation.
use crate::{
    bls12_381::blst::{
        p1_add_or_double, p1_from_affine, p1_scalar_mul, p1_to_affine, p2_from_affine,
        p2_to_affine, pairing_check,
    },
    bls12_381_const::TRUSTED_SETUP_TAU_G2_BYTES,
    PrecompileHalt,
};
use ::blst::{
    blst_p1_affine, blst_p1_affine_in_g1, blst_p1_affine_is_inf, blst_p1_affine_on_curve,
    blst_p2_affine, blst_p2_affine_is_inf, blst_scalar, blst_scalar_fr_check,
    blst_scalar_from_bendian,
};
use primitives::OnceLock;
use std::vec::Vec;

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
    // Parse the commitment and proof (G1 points)
    let Ok(commitment_point) = parse_g1_compressed(commitment) else {
        return false;
    };
    let Ok(proof_point) = parse_g1_compressed(proof) else {
        return false;
    };

    // Parse z and y as canonical scalar field elements (Fr)
    let Ok(z_scalar) = read_scalar_canonical(z) else {
        return false;
    };
    let Ok(y_scalar) = read_scalar_canonical(y) else {
        return false;
    };

    // Get generators and the trusted setup point [τ]G₂.
    let g1 = get_g1_generator();
    let g2 = get_g2_generator();
    let tau_g2 = get_trusted_setup_g2();

    // Reformulated single-proof KZG check. Starting from the standard
    //   e(commitment - [y]G1, -G2) · e(proof, [τ]G2 - [z]G2) == 1
    // and applying bilinearity `e(proof, -[z]G2) = e([z]·proof, -G2)` to move z
    // out of G2, both G2 pairing inputs become the constants -G2 and [τ]G2:
    //   e(D, -G2) · e(proof, [τ]G2) == 1,   D = commitment - [y]G1 + [z]·proof
    // This replaces the per-call G2 scalar multiplication (and the G2 subtraction
    // and its Fp2 inversion) with a single cheaper G1 scalar multiplication,
    // while still feeding the fused `pairing_check` (`blst_miller_loop_n`).
    let y_g1 = p1_scalar_mul(&g1, &y_scalar);
    let z_proof = p1_scalar_mul(&proof_point, &z_scalar);
    let neg_y_g1 = p1_neg(&y_g1);

    // D = commitment + (-[y]G1) + [z]·proof, accumulated in Jacobian coordinates
    // with a single normalization to affine. `blst_p1_add_or_double_affine`
    // handles infinity operands (e.g. y = 0 gives [y]G1 = ∞).
    let mut acc = p1_from_affine(&commitment_point);
    acc = p1_add_or_double(&acc, &neg_y_g1);
    acc = p1_add_or_double(&acc, &z_proof);
    let d = p1_to_affine(&acc);

    let neg_g2 = p2_neg(&g2);

    // Skip pairs containing a point at infinity: their pairing is the identity,
    // and `pairing_check` requires infinity-free inputs (`blst_miller_loop_n`,
    // unlike the per-pair `blst_miller_loop`, does not special-case infinity).
    // Only `d`/`proof` (G1) can be infinity here; the G2 points -G2 and [τ]G2 are
    // fixed and never infinity, but the check stays uniform with the general
    // `pairing_check` contract.
    let pairs: Vec<_> = [(d, neg_g2), (proof_point, *tau_g2)]
        .into_iter()
        // SAFETY: both arguments are valid blst types
        .filter(|(g1, g2)| unsafe { !blst_p1_affine_is_inf(g1) && !blst_p2_affine_is_inf(g2) })
        .collect();

    pairing_check(&pairs)
}

/// Get the trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> &'static blst_p2_affine {
    static TAU_G2: OnceLock<blst_p2_affine> = OnceLock::new();
    TAU_G2.get_or_init(|| {
        // For compressed G2, we need to decompress
        let mut g2_affine = blst_p2_affine::default();
        unsafe {
            // The compressed format has x coordinate and a flag bit for y
            // We use uncompress which handles this automatically
            let result =
                blst::blst_p2_uncompress(&mut g2_affine, TRUSTED_SETUP_TAU_G2_BYTES.as_ptr());
            if result != blst::BLST_ERROR::BLST_SUCCESS {
                panic!("Failed to deserialize trusted setup G2 point");
            }
        }
        g2_affine
    })
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
fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<blst_p1_affine, PrecompileHalt> {
    let mut point = blst_p1_affine::default();
    unsafe {
        let result = blst::blst_p1_uncompress(&mut point, bytes.as_ptr());
        if result != blst::BLST_ERROR::BLST_SUCCESS {
            return Err(PrecompileHalt::KzgInvalidG1Point);
        }

        // Verify the point is on curve
        if !blst_p1_affine_on_curve(&point) {
            return Err(PrecompileHalt::KzgG1PointNotOnCurve);
        }

        // Verify the point is in the correct subgroup
        if !blst_p1_affine_in_g1(&point) {
            return Err(PrecompileHalt::KzgG1PointNotInSubgroup);
        }
    }
    Ok(point)
}

/// Read a scalar field element from bytes and verify it's canonical
fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<blst_scalar, PrecompileHalt> {
    let mut scalar = blst_scalar::default();

    // Read scalar from big endian bytes
    unsafe {
        blst_scalar_from_bendian(&mut scalar, bytes.as_ptr());
    }

    if unsafe { !blst_scalar_fr_check(&scalar) } {
        return Err(PrecompileHalt::NonCanonicalFp);
    }

    Ok(scalar)
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
