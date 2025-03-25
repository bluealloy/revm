use super::{FQ2_LEN, FQ_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;
use std::vec::Vec;

use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::{One, PrimeField, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// Reads a single `Fq` field element from the input slice.
///
/// Takes a byte slice and attempts to interpret the first 32 bytes as an
/// elliptic curve field element. Returns an error if the bytes do not form
/// a valid field element.
///
/// # Panics
///
/// Panics if the input is not at least 32 bytes long.
#[inline]
fn read_fq(input_be: &[u8]) -> Result<Fq, PrecompileError> {
    assert_eq!(input_be.len(), FQ_LEN, "input must be {FQ_LEN} bytes");

    let mut input_le = [0u8; FQ_LEN];
    input_le.copy_from_slice(input_be);

    // Reverse in-place to convert from big-endian to little-endian.
    input_le.reverse();

    Fq::deserialize_uncompressed(&input_le[..])
        .map_err(|_| PrecompileError::Bn128FieldPointNotAMember)
}
/// Reads a Fq2 (quadratic extension field element) from the input slice.
///
/// Parses two consecutive Fq field elements as the real and imaginary parts
/// of an Fq2 element.
/// The second component is parsed before the first, ie if a we represent an
/// element in Fq2 as (x,y) -- `y` is parsed before `x`
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
fn read_fq2(input: &[u8]) -> Result<Fq2, PrecompileError> {
    let y = read_fq(&input[..FQ_LEN])?;
    let x = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;

    Ok(Fq2::new(x, y))
}

/// Creates a new `G1` point from the given `x` and `y` coordinates.
///
/// Constructs a point on the G1 curve from its affine coordinates.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically because `AffineG1` is not capable of
/// representing such a point.
/// In particular, when we convert from `AffineG1` to `G1`, the point
/// will be (0,0,1) instead of (0,1,0)
#[inline]
fn new_g1_point(px: Fq, py: Fq) -> Result<G1Affine, PrecompileError> {
    if px.is_zero() && py.is_zero() {
        Ok(G1Affine::zero())
    } else {
        // We cannot use `G1Affine::new` because that triggers an assert if the point is not on the curve.
        let point = G1Affine::new_unchecked(px, py);
        if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
            return Err(PrecompileError::Bn128AffineGFailedToCreate);
        }
        Ok(point)
    }
}

/// Creates a new `G2` point from the given Fq2 coordinates.
///
/// G2 points in BN128 are defined over a quadratic extension field Fq2.
/// This function takes two Fq2 elements representing the x and y coordinates
/// and creates a G2 point.
///
/// Note: The point at infinity which is represented as (0,0) is
/// handled specifically because `AffineG2` is not capable of
/// representing such a point.
/// In particular, when we convert from `AffineG2` to `G2`, the point
/// will be (0,0,1) instead of (0,1,0)
#[inline]
fn new_g2_point(x: Fq2, y: Fq2) -> Result<G2Affine, PrecompileError> {
    let point = if x.is_zero() && y.is_zero() {
        G2Affine::zero()
    } else {
        // We cannot use `G1Affine::new` because that triggers an assert if the point is not on the curve.
        let point = G2Affine::new_unchecked(x, y);
        if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
            return Err(PrecompileError::Bn128AffineGFailedToCreate);
        }
        point
    };

    Ok(point)
}

/// Reads a G1 point from the input slice.
///
/// Parses a G1 point from a byte slice by reading two consecutive field elements
/// representing the x and y coordinates.
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
pub(super) fn read_g1_point(input: &[u8]) -> Result<G1Affine, PrecompileError> {
    let px = read_fq(&input[0..FQ_LEN])?;
    let py = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
    new_g1_point(px, py)
}

/// Encodes a G1 point into a byte array.
///
/// Converts a G1 point in Jacobian coordinates to affine coordinates and
/// serializes the x and y coordinates as big-endian byte arrays.
///
/// Note: If the point is the point at infinity, this function returns
/// all zeroes.
#[inline]
pub(super) fn encode_g1_point(point: G1Affine) -> [u8; G1_LEN] {
    let mut output = [0u8; G1_LEN];
    let Some((x, y)) = point.xy() else {
        return output;
    };

    let mut x_bytes = [0u8; FQ_LEN];
    x.serialize_uncompressed(&mut x_bytes[..])
        .expect("Failed to serialize x coordinate");

    let mut y_bytes = [0u8; FQ_LEN];
    y.serialize_uncompressed(&mut y_bytes[..])
        .expect("Failed to serialize x coordinate");

    // Convert to big endian by reversing the bytes.
    x_bytes.reverse();
    y_bytes.reverse();

    // Place x in the first half, y in the second half.
    output[0..FQ_LEN].copy_from_slice(&x_bytes);
    output[FQ_LEN..(FQ_LEN * 2)].copy_from_slice(&y_bytes);

    output
}

/// Reads a G2 point from the input slice.
///
/// Parses a G2 point from a byte slice by reading four consecutive Fq field elements
/// representing the two Fq2 coordinates (x and y) of the G2 point.
///
/// # Panics
///
/// Panics if the input is not at least 128 bytes long.
#[inline]
pub(super) fn read_g2_point(input: &[u8]) -> Result<G2Affine, PrecompileError> {
    let ba = read_fq2(&input[0..FQ2_LEN])?;
    let bb = read_fq2(&input[FQ2_LEN..2 * FQ2_LEN])?;
    new_g2_point(ba, bb)
}

/// Reads a scalar from the input slice
///
/// Note: The scalar does not need to be canonical.
///
/// # Panics
///
/// If `input.len()` is not equal to [`SCALAR_LEN`].
#[inline]
pub(super) fn read_scalar(input: &[u8]) -> Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    Fr::from_be_bytes_mod_order(input)
}

/// Performs point addition on two G1 points.
#[inline]
pub(super) fn g1_point_add(p1: G1Affine, p2: G1Affine) -> G1Affine {
    let p1_jacobian: G1Projective = p1.into();

    let p3 = p1_jacobian + p2;

    p3.into_affine()
}

/// Performs a G1 scalar multiplication.
#[inline]
pub(super) fn g1_point_mul(p: G1Affine, fr: Fr) -> G1Affine {
    let big_int = fr.into_bigint();
    let result = p.mul_bigint(big_int);

    result.into_affine()
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
///
/// Note: If the input is empty, this function returns true.
/// This is different to EIP2537 which disallows the empty input.
#[inline]
pub(super) fn pairing_check(pairs: &[(G1Affine, G2Affine)]) -> bool {
    if pairs.is_empty() {
        return true;
    }

    let (g1_points, g2_points): (Vec<G1Affine>, Vec<G2Affine>) = pairs.iter().copied().unzip();

    let pairing_result = Bn254::multi_pairing(&g1_points, &g2_points);
    pairing_result.0.is_one()
}
