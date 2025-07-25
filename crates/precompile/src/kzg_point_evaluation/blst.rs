use crate::PrecompileError;
use ::blst::{
    blst_fp12, blst_fp12_is_one, blst_fp12_mul, blst_miller_loop, blst_p1, blst_p1_affine,
    blst_p1_affine_in_g1, blst_p1_affine_on_curve, blst_p1_from_affine, blst_p1_mult,
    blst_p1_to_affine, blst_p2_affine, blst_scalar, blst_scalar_from_bendian,
};
use primitives::hex_literal::hex;
use std::string::ToString;

/// Verify KZG proof using BLST BLS12-381 implementation.
///
/// https://github.com/ethereum/EIPs/blob/4d2a00692bb131366ede1a16eced2b0e25b1bf99/EIPS/eip-4844.md?plain=1#L203
/// https://github.com/ethereum/consensus-specs/blob/master/specs/deneb/polynomial-commitments.md#verify_kzg_proof_impl
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
    let y_g1 = scalar_mul_g1(&g1, &y_scalar);
    let p_minus_y = p1_sub_affine(&commitment_point, &y_g1);

    // Compute X_minus_z = [τ]G₂ - [z]G₂
    let z_g2 = scalar_mul_g2(&g2, &z_scalar);
    let x_minus_z = p2_sub_affine(&tau_g2, &z_g2);

    // Verify: P - y = Q * (X - z)
    // Using pairing_check([[P_minus_y, -G₂], [proof, X_minus_z]]) == 1
    let neg_g2 = p2_neg(&g2);

    // Compute miller loops
    let ml1 = compute_miller_loop(&p_minus_y, &neg_g2);
    let ml2 = compute_miller_loop(&proof_point, &x_minus_z);

    // Multiply miller loop results
    let mut acc = blst_fp12::default();
    unsafe { blst_fp12_mul(&mut acc, &ml1, &ml2) };

    // Apply final exponentiation and check if result is 1
    let mut final_result = blst_fp12::default();
    unsafe {
        blst::blst_final_exp(&mut final_result, &acc);
    }

    // Check if the result is one (identity element)
    unsafe { blst_fp12_is_one(&final_result) }
}

/// Get the trusted setup G2 point [τ]₂ from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> blst_p2_affine {
    // The trusted setup G2 point [τ]₂ from the Ethereum KZG ceremony (compressed format)
    // Taken from: https://github.com/ethereum/consensus-specs/blob/adc514a1c29532ebc1a67c71dc8741a2fdac5ed4/presets/mainnet/trusted_setups/trusted_setup_4096.json#L8200C6-L8200C200
    const TRUSTED_SETUP_TAU_G2_BYTES: &[u8; 96] = &hex!(
        "b5bfd7dd8cdeb128843bc287230af38926187075cbfbefa81009a2ce615ac53d2914e5870cb452d2afaaab24f3499f72185cbfee53492714734429b7b38608e23926c911cceceac9a36851477ba4c60b087041de621000edc98edada20c1def2"
    );

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
            return Err(PrecompileError::Other(
                "Invalid compressed G1 point".to_string(),
            ));
        }

        // Verify the point is on curve
        if !blst_p1_affine_on_curve(&point) {
            return Err(PrecompileError::Other("G1 point not on curve".to_string()));
        }

        // Verify the point is in the correct subgroup
        if !blst_p1_affine_in_g1(&point) {
            return Err(PrecompileError::Other(
                "G1 point not in correct subgroup".to_string(),
            ));
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

    // Check if the scalar is canonical by converting back and comparing
    let mut bytes_roundtrip = [0u8; 32];
    unsafe {
        blst::blst_bendian_from_scalar(bytes_roundtrip.as_mut_ptr(), &scalar);
    }

    if bytes_roundtrip != *bytes {
        return Err(PrecompileError::Other(
            "Non-canonical scalar field element".to_string(),
        ));
    }

    Ok(scalar)
}

/// Scalar multiplication for G1 points
fn scalar_mul_g1(point: &blst_p1_affine, scalar: &blst_scalar) -> blst_p1_affine {
    let p_jacobian = p1_from_affine(point);
    let mut result = blst_p1::default();

    unsafe {
        blst_p1_mult(
            &mut result,
            &p_jacobian,
            scalar.b.as_ptr(),
            scalar.b.len() * 8,
        );
    }

    p1_to_affine(&result)
}

/// Scalar multiplication for G2 points
fn scalar_mul_g2(point: &blst_p2_affine, scalar: &blst_scalar) -> blst_p2_affine {
    let p_jacobian = p2_from_affine(point);
    let mut result = ::blst::blst_p2::default();

    unsafe {
        ::blst::blst_p2_mult(
            &mut result,
            &p_jacobian,
            scalar.b.as_ptr(),
            scalar.b.len() * 8,
        );
    }

    p2_to_affine(&result)
}

/// Subtract two G1 points in affine form
fn p1_sub_affine(a: &blst_p1_affine, b: &blst_p1_affine) -> blst_p1_affine {
    // Convert first point to Jacobian
    let a_jacobian = p1_from_affine(a);

    // Negate second point
    let neg_b = p1_neg(b);

    // Add a + (-b)
    let mut result = blst_p1::default();
    unsafe {
        blst::blst_p1_add_or_double_affine(&mut result, &a_jacobian, &neg_b);
    }

    p1_to_affine(&result)
}

/// Subtract two G2 points in affine form
fn p2_sub_affine(a: &blst_p2_affine, b: &blst_p2_affine) -> blst_p2_affine {
    // Convert first point to Jacobian
    let a_jacobian = p2_from_affine(a);

    // Negate second point
    let neg_b = p2_neg(b);

    // Add a + (-b)
    let mut result = ::blst::blst_p2::default();
    unsafe {
        blst::blst_p2_add_or_double_affine(&mut result, &a_jacobian, &neg_b);
    }

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

/// Convert affine to Jacobian for G1
fn p1_from_affine(p_affine: &blst_p1_affine) -> blst_p1 {
    let mut p = blst_p1::default();
    unsafe {
        blst_p1_from_affine(&mut p, p_affine);
    }
    p
}

/// Convert Jacobian to affine for G1
fn p1_to_affine(p: &blst_p1) -> blst_p1_affine {
    let mut p_affine = blst_p1_affine::default();
    unsafe {
        blst_p1_to_affine(&mut p_affine, p);
    }
    p_affine
}

/// Convert affine to Jacobian for G2
fn p2_from_affine(p_affine: &blst_p2_affine) -> blst::blst_p2 {
    let mut p = blst::blst_p2::default();
    unsafe {
        blst::blst_p2_from_affine(&mut p, p_affine);
    }
    p
}

/// Convert Jacobian to affine for G2
fn p2_to_affine(p: &blst::blst_p2) -> blst_p2_affine {
    let mut p_affine = blst_p2_affine::default();
    unsafe {
        blst::blst_p2_to_affine(&mut p_affine, p);
    }
    p_affine
}

/// Compute miller loop for pairing
fn compute_miller_loop(g1: &blst_p1_affine, g2: &blst_p2_affine) -> blst_fp12 {
    let mut result = blst_fp12::default();
    unsafe {
        blst_miller_loop(&mut result, g2, g1);
    }
    result
}
