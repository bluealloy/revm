use super::{FQ2_LEN, FQ_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;
use bn::{AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

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
fn read_fq(input: &[u8]) -> Result<Fq, PrecompileError> {
    Fq::from_slice(&input[..FQ_LEN]).map_err(|_| PrecompileError::Bn128FieldPointNotAMember)
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
fn new_g1_point(px: Fq, py: Fq) -> Result<G1, PrecompileError> {
    if px == Fq::zero() && py == Fq::zero() {
        Ok(G1::zero())
    } else {
        AffineG1::new(px, py)
            .map(Into::into)
            .map_err(|_| PrecompileError::Bn128AffineGFailedToCreate)
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
fn new_g2_point(x: Fq2, y: Fq2) -> Result<G2, PrecompileError> {
    let point = if x.is_zero() && y.is_zero() {
        G2::zero()
    } else {
        G2::from(AffineG2::new(x, y).map_err(|_| PrecompileError::Bn128AffineGFailedToCreate)?)
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
pub(super) fn read_g1_point(input: &[u8]) -> Result<G1, PrecompileError> {
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
pub(super) fn encode_g1_point(point: G1) -> [u8; G1_LEN] {
    let mut output = [0u8; G1_LEN];

    if let Some(point_affine) = AffineG1::from_jacobian(point) {
        point_affine
            .x()
            .to_big_endian(&mut output[..FQ_LEN])
            .unwrap();
        point_affine
            .y()
            .to_big_endian(&mut output[FQ_LEN..])
            .unwrap();
    }

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
pub(super) fn read_g2_point(input: &[u8]) -> Result<G2, PrecompileError> {
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
pub(super) fn read_scalar(input: &[u8]) -> bn::Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    // `Fr::from_slice` can only fail when the length is not `SCALAR_LEN`.
    bn::Fr::from_slice(input).unwrap()
}

/// Performs point addition on two G1 points.
#[inline]
pub(super) fn g1_point_add(p1: G1, p2: G1) -> G1 {
    p1 + p2
}

/// Performs a G1 scalar multiplication.
#[inline]
pub(super) fn g1_point_mul(p: G1, fr: bn::Fr) -> G1 {
    p * fr
}

/// pairing_check performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
///
/// Note: If the input is empty, this function returns true.
/// This is different to EIP2537 which disallows the empty input.
#[inline]
pub(super) fn pairing_check(pairs: &[(G1, G2)]) -> bool {
    if pairs.is_empty() {
        return true;
    }
    bn::pairing_batch(pairs) == Gt::one()
}
