//! BN254 precompile implementation using herumi/mcl via [`mcl_rust`].

use super::{FQ2_LEN, FQ_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;
use mcl_rust::{CurveType, Fp, Fp2, Fr, G1, G2, GT};
use std::vec::Vec;

/// Ensure the mcl library is initialized for BN254.
#[inline]
fn ensure_init() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // CurveType::SNARK (MCL_BN_SNARK1) is the Ethereum-compatible BN254 (alt_bn128).
        // CurveType::BN254 is a different, older BN254 parameterization.
        assert!(
            mcl_rust::init(CurveType::SNARK),
            "mcl BN254 initialization failed"
        );
    });
}

/// Deserialize a big-endian 32-byte field element into an mcl `Fp`.
///
/// Returns an error if the value is not a valid field element (>= field modulus).
#[inline]
fn read_fp(input: &[u8]) -> Result<Fp, PrecompileError> {
    // Convert big-endian to little-endian for mcl
    let mut le_bytes = [0u8; FQ_LEN];
    le_bytes.copy_from_slice(&input[..FQ_LEN]);
    le_bytes.reverse();

    let mut fp = Fp::zero();
    // set_little_endian fails if value >= field modulus
    if !fp.set_little_endian(&le_bytes) {
        return Err(PrecompileError::Bn254FieldPointNotAMember);
    }
    Ok(fp)
}

/// Encode an `Fp` element to a 32-byte big-endian representation.
#[inline]
fn encode_fp(fp: &Fp) -> [u8; FQ_LEN] {
    let serialized = fp.serialize();
    // serialized is little-endian; convert to big-endian padded to 32 bytes
    let mut result = [0u8; FQ_LEN];
    let len = serialized.len().min(FQ_LEN);
    result[..len].copy_from_slice(&serialized[..len]);
    result[..FQ_LEN].reverse();
    result
}

/// Reads an Fq2 (quadratic extension field element) from the input slice.
///
/// Ethereum encoding: `[imag(32) | real(32)]`
/// MCL Fp2: `d[0]` is real part, `d[1]` is imaginary part.
#[inline]
fn read_fp2(input: &[u8]) -> Result<Fp2, PrecompileError> {
    let imag = read_fp(&input[..FQ_LEN])?;
    let real = read_fp(&input[FQ_LEN..2 * FQ_LEN])?;
    let mut fp2 = Fp2::zero();
    fp2.d[0] = real;
    fp2.d[1] = imag;
    Ok(fp2)
}

/// Reads a G1 point from a 64-byte Ethereum-format input.
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
pub(super) fn read_g1_point(input: &[u8]) -> Result<G1, PrecompileError> {
    ensure_init();
    let px = read_fp(&input[..FQ_LEN])?;
    let py = read_fp(&input[FQ_LEN..G1_LEN])?;

    // Point at infinity
    if px.is_zero() && py.is_zero() {
        return Ok(G1::zero());
    }

    let mut p = G1::zero();
    p.x = px;
    p.y = py;
    p.z = Fp::from_int(1);

    if !p.is_valid() {
        return Err(PrecompileError::Bn254AffineGFailedToCreate);
    }

    Ok(p)
}

/// Encodes a G1 point to a 64-byte Ethereum-format output.
#[inline]
pub(super) fn encode_g1_point(point: G1) -> [u8; G1_LEN] {
    if point.is_zero() {
        return [0u8; G1_LEN];
    }

    // Normalize to affine coordinates (z = 1)
    let mut affine = G1::zero();
    G1::normalize(&mut affine, &point);

    let mut output = [0u8; G1_LEN];
    output[..FQ_LEN].copy_from_slice(&encode_fp(&affine.x));
    output[FQ_LEN..].copy_from_slice(&encode_fp(&affine.y));
    output
}

/// Reads a G2 point from a 128-byte Ethereum-format input.
///
/// # Panics
///
/// Panics if the input is not at least 128 bytes long.
#[inline]
pub(super) fn read_g2_point(input: &[u8]) -> Result<G2, PrecompileError> {
    ensure_init();
    let x = read_fp2(&input[..FQ2_LEN])?;
    let y = read_fp2(&input[FQ2_LEN..2 * FQ2_LEN])?;

    // Point at infinity
    if x.is_zero() && y.is_zero() {
        return Ok(G2::zero());
    }

    let mut p = G2::zero();
    p.x = x;
    p.y = y;
    // Set z = 1 in Fp2 (real = 1, imag = 0)
    p.z.d[0] = Fp::from_int(1);

    if !p.is_valid() {
        return Err(PrecompileError::Bn254AffineGFailedToCreate);
    }

    Ok(p)
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

    // Convert big-endian to little-endian for mcl
    let mut le_bytes = [0u8; SCALAR_LEN];
    le_bytes.copy_from_slice(input);
    le_bytes.reverse();

    let mut fr = Fr::zero();
    // set_little_endian_mod reduces the value modulo the curve order,
    // matching Ethereum's behavior of accepting any 32-byte scalar.
    fr.set_little_endian_mod(&le_bytes);
    fr
}

/// Performs point addition on two G1 points.
#[inline]
pub(crate) fn g1_point_add(p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], PrecompileError> {
    let p1 = read_g1_point(p1_bytes)?;
    let p2 = read_g1_point(p2_bytes)?;
    let result = &p1 + &p2;
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
    let mut result = G1::zero();
    G1::mul(&mut result, &p, &fr);
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

    // Compute product of miller loops, then do a single final exponentiation.
    // This is more efficient than computing individual pairings.
    let mut acc = GT::zero();
    mcl_rust::miller_loop(&mut acc, &g1_points[0], &g2_points[0]);

    for i in 1..g1_points.len() {
        let mut tmp = GT::zero();
        mcl_rust::miller_loop(&mut tmp, &g1_points[i], &g2_points[i]);
        acc *= &tmp;
    }

    let mut result = GT::zero();
    mcl_rust::final_exp(&mut result, &acc);

    Ok(result.is_one())
}
