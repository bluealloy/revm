use super::{FQ_LEN, G1_LEN, G2_LEN, SCALAR_LEN};
use crate::PrecompileHalt;
use eth_pairings::{
    engines::bn254::*,
    extension_towers::fp12_as_2_over3_over_2::Fp12,
    field::U256Repr,
    integers::MaxGroupSizeUint,
    pairings::PairingEngine,
    public_interface::{decode_g1, decode_g2},
    traits::{Group, ZeroAndOne},
};
use std::vec::Vec;

type G1Point = eth_pairings::weierstrass::curve::CurvePoint<
    'static,
    eth_pairings::weierstrass::CurveOverFpParameters<
        'static,
        U256Repr,
        eth_pairings::field::PrimeField<U256Repr>,
    >,
>;

type G2Point = eth_pairings::weierstrass::curve::CurvePoint<
    'static,
    eth_pairings::weierstrass::CurveOverFp2Parameters<
        'static,
        U256Repr,
        eth_pairings::field::PrimeField<U256Repr>,
    >,
>;

type Fr = MaxGroupSizeUint;

#[inline]
fn read_g1_point(input: &[u8]) -> Result<G1Point, PrecompileHalt> {
    let (point, _) = decode_g1::decode_g1_point_from_xy_oversized(input, FQ_LEN, &BN254_G1_CURVE)
        .map_err(|_| PrecompileHalt::Bn254AffineGFailedToCreate)?;

    if !point.is_on_curve() {
        return Err(PrecompileHalt::Bn254AffineGFailedToCreate);
    }

    Ok(point)
}

#[inline]
fn encode_g1_point(point: G1Point) -> [u8; G1_LEN] {
    let mut output = [0u8; G1_LEN];

    if !point.is_zero() {
        let encoded = decode_g1::serialize_g1_point(FQ_LEN, &point)
            .expect("failed to serialize BN254 G1 point");
        output.copy_from_slice(&encoded);
    }

    output
}

#[inline]
fn read_g2_point(input: &[u8]) -> Result<G2Point, PrecompileHalt> {
    let mut swapped_encoding = [0u8; G2_LEN];

    let x_0 = &input[0..FQ_LEN];
    let x_1 = &input[FQ_LEN..(FQ_LEN * 2)];
    let y_0 = &input[(FQ_LEN * 2)..(FQ_LEN * 3)];
    let y_1 = &input[(FQ_LEN * 3)..(FQ_LEN * 4)];

    swapped_encoding[0..FQ_LEN].copy_from_slice(x_1);
    swapped_encoding[FQ_LEN..(FQ_LEN * 2)].copy_from_slice(x_0);
    swapped_encoding[(FQ_LEN * 2)..(FQ_LEN * 3)].copy_from_slice(y_1);
    swapped_encoding[(FQ_LEN * 3)..(FQ_LEN * 4)].copy_from_slice(y_0);

    let (point, _) = decode_g2::decode_g2_point_from_xy_in_fp2_oversized(
        &swapped_encoding,
        FQ_LEN,
        &BN254_G2_CURVE,
    )
    .map_err(|_| PrecompileHalt::Bn254AffineGFailedToCreate)?;

    if !point.is_on_curve() {
        return Err(PrecompileHalt::Bn254FieldPointNotAMember);
    }

    if point.is_zero() {
        return Ok(point);
    }

    if !point
        .wnaf_mul_with_window_size(&BN254_SUBGROUP_ORDER[..], 5)
        .is_zero()
    {
        return Err(PrecompileHalt::Bn254FieldPointNotAMember);
    }

    Ok(point)
}

#[inline]
fn read_scalar(input: &[u8]) -> Fr {
    assert_eq!(
        input.len(),
        SCALAR_LEN,
        "unexpected scalar length. got {}, expected {SCALAR_LEN}",
        input.len()
    );
    let (scalar, _) = decode_g1::decode_scalar_representation(input, SCALAR_LEN).unwrap();
    scalar
}

#[inline]
pub(crate) fn g1_point_add(
    p1_bytes: &[u8],
    p2_bytes: &[u8],
) -> Result<[u8; G1_LEN], PrecompileHalt> {
    let mut p1 = read_g1_point(p1_bytes)?;
    let p2 = read_g1_point(p2_bytes)?;
    p1.add_assign(&p2);
    Ok(encode_g1_point(p1))
}

#[inline]
pub(crate) fn g1_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; G1_LEN], PrecompileHalt> {
    let p = read_g1_point(point_bytes)?;
    let scalar = read_scalar(scalar_bytes);
    Ok(encode_g1_point(p.mul(scalar)))
}

#[inline]
pub(crate) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileHalt> {
    let mut g1_points = Vec::with_capacity(pairs.len());
    let mut g2_points = Vec::with_capacity(pairs.len());

    for (g1_bytes, g2_bytes) in pairs {
        let g1 = read_g1_point(g1_bytes)?;
        let g2 = read_g2_point(g2_bytes)?;

        if !g1.is_zero() && !g2.is_zero() {
            g1_points.push(g1);
            g2_points.push(g2);
        }
    }

    if g1_points.is_empty() {
        return Ok(true);
    }

    let Some(pairing_result) = BN254_PAIRING_ENGINE.pair(&g1_points, &g2_points) else {
        return Ok(false);
    };

    Ok(pairing_result == Fp12::one(&BN254_EXT12_FIELD))
}
