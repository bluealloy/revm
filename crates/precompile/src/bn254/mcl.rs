//! BN254 precompile using herumi/mcl implementation.

use super::{FQ2_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;
use std::vec::Vec;

use mcl_bn254::bn254::{self, Fr, G1, G2};

/// Ensure the mcl library is initialized. Uses OnceLock internally,
/// so this is cheap after the first call.
#[inline]
fn ensure_init() {
    assert!(bn254::init(), "mcl BN254 initialization failed");
}

/// Reads a G1 point from a 64-byte Ethereum-format input.
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
pub(super) fn read_g1_point(input: &[u8]) -> Result<G1, PrecompileError> {
    ensure_init();
    G1::from_eth(input[..G1_LEN].try_into().unwrap())
        .ok_or(PrecompileError::Bn254AffineGFailedToCreate)
}

/// Encodes a G1 point to a 64-byte Ethereum-format output.
#[inline]
pub(super) fn encode_g1_point(point: G1) -> [u8; G1_LEN] {
    point.to_eth()
}

/// Reads a G2 point from a 128-byte Ethereum-format input.
///
/// # Panics
///
/// Panics if the input is not at least 128 bytes long.
#[inline]
pub(super) fn read_g2_point(input: &[u8]) -> Result<G2, PrecompileError> {
    ensure_init();
    G2::from_eth(input[..2 * FQ2_LEN].try_into().unwrap())
        .ok_or(PrecompileError::Bn254AffineGFailedToCreate)
}

/// Reads a scalar from a 32-byte big-endian input.
///
/// The scalar is reduced modulo the curve order.
///
/// # Panics
///
/// Panics if the input is not exactly 32 bytes.
#[inline]
pub(super) fn read_scalar(input: &[u8]) -> Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    ensure_init();
    Fr::from_be_bytes(input.try_into().unwrap())
}

/// Performs point addition on two G1 points.
#[inline]
pub(crate) fn g1_point_add(p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], PrecompileError> {
    let p1 = read_g1_point(p1_bytes)?;
    let p2 = read_g1_point(p2_bytes)?;
    let result = p1.add(&p2);
    Ok(encode_g1_point(result))
}

/// Performs a G1 scalar multiplication.
#[inline]
pub(crate) fn g1_point_mul(
    point_bytes: &[u8],
    fr_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    let p = read_g1_point(point_bytes)?;
    let fr = read_scalar(fr_bytes);
    let result = p.mul(&fr);
    Ok(encode_g1_point(result))
}

/// Performs a pairing check on a list of G1 and G2 point pairs and
/// returns true if the result is equal to the identity element.
///
/// Note: If the input is empty, this function returns true.
#[inline]
pub(crate) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    ensure_init();

    let mut g1_points = Vec::with_capacity(pairs.len());
    let mut g2_points = Vec::with_capacity(pairs.len());

    for (g1_bytes, g2_bytes) in pairs {
        let g1 = read_g1_point(g1_bytes)?;
        let g2 = read_g2_point(g2_bytes)?;

        // Skip pairs where either point is at infinity
        if !g1.is_zero() && !g2.is_zero() {
            g1_points.push(g1);
            g2_points.push(g2);
        }
    }

    if g1_points.is_empty() {
        return Ok(true);
    }

    Ok(bn254::pairing_check(&g1_points, &g2_points))
}
