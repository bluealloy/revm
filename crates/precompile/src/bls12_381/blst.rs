// This module contains a safe wrapper around the blst library.

use crate::bls12_381_const::SCALAR_LENGTH;
use blst::{
    blst_final_exp, blst_fp, blst_fp12, blst_fp12_is_one, blst_fp12_mul, blst_fp2, blst_map_to_g1,
    blst_map_to_g2, blst_miller_loop, blst_p1, blst_p1_add_or_double_affine, blst_p1_affine,
    blst_p1_from_affine, blst_p1_mult, blst_p1_to_affine, blst_p2, blst_p2_add_or_double_affine,
    blst_p2_affine, blst_p2_from_affine, blst_p2_mult, blst_p2_to_affine, MultiPoint,
};

#[inline]
fn p1_to_affine(p: &blst_p1) -> blst_p1_affine {
    let mut p_affine = blst_p1_affine::default();
    // SAFETY: both inputs are valid blst types
    unsafe { blst_p1_to_affine(&mut p_affine, p) };
    p_affine
}

#[inline]
fn p1_from_affine(p_affine: &blst_p1_affine) -> blst_p1 {
    let mut p = blst_p1::default();
    // SAFETY: both inputs are valid blst types
    unsafe { blst_p1_from_affine(&mut p, p_affine) };
    p
}

#[inline]
fn p1_add_or_double(p: &blst_p1, p_affine: &blst_p1_affine) -> blst_p1 {
    let mut result = blst_p1::default();
    // SAFETY: all inputs are valid blst types
    unsafe { blst_p1_add_or_double_affine(&mut result, p, p_affine) };
    result
}

#[inline]
fn p2_to_affine(p: &blst_p2) -> blst_p2_affine {
    let mut p_affine = blst_p2_affine::default();
    // SAFETY: both inputs are valid blst types
    unsafe { blst_p2_to_affine(&mut p_affine, p) };
    p_affine
}

#[inline]
fn p2_from_affine(p_affine: &blst_p2_affine) -> blst_p2 {
    let mut p = blst_p2::default();
    // SAFETY: both inputs are valid blst types
    unsafe { blst_p2_from_affine(&mut p, p_affine) };
    p
}

#[inline]
fn p2_add_or_double(p: &blst_p2, p_affine: &blst_p2_affine) -> blst_p2 {
    let mut result = blst_p2::default();
    // SAFETY: all inputs are valid blst types
    unsafe { blst_p2_add_or_double_affine(&mut result, p, p_affine) };
    result
}

/// p1_add_affine adds two G1 points in affine form, returning the result in affine form
///
/// Note: `a` and `b` can be the same, ie this method is safe to call if one wants
/// to essentially double a point
#[inline]
pub(super) fn p1_add_affine(a: &blst_p1_affine, b: &blst_p1_affine) -> blst_p1_affine {
    // Convert first point to Jacobian coordinates
    let a_jacobian = p1_from_affine(a);

    // Add second point (in affine) to first point (in Jacobian)
    let sum_jacobian = p1_add_or_double(&a_jacobian, b);

    // Convert result back to affine coordinates
    p1_to_affine(&sum_jacobian)
}

/// Add two G2 points in affine form, returning the result in affine form
#[inline]
pub(super) fn p2_add_affine(a: &blst_p2_affine, b: &blst_p2_affine) -> blst_p2_affine {
    // Convert first point to Jacobian coordinates
    let a_jacobian = p2_from_affine(a);

    // Add second point (in affine) to first point (in Jacobian)
    let sum_jacobian = p2_add_or_double(&a_jacobian, b);

    // Convert result back to affine coordinates
    p2_to_affine(&sum_jacobian)
}

/// Performs a G1 scalar multiplication
///
/// Takes a G1 point in affine form and a scalar, and returns the result
/// of the scalar multiplication in affine form
#[inline]
fn p1_scalar_mul(p: &blst_p1_affine, scalar: &[u8]) -> blst_p1_affine {
    // Convert point to Jacobian coordinates
    let p_jacobian = p1_from_affine(p);

    let mut result = blst_p1::default();

    // SAFETY: all inputs are valid blst types
    unsafe { blst_p1_mult(&mut result, &p_jacobian, scalar.as_ptr(), scalar.len() * 8) };

    // Convert result back to affine coordinates
    p1_to_affine(&result)
}

/// Performs a G2 scalar multiplication
///
/// Takes a G2 point in affine form and a scalar, and returns the result
/// of the scalar multiplication in affine form
#[inline]
fn p2_scalar_mul(p: &blst_p2_affine, scalar: &[u8]) -> blst_p2_affine {
    // Convert point to Jacobian coordinates
    let p_jacobian = p2_from_affine(p);

    let mut result = blst_p2::default();
    // SAFETY: all inputs are valid blst types
    unsafe { blst_p2_mult(&mut result, &p_jacobian, scalar.as_ptr(), scalar.len() * 8) };

    // Convert result back to affine coordinates
    p2_to_affine(&result)
}

/// Performs multi-scalar multiplication (MSM) for G1 points
///
/// Takes a vector of G1 points and corresponding scalars, and returns their weighted sum
///
/// Note: This method assumes that `g1_points` does not contain any points at infinity.
#[inline]
pub(super) fn p1_msm(
    g1_points: Vec<blst_p1_affine>,
    scalars_bytes: Vec<u8>,
    nbits: usize,
) -> blst_p1_affine {
    assert!(
        scalars_bytes.len() % SCALAR_LENGTH == 0,
        "Each scalar should be {SCALAR_LENGTH} bytes"
    );
    assert_eq!(
        g1_points.len(),
        scalars_bytes.len() / SCALAR_LENGTH,
        "number of scalars should equal the number of g1 points"
    );

    // When no inputs are given, we return the point at infinity.
    // This case can only trigger, if the initial MSM pairs
    // all had, either a zero scalar or the point at infinity.
    //
    // The precompile will return an error, if the initial input
    // was empty, in accordance with EIP-2537.
    if g1_points.is_empty() {
        return blst_p1_affine::default();
    }

    // When there is only a single point, we use a simpler scalar multiplication
    // procedure
    if g1_points.len() == 1 {
        return p1_scalar_mul(&g1_points[0], &scalars_bytes);
    }

    // Perform multi-scalar multiplication
    let multiexp = g1_points.mult(&scalars_bytes, nbits);

    // Convert result back to affine coordinates
    p1_to_affine(&multiexp)
}

/// Performs multi-scalar multiplication (MSM) for G2 points
///
/// Takes a vector of G2 points and corresponding scalars, and returns their weighted sum
///
/// Note: This method assumes that `g2_points` does not contain any points at infinity.
#[inline]
pub(super) fn p2_msm(
    g2_points: Vec<blst_p2_affine>,
    scalars_bytes: Vec<u8>,
    nbits: usize,
) -> blst_p2_affine {
    assert!(
        scalars_bytes.len() % SCALAR_LENGTH == 0,
        "Each scalar should be {SCALAR_LENGTH} bytes"
    );

    assert_eq!(
        g2_points.len(),
        scalars_bytes.len() / SCALAR_LENGTH,
        "number of scalars should equal the number of g2 points"
    );

    // When no inputs are given, we return the point at infinity.
    // This case can only trigger, if the initial MSM pairs
    // all had, either a zero scalar or the point at infinity.
    //
    // The precompile will return an error, if the initial input
    // was empty, in accordance with EIP-2537.
    if g2_points.is_empty() {
        return blst_p2_affine::default();
    }

    // When there is only a single point, we use a simpler scalar multiplication
    // procedure
    if g2_points.len() == 1 {
        return p2_scalar_mul(&g2_points[0], &scalars_bytes);
    }

    // Perform multi-scalar multiplication
    let multiexp = g2_points.mult(&scalars_bytes, nbits);

    // Convert result back to affine coordinates
    p2_to_affine(&multiexp)
}

/// Maps a field element to a G1 point
///
/// Takes a field element (blst_fp) and returns the corresponding G1 point in affine form
#[inline]
pub(super) fn map_fp_to_g1(fp: &blst_fp) -> blst_p1_affine {
    // Create a new G1 point in Jacobian coordinates
    let mut p = blst_p1::default();

    // Map the field element to a point on the curve
    // SAFETY: `p` and `fp` are blst values
    // Third argument is unused if null
    unsafe { blst_map_to_g1(&mut p, fp, core::ptr::null()) };

    // Convert to affine coordinates
    p1_to_affine(&p)
}

/// Maps a field element to a G2 point
///
/// Takes a field element (blst_fp2) and returns the corresponding G2 point in affine form
#[inline]
pub(super) fn map_fp2_to_g2(fp2: &blst_fp2) -> blst_p2_affine {
    // Create a new G2 point in Jacobian coordinates
    let mut p = blst_p2::default();

    // Map the field element to a point on the curve
    // SAFETY: `p` and `fp2` are blst values
    // Third argument is unused if null
    unsafe { blst_map_to_g2(&mut p, fp2, core::ptr::null()) };

    // Convert to affine coordinates
    p2_to_affine(&p)
}

/// Computes a single miller loop for a given G1, G2 pair
#[inline]
fn compute_miller_loop(g1: &blst_p1_affine, g2: &blst_p2_affine) -> blst_fp12 {
    let mut result = blst_fp12::default();

    // SAFETY: All arguments are valid blst types
    unsafe { blst_miller_loop(&mut result, g2, g1) }

    result
}

/// multiply_fp12 multiplies two fp12 elements
#[inline]
fn multiply_fp12(a: &blst_fp12, b: &blst_fp12) -> blst_fp12 {
    let mut result = blst_fp12::default();

    // SAFETY: All arguments are valid blst types
    unsafe { blst_fp12_mul(&mut result, a, b) }

    result
}

/// final_exp computes the final exponentiation on an fp12 element
#[inline]
fn final_exp(f: &blst_fp12) -> blst_fp12 {
    let mut result = blst_fp12::default();

    // SAFETY: All arguments are valid blst types
    unsafe { blst_final_exp(&mut result, f) }

    result
}

/// is_fp12_one checks if an fp12 element equals
/// multiplicative identity element, one
#[inline]
fn is_fp12_one(f: &blst_fp12) -> bool {
    // SAFETY: argument is a valid blst type
    unsafe { blst_fp12_is_one(f) }
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
#[inline]
pub(super) fn pairing_check(pairs: &[(blst_p1_affine, blst_p2_affine)]) -> bool {
    // When no inputs are given, we return true
    // This case can only trigger, if the initial pairing components
    // all had, either the G1 element as the point at infinity
    // or the G2 element as the point at infinity.
    //
    // The precompile will return an error, if the initial input
    // was empty, in accordance with EIP-2537.
    if pairs.is_empty() {
        return true;
    }
    // Compute the miller loop for the first pair
    let (first_g1, first_g2) = &pairs[0];
    let mut acc = compute_miller_loop(first_g1, first_g2);

    // For the remaining pairs, compute miller loop and multiply with the accumulated result
    for (g1, g2) in pairs.iter().skip(1) {
        let ml = compute_miller_loop(g1, g2);
        acc = multiply_fp12(&acc, &ml);
    }

    // Apply final exponentiation and check if result is 1
    let final_result = final_exp(&acc);

    // Check if the result is one (identity element)
    is_fp12_one(&final_result)
}
