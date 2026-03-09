//! BN254 precompile implementation using substrate-bn.

use super::{Bn254Ops, FQ2_LEN, FQ_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;
use bn::{AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

/// Substrate-bn backend marker type.
pub(crate) struct SubstrateOps;

impl Bn254Ops for SubstrateOps {
    type G1 = G1;
    type G2 = G2;
    type Scalar = bn::Fr;

    #[inline]
    fn read_g1(input: &[u8]) -> Result<Self::G1, PrecompileError> {
        let px = read_fq(&input[0..FQ_LEN])?;
        let py = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
        new_g1_point(px, py)
    }

    #[inline]
    fn encode_g1(point: Self::G1) -> [u8; G1_LEN] {
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

    #[inline]
    fn read_g2(input: &[u8]) -> Result<Self::G2, PrecompileError> {
        let ba = read_fq2(&input[0..FQ2_LEN])?;
        let bb = read_fq2(&input[FQ2_LEN..2 * FQ2_LEN])?;
        new_g2_point(ba, bb)
    }

    #[inline]
    fn read_scalar(input: &[u8]) -> Self::Scalar {
        assert_eq!(
            input.len(),
            SCALAR_LEN,
            "unexpected scalar length. got {}, expected {SCALAR_LEN}",
            input.len()
        );
        // `Fr::from_slice` can only fail when the length is not `SCALAR_LEN`.
        bn::Fr::from_slice(input).unwrap()
    }

    #[inline]
    fn g1_is_zero(p: &Self::G1) -> bool {
        p.is_zero()
    }

    #[inline]
    fn g2_is_zero(p: &Self::G2) -> bool {
        p.is_zero()
    }

    #[inline]
    fn g1_add(p1: Self::G1, p2: Self::G1) -> Self::G1 {
        p1 + p2
    }

    #[inline]
    fn g1_mul(p: Self::G1, s: Self::Scalar) -> Self::G1 {
        p * s
    }

    #[inline]
    fn pairing_check(g1: &[Self::G1], g2: &[Self::G2]) -> bool {
        let pairs: Vec<(G1, G2)> = g1.iter().copied().zip(g2.iter().copied()).collect();
        bn::pairing_batch(&pairs) == Gt::one()
    }
}

/// Reads a single `Fq` field element from a 32-byte big-endian input.
#[inline]
fn read_fq(input: &[u8]) -> Result<Fq, PrecompileError> {
    Fq::from_slice(&input[..FQ_LEN]).map_err(|_| PrecompileError::Bn254FieldPointNotAMember)
}

/// Reads an Fq2 element from the input slice.
///
/// Ethereum encoding: `[imag(32) | real(32)]`
#[inline]
fn read_fq2(input: &[u8]) -> Result<Fq2, PrecompileError> {
    let y = read_fq(&input[..FQ_LEN])?;
    let x = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
    Ok(Fq2::new(x, y))
}

/// Creates a new validated `G1` point from affine coordinates.
///
/// The point at infinity (0,0) is handled specifically because `AffineG1`
/// cannot represent it.
#[inline]
fn new_g1_point(px: Fq, py: Fq) -> Result<G1, PrecompileError> {
    if px == Fq::zero() && py == Fq::zero() {
        Ok(G1::zero())
    } else {
        AffineG1::new(px, py)
            .map(Into::into)
            .map_err(|_| PrecompileError::Bn254AffineGFailedToCreate)
    }
}

/// Creates a new validated `G2` point from affine coordinates.
///
/// The point at infinity (0,0) is handled specifically because `AffineG2`
/// cannot represent it.
#[inline]
fn new_g2_point(x: Fq2, y: Fq2) -> Result<G2, PrecompileError> {
    if x.is_zero() && y.is_zero() {
        Ok(G2::zero())
    } else {
        Ok(G2::from(
            AffineG2::new(x, y).map_err(|_| PrecompileError::Bn254AffineGFailedToCreate)?,
        ))
    }
}
