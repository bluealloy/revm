use super::{PrecompileError, FQ_LEN, G1_LEN, G2_LEN, SCALAR_LEN};
use eth_pairings::{
    engines::bn254::*,
    extension_towers::fp12_as_2_over3_over_2::Fp12,
    field::U256Repr,
    integers::MaxGroupSizeUint,
    pairings::PairingEngine,
    public_interface::{decode_g1, decode_g2},
    traits::{Group, ZeroAndOne},
};

/// G1Point is the concrete representation of a G1 element
pub(super) type G1Point = eth_pairings::weierstrass::curve::CurvePoint<
    'static,
    eth_pairings::weierstrass::CurveOverFpParameters<
        'static,
        U256Repr,
        eth_pairings::field::PrimeField<U256Repr>,
    >,
>;

/// G2Point is the concrete representation of a G2 element
pub(super) type G2Point = eth_pairings::weierstrass::curve::CurvePoint<
    'static,
    eth_pairings::weierstrass::CurveOverFp2Parameters<
        'static,
        U256Repr,
        eth_pairings::field::PrimeField<U256Repr>,
    >,
>;
/// Fr is the concrete representation of an element in the scalar field.
pub(super) type Fr = MaxGroupSizeUint;

/// Reads a G1 point from the input slice.
///
/// Parses a G1 point from a byte slice by reading two consecutive field elements
/// representing the x and y coordinates.
#[inline]
pub(super) fn read_g1_point(input: &[u8]) -> Result<G1Point, PrecompileError> {
    let (point, _) = decode_g1::decode_g1_point_from_xy_oversized(input, FQ_LEN, &*BN254_G1_CURVE)
        .map_err(|_| PrecompileError::Bn128AffineGFailedToCreate)?;

    if !point.is_on_curve() {
        return Err(PrecompileError::Bn128AffineGFailedToCreate);
    }

    // We can skip the subgroup check since G1 is prime ordered.

    Ok(point)
}

/// Encodes a G1 point into a byte array.
///
/// Serializes a G1 point into its x and y coordinates as a byte array.
#[inline]
pub(super) fn encode_g1_point(point: G1Point) -> [u8; G1_LEN] {
    let mut output = [0u8; G1_LEN];

    if !point.is_zero() {
        let as_vec = decode_g1::serialize_g1_point(FQ_LEN, &point).unwrap();
        output.copy_from_slice(&as_vec[..]);
    }

    output
}

/// Reads a G2 point from the input slice.
///
/// Parses a G2 point from a byte slice by reading four consecutive field elements
/// representing the two coordinates (x and y) of the G2 point.
#[inline]
pub(super) fn read_g2_point(input: &[u8]) -> Result<G2Point, PrecompileError> {
    // G2 encoding in EIP 196/197 is non-standard: Fp2 element c0 + v*c1 where v is non-residue is
    // encoded as (c1, c0) instead of usual (c0, c1)
    let mut swapped_encoding = [0u8; G2_LEN];

    let x_0 = &input[0..FQ_LEN];
    let x_1 = &input[FQ_LEN..(FQ_LEN * 2)];
    let y_0 = &input[(FQ_LEN * 2)..(FQ_LEN * 3)];
    let y_1 = &input[(FQ_LEN * 3)..(FQ_LEN * 4)];

    // swap for x coordinate
    swapped_encoding[0..FQ_LEN].copy_from_slice(x_1);
    swapped_encoding[FQ_LEN..(FQ_LEN * 2)].copy_from_slice(x_0);

    // swap for y coordinate
    swapped_encoding[(FQ_LEN * 2)..(FQ_LEN * 3)].copy_from_slice(y_1);
    swapped_encoding[(FQ_LEN * 3)..(FQ_LEN * 4)].copy_from_slice(y_0);

    let (g2_point, _) = decode_g2::decode_g2_point_from_xy_in_fp2_oversized(
        &swapped_encoding,
        FQ_LEN,
        &*BN254_G2_CURVE,
    )
    .map_err(|_| PrecompileError::Bn128AffineGFailedToCreate)?;

    if !g2_point.is_on_curve() {
        return Err(PrecompileError::Bn128FieldPointNotAMember);
    }

    // The zero point is on the curve and in the subgroup
    if g2_point.is_zero() {
        return Ok(g2_point);
    }
    // Check G2 point is in the correct subgroup
    let is_in_subgroup = g2_point
        .wnaf_mul_with_window_size(&BN254_SUBGROUP_ORDER[..], 5)
        .is_zero();
    if !is_in_subgroup {
        return Err(PrecompileError::Bn128FieldPointNotAMember);
    }

    Ok(g2_point)
}

/// Reads a scalar from the input slice
///
/// Note: The scalar does not need to be canonical.
#[inline]
pub(super) fn read_scalar(input: &[u8]) -> Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    let (scalar, _) = decode_g1::decode_scalar_representation(input, SCALAR_LEN).unwrap();

    scalar
}

/// Performs point addition on two G1 points.
#[inline]
pub(super) fn g1_point_add(p1: G1Point, p2: G1Point) -> G1Point {
    let mut result = p1.clone();
    result.add_assign(&p2);
    result
}

/// Performs point multiplication.
///
/// Takes a G1 point and a scalar representation, and returns the result of the multiplication.
#[inline]
pub(super) fn g1_point_mul(p: G1Point, scalar: Fr) -> G1Point {
    p.mul(scalar)
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs.
///
/// Returns true if the result of the pairing is equal to the identity element.
#[inline]
pub(super) fn pairing_check(pairs: &[(G1Point, G2Point)]) -> bool {
    if pairs.is_empty() {
        return true;
    }

    let engine = &*BN254_PAIRING_ENGINE;

    // Convert to vectors as required by Matter Labs implementation
    let g1_points: Vec<_> = pairs.iter().map(|(g1, _)| g1.clone()).collect();
    let g2_points: Vec<_> = pairs.iter().map(|(_, g2)| g2.clone()).collect();

    let pairing_result = engine.pair(&g1_points, &g2_points);

    // This returns None under two conditions:
    //
    // - g1_points.len() != g2_points.len()
    // - The final_exponentiation value is 0
    //
    // - The first case is not possible by construction
    // - In the second case, we want to return false because the
    //   result is not 1
    let pairing_result = match pairing_result {
        Some(pr) => pr,
        None => return false,
    };

    let one_fp12 = Fp12::one(&*BN254_EXT12_FIELD);
    pairing_result == one_fp12
}
