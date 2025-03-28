use crate::{
    bls12_381_const::{
        FP_LENGTH, FP_PAD_BY, PADDED_FP_LENGTH, PADDED_G1_LENGTH, PADDED_G2_LENGTH, SCALAR_LENGTH,
    },
    PrecompileError,
};
use ark_bls12_381::{Bls12_381, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{
    hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurve},
    pairing::Pairing,
    AffineRepr, CurveGroup, VariableBaseMSM,
};
use ark_ff::{One, PrimeField, Zero};

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::{string::ToString, vec::Vec};

/// Reads a single `Fp` field element from the input slice.
///
/// Takes a byte slice in Big Endian format and attempts to interpret it as an
/// elliptic curve field element. Returns an error if the bytes do not form
/// a valid field element.
///
/// # Panics
///
/// Panics if the input is not exactly 48 bytes long.
#[inline]
pub(super) fn read_fp(input_be: &[u8]) -> Result<Fq, PrecompileError> {
    assert_eq!(input_be.len(), FP_LENGTH, "input must be {FP_LENGTH} bytes");

    let mut input_le = [0u8; FP_LENGTH];
    input_le.copy_from_slice(input_be);

    // Reverse in-place to convert from big-endian to little-endian.
    input_le.reverse();

    Fq::deserialize_uncompressed(&input_le[..])
        .map_err(|_| PrecompileError::Other("non-canonical fp value".to_string()))
}

/// Encodes an `Fp` field element into a padded, big-endian byte array.
///
/// # Panics
///
/// Panics if serialization fails, which should not occur for a valid field element.
pub(super) fn encode_fp(fp: &Fq) -> [u8; PADDED_FP_LENGTH] {
    let mut bytes = [0u8; FP_LENGTH];
    fp.serialize_uncompressed(&mut bytes[..])
        .expect("Failed to serialize field element");
    bytes.reverse();

    let mut padded_bytes = [0; PADDED_FP_LENGTH];
    padded_bytes[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&bytes);

    padded_bytes
}

/// Reads a Fp2 (quadratic extension field element) from the input slices.
///
/// Parses two Fp field elements in Big Endian format for the Fp2 element.
///
/// # Panics
///
/// Panics if either input is not exactly 48 bytes long.
#[inline]
pub(super) fn read_fp2(
    input_1: &[u8; FP_LENGTH],
    input_2: &[u8; FP_LENGTH],
) -> Result<Fq2, PrecompileError> {
    let fp_1 = read_fp(input_1)?;
    let fp_2 = read_fp(input_2)?;

    Ok(Fq2::new(fp_1, fp_2))
}

/// Creates a new `G1` point from the given `x` and `y` coordinates.
///
/// Constructs a point on the G1 curve from its affine coordinates.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically.
#[inline]
fn new_g1_point_no_subgroup_check(px: Fq, py: Fq) -> Result<G1Affine, PrecompileError> {
    if px.is_zero() && py.is_zero() {
        Ok(G1Affine::zero())
    } else {
        // We cannot use `G1Affine::new` because that triggers an assert if the point is not on the curve.
        let point = G1Affine::new_unchecked(px, py);
        if !point.is_on_curve() {
            return Err(PrecompileError::Other(
                "Element not on G1 curve".to_string(),
            ));
        }
        Ok(point)
    }
}

/// Creates a new `G2` point from the given Fq2 coordinates.
///
/// G2 points in BLS12_381 are defined over a quadratic extension field Fq2.
/// This function takes two Fq2 elements representing the x and y coordinates
/// and creates a G2 point.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically.
#[inline]
fn new_g2_point_no_subgroup_check(x: Fq2, y: Fq2) -> Result<G2Affine, PrecompileError> {
    let point = if x.is_zero() && y.is_zero() {
        G2Affine::zero()
    } else {
        // We cannot use `G2Affine::new` because that triggers an assert if the point is not on the curve.
        let point = G2Affine::new_unchecked(x, y);
        if !point.is_on_curve() {
            return Err(PrecompileError::Other(
                "Element not on G2 curve".to_string(),
            ));
        }
        point
    };

    Ok(point)
}

/// Reads a G1 point from the input slices.
///
/// Parses a G1 point from byte slices by reading two field elements
/// representing the x and y coordinates in Big Endian format.
/// Also performs a subgroup check to ensure the point is in the correct subgroup.
///
/// # Panics
///
/// Panics if the inputs are not exactly 48 bytes long.
#[inline]
pub(super) fn read_g1(
    x: &[u8; FP_LENGTH],
    y: &[u8; FP_LENGTH],
) -> Result<G1Affine, PrecompileError> {
    let point = read_g1_no_subgroup_check(x, y)?;
    if !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err(PrecompileError::Other(
            "Element not in the correct subgroup".to_string(),
        ));
    }
    Ok(point)
}

/// Reads a G1 point without performing a subgroup check.
///
/// Note: Skipping subgroup checks can introduce security issues.
/// This method should only be called if:
///     - The EIP specifies that no subgroup check should be performed
///     - One can be certain that the point is in the correct subgroup.
#[inline]
pub(super) fn read_g1_no_subgroup_check(
    x: &[u8; FP_LENGTH],
    y: &[u8; FP_LENGTH],
) -> Result<G1Affine, PrecompileError> {
    let px = read_fp(x)?;
    let py = read_fp(y)?;
    new_g1_point_no_subgroup_check(px, py)
}

/// Encodes a G1 point into a byte array with padded elements.
///
/// Converts a G1 point to affine coordinates and serializes the x and y coordinates
/// as big-endian byte arrays with padding to match the expected format.
#[inline]
pub(super) fn encode_g1_point(input: &G1Affine) -> [u8; PADDED_G1_LENGTH] {
    let mut output = [0u8; PADDED_G1_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    let x_encoded = encode_fp(&x);
    let y_encoded = encode_fp(&y);

    // Copy the encoded values to the output
    output[..PADDED_FP_LENGTH].copy_from_slice(&x_encoded);
    output[PADDED_FP_LENGTH..].copy_from_slice(&y_encoded);

    output
}

/// Reads a G2 point from the input slices.
///
/// Parses a G2 point from byte slices by reading four field elements
/// representing the x and y coordinates in Big Endian format.
/// Also performs a subgroup check to ensure the point is in the correct subgroup.
#[inline]
pub(super) fn read_g2(
    a_x_0: &[u8; FP_LENGTH],
    a_x_1: &[u8; FP_LENGTH],
    a_y_0: &[u8; FP_LENGTH],
    a_y_1: &[u8; FP_LENGTH],
) -> Result<G2Affine, PrecompileError> {
    let point = read_g2_no_subgroup_check(a_x_0, a_x_1, a_y_0, a_y_1)?;
    if !point.is_in_correct_subgroup_assuming_on_curve() {
        return Err(PrecompileError::Other(
            "Element not in the correct subgroup".to_string(),
        ));
    }
    Ok(point)
}

/// Reads a G2 point without performing a subgroup check.
///
/// Note: Skipping subgroup checks can introduce security issues.
/// This method should only be called if:
///     - The EIP specifies that no subgroup check should be performed
///     - One can be certain that the point is in the correct subgroup.
#[inline]
pub(super) fn read_g2_no_subgroup_check(
    a_x_0: &[u8; FP_LENGTH],
    a_x_1: &[u8; FP_LENGTH],
    a_y_0: &[u8; FP_LENGTH],
    a_y_1: &[u8; FP_LENGTH],
) -> Result<G2Affine, PrecompileError> {
    let x = read_fp2(a_x_0, a_x_1)?;
    let y = read_fp2(a_y_0, a_y_1)?;
    new_g2_point_no_subgroup_check(x, y)
}

/// Encodes a G2 point into a byte array with padded elements.
///
/// Converts a G2 point to affine coordinates and serializes the coordinates
/// as big-endian byte arrays with padding to match the expected format.
#[inline]
pub(super) fn encode_g2_point(input: &G2Affine) -> [u8; PADDED_G2_LENGTH] {
    let mut output = [0u8; PADDED_G2_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    let x_c0_encoded = encode_fp(&x.c0);
    let x_c1_encoded = encode_fp(&x.c1);
    let y_c0_encoded = encode_fp(&y.c0);
    let y_c1_encoded = encode_fp(&y.c1);

    output[..PADDED_FP_LENGTH].copy_from_slice(&x_c0_encoded);
    output[PADDED_FP_LENGTH..2 * PADDED_FP_LENGTH].copy_from_slice(&x_c1_encoded);
    output[2 * PADDED_FP_LENGTH..3 * PADDED_FP_LENGTH].copy_from_slice(&y_c0_encoded);
    output[3 * PADDED_FP_LENGTH..4 * PADDED_FP_LENGTH].copy_from_slice(&y_c1_encoded);

    output
}

/// Extracts a scalar from a byte slice representation, decoding the input as a Big Endian
/// unsigned integer.
///
/// Note: We do not check that the scalar is a canonical Fr element, because the EIP specifies:
/// * The corresponding integer is not required to be less than or equal than main subgroup order.
#[inline]
pub(super) fn read_scalar(input: &[u8]) -> Result<Fr, PrecompileError> {
    if input.len() != SCALAR_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {SCALAR_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    Ok(Fr::from_be_bytes_mod_order(input))
}

/// Performs point addition on two G1 points.
#[inline]
pub(super) fn p1_add_affine(p1: &G1Affine, p2: &G1Affine) -> G1Affine {
    let p1_proj: G1Projective = (*p1).into();
    let p3 = p1_proj + p2;
    p3.into_affine()
}

/// Performs point addition on two G2 points.
#[inline]
pub(super) fn p2_add_affine(p1: &G2Affine, p2: &G2Affine) -> G2Affine {
    let p1_proj: G2Projective = (*p1).into();
    let p3 = p1_proj + p2;
    p3.into_affine()
}

/// Performs multi-scalar multiplication (MSM) for G1 points
///
/// Takes a vector of G1 points and corresponding scalars, and returns their weighted sum
///
/// Note: This method assumes that `g1_points` does not contain any points at infinity.
#[inline]
pub(super) fn p1_msm(g1_points: Vec<G1Affine>, scalars: Vec<Fr>) -> G1Affine {
    assert_eq!(
        g1_points.len(),
        scalars.len(),
        "number of scalars should equal the number of g1 points"
    );

    if g1_points.is_empty() {
        return G1Affine::zero();
    }

    if g1_points.len() == 1 {
        let big_int = scalars[0].into_bigint();
        return g1_points[0].mul_bigint(big_int).into_affine();
    }

    // Perform multi-scalar multiplication
    G1Projective::msm(&g1_points, &scalars)
        .expect("MSM should succeed")
        .into_affine()
}

/// Performs multi-scalar multiplication (MSM) for G2 points
///
/// Takes a vector of G2 points and corresponding scalars, and returns their weighted sum
///
/// Note: This method assumes that `g2_points` does not contain any points at infinity.
#[inline]
pub(super) fn p2_msm(g2_points: Vec<G2Affine>, scalars: Vec<Fr>) -> G2Affine {
    assert_eq!(
        g2_points.len(),
        scalars.len(),
        "number of scalars should equal the number of g2 points"
    );

    if g2_points.is_empty() {
        return G2Affine::zero();
    }

    if g2_points.len() == 1 {
        let big_int = scalars[0].into_bigint();
        return g2_points[0].mul_bigint(big_int).into_affine();
    }

    // Perform multi-scalar multiplication
    G2Projective::msm(&g2_points, &scalars)
        .expect("MSM should succeed")
        .into_affine()
}

/// Maps a field element to a G1 point
///
/// Takes a field element (Fq) and returns the corresponding G1 point in affine form
#[inline]
pub(super) fn map_fp_to_g1(fp: &Fq) -> G1Affine {
    WBMap::map_to_curve(*fp)
        .expect("map_to_curve is infallible")
        .clear_cofactor()
}

/// Maps a field element to a G2 point
///
/// Takes a field element (Fq2) and returns the corresponding G2 point in affine form
#[inline]
pub(super) fn map_fp2_to_g2(fp2: &Fq2) -> G2Affine {
    WBMap::map_to_curve(*fp2)
        .expect("map_to_curve is infallible")
        .clear_cofactor()
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
#[inline]
pub(super) fn pairing_check(pairs: &[(G1Affine, G2Affine)]) -> bool {
    if pairs.is_empty() {
        return true;
    }

    let (g1_points, g2_points): (Vec<G1Affine>, Vec<G2Affine>) = pairs.iter().copied().unzip();

    let pairing_result = Bls12_381::multi_pairing(&g1_points, &g2_points);
    pairing_result.0.is_one()
}
