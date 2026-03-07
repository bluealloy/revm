//! BN254 precompile implementation using Arkworks.

use super::{Bn254Ops, FQ2_LEN, FQ_LEN, G1_LEN, SCALAR_LEN};
use crate::PrecompileError;

use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G1Projective, G2Affine};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::{One, PrimeField, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// Arkworks backend marker type.
pub(crate) struct ArkworksOps;

impl Bn254Ops for ArkworksOps {
    type G1 = G1Affine;
    type G2 = G2Affine;
    type Scalar = Fr;

    #[inline]
    fn read_g1(input: &[u8]) -> Result<Self::G1, PrecompileError> {
        let px = read_fq(&input[0..FQ_LEN])?;
        let py = read_fq(&input[FQ_LEN..2 * FQ_LEN])?;
        new_g1_point(px, py)
    }

    #[inline]
    fn encode_g1(point: Self::G1) -> [u8; G1_LEN] {
        let mut output = [0u8; G1_LEN];
        let Some((x, y)) = point.xy() else {
            return output;
        };

        let mut x_bytes = [0u8; FQ_LEN];
        x.serialize_uncompressed(&mut x_bytes[..])
            .expect("Failed to serialize x coordinate");

        let mut y_bytes = [0u8; FQ_LEN];
        y.serialize_uncompressed(&mut y_bytes[..])
            .expect("Failed to serialize y coordinate");

        // Convert to big endian by reversing the bytes.
        x_bytes.reverse();
        y_bytes.reverse();

        output[0..FQ_LEN].copy_from_slice(&x_bytes);
        output[FQ_LEN..(FQ_LEN * 2)].copy_from_slice(&y_bytes);

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
        Fr::from_be_bytes_mod_order(input)
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
        (G1Projective::from(p1) + p2).into_affine()
    }

    #[inline]
    fn g1_mul(p: Self::G1, s: Self::Scalar) -> Self::G1 {
        p.mul_bigint(s.into_bigint()).into_affine()
    }

    #[inline]
    fn pairing_check(g1: &[Self::G1], g2: &[Self::G2]) -> bool {
        Bn254::multi_pairing(g1, g2).0.is_one()
    }
}

/// Reads a single `Fq` field element from a 32-byte big-endian input.
#[inline]
fn read_fq(input_be: &[u8]) -> Result<Fq, PrecompileError> {
    assert_eq!(input_be.len(), FQ_LEN, "input must be {FQ_LEN} bytes");

    let mut input_le = [0u8; FQ_LEN];
    input_le.copy_from_slice(input_be);

    // Reverse in-place to convert from big-endian to little-endian.
    input_le.reverse();

    Fq::deserialize_uncompressed(&input_le[..])
        .map_err(|_| PrecompileError::Bn254FieldPointNotAMember)
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

/// Creates a new validated `G1Affine` point from affine coordinates.
#[inline]
fn new_g1_point(px: Fq, py: Fq) -> Result<G1Affine, PrecompileError> {
    if px.is_zero() && py.is_zero() {
        Ok(G1Affine::zero())
    } else {
        let point = G1Affine::new_unchecked(px, py);
        if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
            return Err(PrecompileError::Bn254AffineGFailedToCreate);
        }
        Ok(point)
    }
}

/// Creates a new validated `G2Affine` point from affine coordinates.
#[inline]
fn new_g2_point(x: Fq2, y: Fq2) -> Result<G2Affine, PrecompileError> {
    if x.is_zero() && y.is_zero() {
        Ok(G2Affine::zero())
    } else {
        let point = G2Affine::new_unchecked(x, y);
        if !point.is_on_curve() || !point.is_in_correct_subgroup_assuming_on_curve() {
            return Err(PrecompileError::Bn254AffineGFailedToCreate);
        }
        Ok(point)
    }
}
