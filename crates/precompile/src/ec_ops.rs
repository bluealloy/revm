use blsful::Pairing;
use curve25519_dalek::EdwardsPoint;
use elliptic_curve::{
    group::{prime::PrimeCurveAffine, Curve, GroupEncoding},
    hash2curve::GroupDigest,
    ops::{Invert, Reduce},
    point::AffineCoordinates,
    sec1::ToEncodedPoint,
    CurveArithmetic, Field, Group, PrimeCurve, PrimeField, ScalarPrimitive,
};
use num::ToPrimitive;
use std::marker::PhantomData;

use super::{calc_linear_cost_u32, extract_points, IDENTITY_BASE, IDENTITY_PER_WORD};
use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn, Vec};

pub const EC_MUL: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(300),
    Precompile::Standard(ec_mul as StandardPrecompileFn),
);

pub const EC_ADD: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(301),
    Precompile::Standard(ec_add as StandardPrecompileFn),
);

pub const EC_NEG: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(302),
    Precompile::Standard(ec_neg as StandardPrecompileFn),
);

pub const EC_EQUAL: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(303),
    Precompile::Standard(ec_equal as StandardPrecompileFn),
);

pub const EC_IS_INFINITY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(304),
    Precompile::Standard(ec_is_infinity as StandardPrecompileFn),
);

pub const EC_IS_VALID: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(305),
    Precompile::Standard(ec_is_valid as StandardPrecompileFn),
);

pub const EC_HASH: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(306),
    Precompile::Standard(ec_hash as StandardPrecompileFn),
);

pub const EC_SUM_OF_PRODUCTS: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(307),
    Precompile::Standard(ec_sum_of_products as StandardPrecompileFn),
);

pub const SCALAR_ADD: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(308),
    Precompile::Standard(scalar_add as StandardPrecompileFn),
);

pub const SCALAR_MUL: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(309),
    Precompile::Standard(scalar_mul as StandardPrecompileFn),
);

pub const SCALAR_NEG: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(310),
    Precompile::Standard(scalar_neg as StandardPrecompileFn),
);

pub const SCALAR_INV: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(311),
    Precompile::Standard(scalar_inv as StandardPrecompileFn),
);

pub const SCALAR_SQRT: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(312),
    Precompile::Standard(scalar_sqrt as StandardPrecompileFn),
);

pub const SCALAR_EQUAL: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(313),
    Precompile::Standard(scalar_equal as StandardPrecompileFn),
);

pub const SCALAR_IS_ZERO: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(314),
    Precompile::Standard(scalar_is_zero as StandardPrecompileFn),
);

pub const SCALAR_IS_VALID: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(315),
    Precompile::Standard(scalar_is_valid as StandardPrecompileFn),
);

pub const SCALAR_FROM_WIDE_BYTES: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(316),
    Precompile::Standard(scalar_from_wide_bytes as StandardPrecompileFn),
);

pub const SCALAR_HASH: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(317),
    Precompile::Standard(scalar_hash as StandardPrecompileFn),
);

pub const ECDSA_VERIFY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(318),
    Precompile::Standard(ecdsa_verify as StandardPrecompileFn),
);

pub const SCHNORR_VERIFY1: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(319),
    Precompile::Standard(schnorr_verify1 as StandardPrecompileFn),
);

pub const SCHNORR_VERIFY2: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(320),
    Precompile::Standard(schnorr_verify2 as StandardPrecompileFn),
);

pub const BLS_VERIFY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(321),
    Precompile::Standard(bls_verify as StandardPrecompileFn),
);

fn ec_mul(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcMultiply {}.handle(input, gas_limit)
}

fn ec_add(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcAdd {}.handle(input, gas_limit)
}

fn ec_neg(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcNeg {}.handle(input, gas_limit)
}

fn ec_equal(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcEqual {}.handle(input, gas_limit)
}

fn ec_is_infinity(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcIsInfinity {}.handle(input, gas_limit)
}

fn ec_is_valid(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcIsValid {}.handle(input, gas_limit)
}

fn ec_hash(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcHash {}.handle(input, gas_limit)
}

fn ec_sum_of_products(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcSumOfProducts {}.handle(input, gas_limit)
}

fn scalar_add(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarAdd {}.handle(input, gas_limit)
}

fn scalar_mul(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarMul {}.handle(input, gas_limit)
}

fn scalar_neg(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarNeg {}.handle(input, gas_limit)
}

fn scalar_inv(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarInv {}.handle(input, gas_limit)
}

fn scalar_sqrt(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarSqrt {}.handle(input, gas_limit)
}

fn scalar_equal(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarEqual {}.handle(input, gas_limit)
}

fn scalar_is_zero(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarIsZero {}.handle(input, gas_limit)
}

fn scalar_is_valid(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarIsValid {}.handle(input, gas_limit)
}

fn scalar_from_wide_bytes(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarFromWideBytes {}.handle(input, gas_limit)
}

fn scalar_hash(input: &[u8], gas_limit: u64) -> PrecompileResult {
    ScalarHash {}.handle(input, gas_limit)
}

fn ecdsa_verify(input: &[u8], gas_limit: u64) -> PrecompileResult {
    EcdsaVerify {}.handle(input, gas_limit)
}

fn schnorr_verify1(input: &[u8], gas_limit: u64) -> PrecompileResult {
    SchnorrVerify1 {}.handle(input, gas_limit)
}

fn schnorr_verify2(input: &[u8], gas_limit: u64) -> PrecompileResult {
    SchnorrVerify2 {}.handle(input, gas_limit)
}

fn bls_verify(input: &[u8], gas_limit: u64) -> PrecompileResult {
    BlsVerify {}.handle(input, gas_limit)
}

const CURVE_NAME_SECP256K1: &[u8] = &[
    56, 59, 39, 83, 33, 83, 243, 83, 250, 76, 198, 137, 35, 159, 115, 101, 223, 233, 36, 235, 207,
    103, 128, 126, 182, 145, 99, 7, 164, 226, 112, 30,
];
const CURVE_NAME_PRIME256V1: &[u8] = &[
    236, 151, 14, 250, 71, 58, 162, 250, 152, 240, 56, 58, 218, 26, 64, 52, 15, 149, 88, 58, 236,
    119, 101, 93, 71, 74, 121, 18, 49, 68, 120, 167,
];
const CURVE_NAME_CURVE25519: &[u8] = &[
    95, 235, 190, 179, 75, 175, 72, 27, 200, 83, 4, 244, 249, 232, 242, 193, 145, 139, 223, 192,
    16, 239, 86, 182, 149, 122, 201, 43, 169, 112, 141, 196,
];
const CURVE_NAME_BLS12381G1: &[u8] = &[
    157, 137, 108, 202, 42, 239, 133, 106, 124, 17, 78, 140, 254, 165, 166, 3, 68, 236, 72, 237,
    26, 60, 125, 231, 225, 12, 198, 231, 69, 129, 98, 109,
];
const CURVE_NAME_BLS12381G2: &[u8] = &[
    234, 117, 92, 131, 99, 84, 34, 238, 113, 135, 28, 154, 84, 213, 205, 6, 52, 142, 9, 84, 93, 98,
    145, 179, 160, 123, 115, 254, 95, 105, 154, 249,
];
const CURVE_NAME_BLS12381GT: &[u8] = &[
    72, 104, 114, 249, 247, 74, 129, 138, 239, 93, 192, 105, 87, 88, 22, 147, 201, 72, 247, 204,
    168, 110, 248, 13, 211, 195, 253, 59, 152, 53, 40, 135,
];
const HASH_NAME_SHA2_256: &[u8] = &[
    231, 8, 169, 121, 9, 175, 229, 141, 81, 199, 223, 139, 162, 228, 170, 161, 233, 154, 116, 235,
    240, 211, 10, 216, 160, 162, 14, 213, 193, 29, 101, 84,
];
const HASH_NAME_SHA2_384: &[u8] = &[
    165, 231, 169, 152, 179, 76, 168, 208, 185, 190, 244, 4, 230, 133, 69, 8, 117, 4, 239, 14, 186,
    60, 224, 171, 107, 45, 169, 141, 56, 53, 132, 218,
];
const HASH_NAME_SHA2_512: &[u8] = &[
    108, 235, 120, 129, 121, 66, 58, 97, 47, 240, 51, 176, 106, 220, 211, 45, 31, 41, 13, 229, 190,
    86, 186, 224, 216, 251, 42, 59, 12, 137, 61, 187,
];
const HASH_NAME_SHA3_256: &[u8] = &[
    95, 185, 33, 85, 116, 164, 111, 26, 144, 41, 228, 98, 213, 136, 12, 218, 137, 103, 7, 6, 108,
    31, 75, 243, 13, 131, 136, 147, 145, 17, 191, 204,
];
const HASH_NAME_SHA3_384: &[u8] = &[
    109, 242, 159, 237, 211, 254, 58, 205, 67, 35, 215, 64, 115, 228, 107, 173, 74, 204, 7, 118,
    106, 22, 62, 188, 20, 44, 200, 203, 243, 1, 21, 100,
];
const HASH_NAME_SHA3_512: &[u8] = &[
    20, 64, 42, 213, 151, 220, 133, 115, 38, 130, 119, 163, 202, 176, 151, 54, 38, 167, 226, 26,
    193, 245, 177, 151, 249, 38, 251, 239, 42, 144, 199, 74,
];
const HASH_NAME_SHAKE128: &[u8] = &[
    82, 242, 139, 107, 140, 215, 88, 250, 189, 215, 74, 41, 202, 221, 102, 126, 152, 31, 74, 226,
    45, 64, 52, 33, 130, 102, 134, 86, 232, 127, 190, 59,
];
const HASH_NAME_SHAKE256: &[u8] = &[
    28, 128, 198, 113, 20, 210, 141, 235, 57, 106, 193, 29, 195, 23, 49, 25, 252, 247, 70, 234, 53,
    165, 151, 207, 109, 213, 180, 102, 191, 72, 169, 159,
];
const HASH_NAME_KECCAK256: &[u8] = &[
    7, 183, 43, 66, 46, 159, 31, 22, 175, 173, 79, 183, 247, 18, 28, 221, 255, 124, 31, 87, 161,
    229, 168, 198, 233, 193, 67, 1, 4, 63, 81, 56,
];
const HASH_NAME_TAPROOT: &[u8] = &[
    8, 215, 83, 31, 179, 38, 223, 4, 226, 165, 107, 122, 113, 187, 97, 125, 54, 221, 210, 133, 184,
    114, 109, 3, 149, 156, 81, 26, 98, 162, 91, 241,
];

trait EcOps {
    fn handle(&self, data: &[u8], gas_limit: u64) -> PrecompileResult {
        let gas_used = calc_linear_cost_u32(data.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
        if gas_used > gas_limit {
            return Err(Error::OutOfGas);
        }

        let mut i = 0;
        while i < data.len() {
            if i + 32 > data.len() {
                return Err(Error::EcOpsInvalidCurve);
            }
            match &data[i..i + 32] {
                CURVE_NAME_SECP256K1 => {
                    println!("secp256k1");
                    let result = self.secp256k1(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                CURVE_NAME_PRIME256V1 => {
                    println!("prime256v1");
                    let result = self.prime256v1(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                CURVE_NAME_CURVE25519 => {
                    println!("curve25519");
                    let result = self.curve25519(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                CURVE_NAME_BLS12381G1 => {
                    println!("bls12381g1");
                    let result = self.bls12381g1(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                CURVE_NAME_BLS12381G2 => {
                    println!("bls12381g2");
                    let result = self.bls12381g2(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                CURVE_NAME_BLS12381GT => {
                    println!("bls12381gt");
                    let result = self.bls12381g1(&data[i + 32..])?;
                    return Ok((gas_used, result));
                }
                _ => {}
            };
            i += 1;
        }
        Err(Error::EcOpsInvalidCurve)
    }
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}

fn parse_hash<'a>(data: &'a [u8]) -> Result<(&'a [u8], Box<dyn SchnorrChallenge>), Error> {
    match &data[..32] {
        HASH_NAME_SHA2_256 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha2::Sha256> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHA2_384 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha2::Sha384> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHA2_512 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha2::Sha512> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHA3_256 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha3::Sha3_256> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHA3_384 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha3::Sha3_384> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHA3_512 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha3::Sha3_512> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_KECCAK256 => Ok((
            &data[32..],
            Box::new(SchnorrFixedDigest::<sha3::Keccak256> {
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_TAPROOT => Ok((&data[32..], Box::new(SchnorrHashTaproot {}))),
        HASH_NAME_SHAKE128 => Ok((
            &data[32..],
            Box::new(SchnorrXofDigest::<sha3::Shake128> {
                output_size: 32,
                _marker: PhantomData,
            }),
        )),
        HASH_NAME_SHAKE256 => Ok((
            &data[32..],
            Box::new(SchnorrXofDigest::<sha3::Shake256> {
                output_size: 64,
                _marker: PhantomData,
            }),
        )),
        _ => Err(Error::EcOpsInvalidHash),
    }
}

trait SchnorrChallenge {
    fn compute_challenge(&self, r: &[u8], pub_key: &[u8], msg: &[u8]) -> Vec<u8>;
}

struct SchnorrHashTaproot {}

struct SchnorrFixedDigest<D: sha2::Digest> {
    _marker: PhantomData<D>,
}

struct SchnorrXofDigest<D: Default + sha2::digest::ExtendableOutput + sha2::digest::Update> {
    output_size: usize,
    _marker: PhantomData<D>,
}

impl<D: sha2::Digest> SchnorrChallenge for SchnorrFixedDigest<D> {
    fn compute_challenge(&self, r: &[u8], pub_key: &[u8], msg: &[u8]) -> Vec<u8> {
        let mut hasher = D::new();
        hasher.update(r);
        hasher.update(pub_key);
        hasher.update(msg);
        hasher.finalize().to_vec()
    }
}

impl<D: Default + sha2::digest::ExtendableOutput + sha2::digest::Update> SchnorrChallenge
    for SchnorrXofDigest<D>
{
    fn compute_challenge(&self, r: &[u8], pub_key: &[u8], msg: &[u8]) -> Vec<u8> {
        let mut hasher = D::default();
        hasher.update(r);
        hasher.update(pub_key);
        hasher.update(msg);
        hasher.finalize_boxed(self.output_size).to_vec()
    }
}

impl SchnorrChallenge for SchnorrHashTaproot {
    fn compute_challenge(&self, r: &[u8], pub_key: &[u8], msg: &[u8]) -> Vec<u8> {
        use sha2::Digest;

        let tag_hash = sha2::Sha256::digest(b"BIP0340/challenge");
        let digest = sha2::Sha256::new();
        digest
            .chain_update(tag_hash)
            .chain_update(tag_hash)
            .chain_update(r)
            .chain_update(pub_key)
            .chain_update(msg)
            .finalize()
            .to_vec()
    }
}

fn secp256k1_points<'a>(
    data: &'a [u8],
    point_cnt: usize,
) -> Result<(&'a [u8], Vec<k256::ProjectivePoint>), Error> {
    if 64 * point_cnt > data.len() {
        return Err(Error::EcOpsInvalidPoint);
    }
    let points = extract_points::<k256::Secp256k1>(&data[..64 * point_cnt], point_cnt)
        .map_err(|_| Error::EcOpsInvalidPoint)?;
    if points.len() != point_cnt {
        return Err(Error::EcOpsInvalidPoint);
    }
    Ok((&data[64 * point_cnt..], points))
}

fn secp256k1_scalars<'a>(
    data: &'a [u8],
    scalar_cnt: usize,
) -> Result<(&'a [u8], Vec<k256::Scalar>), Error> {
    if 32 * scalar_cnt > data.len() {
        return Err(Error::EcOpsInvalidScalar);
    }
    let mut scalars = Vec::with_capacity(scalar_cnt);
    for i in 0..scalar_cnt {
        let mut repr = <k256::Scalar as PrimeField>::Repr::default();
        <<k256::Scalar as PrimeField>::Repr as AsMut<[u8]>>::as_mut(&mut repr)
            .copy_from_slice(&data[32 * i..32 * (i + 1)]);
        let scalar = Option::<k256::Scalar>::from(k256::Scalar::from_repr(repr))
            .ok_or(Error::EcOpsInvalidScalar)?;
        scalars.push(scalar);
    }
    Ok((&data[32 * scalar_cnt..], scalars))
}

fn prime256v1_points<'a>(
    data: &'a [u8],
    point_cnt: usize,
) -> Result<(&'a [u8], Vec<p256::ProjectivePoint>), Error> {
    if 64 * point_cnt > data.len() {
        return Err(Error::EcOpsInvalidPoint);
    }
    let points = extract_points::<p256::NistP256>(&data[..64 * point_cnt], point_cnt)
        .map_err(|_| Error::EcOpsInvalidPoint)?;
    if points.len() != point_cnt {
        return Err(Error::EcOpsInvalidPoint);
    }
    Ok((&data[64 * point_cnt..], points))
}

fn prime256v1_scalars<'a>(
    data: &'a [u8],
    scalar_cnt: usize,
) -> Result<(&'a [u8], Vec<p256::Scalar>), Error> {
    if 32 * scalar_cnt > data.len() {
        return Err(Error::EcOpsInvalidScalar);
    }
    let mut scalars = Vec::with_capacity(scalar_cnt);
    for i in 0..scalar_cnt {
        let mut repr = <p256::Scalar as PrimeField>::Repr::default();
        <<p256::Scalar as PrimeField>::Repr as AsMut<[u8]>>::as_mut(&mut repr)
            .copy_from_slice(&data[32 * i..32 * (i + 1)]);
        let scalar = Option::<p256::Scalar>::from(p256::Scalar::from_repr(repr))
            .ok_or(Error::EcOpsInvalidScalar)?;
        scalars.push(scalar);
    }
    Ok((&data[32 * scalar_cnt..], scalars))
}

fn curve25519_points<'a>(
    data: &'a [u8],
    point_cnt: usize,
) -> Result<(&'a [u8], Vec<curve25519_dalek::EdwardsPoint>), Error> {
    if 64 * point_cnt > data.len() {
        return Err(Error::EcOpsInvalidPoint);
    }
    let mut points = Vec::with_capacity(point_cnt);
    for i in 0..point_cnt {
        let compressed_point = curve25519_dalek::edwards::CompressedEdwardsY::from_slice(
            &data[(64 * i) + 32..64 * (i + 1)],
        )
        .map_err(|_| Error::EcOpsInvalidPoint)?;
        let point = compressed_point
            .decompress()
            .ok_or(Error::EcOpsInvalidPoint)?;
        points.push(point);
    }

    Ok((&data[64 * point_cnt..], points))
}

fn curve25519_scalars<'a>(
    data: &'a [u8],
    scalar_cnt: usize,
) -> Result<(&'a [u8], Vec<curve25519_dalek::Scalar>), Error> {
    if 32 * scalar_cnt > data.len() {
        return Err(Error::EcOpsInvalidScalar);
    }
    let mut scalars = Vec::with_capacity(scalar_cnt);
    for i in 0..scalar_cnt {
        let bytes = <[u8; 32]>::try_from(&data[32 * i..32 * (i + 1)]).unwrap();
        let scalar = Option::<curve25519_dalek::Scalar>::from(
            curve25519_dalek::Scalar::from_canonical_bytes(bytes),
        )
        .ok_or(Error::EcOpsInvalidScalar)?;
        scalars.push(scalar);
    }
    Ok((&data[32 * scalar_cnt..], scalars))
}

fn bls12381g1_points<'a>(
    data: &'a [u8],
    point_cnt: usize,
) -> Result<(&'a [u8], Vec<blsful::inner_types::G1Projective>), Error> {
    use blsful::inner_types::G1Projective;

    if G1Projective::UNCOMPRESSED_BYTES * point_cnt > data.len() {
        return Err(Error::EcOpsInvalidPoint);
    }
    let mut points = Vec::with_capacity(point_cnt);
    for i in 0..point_cnt {
        let bytes = <[u8; G1Projective::UNCOMPRESSED_BYTES]>::try_from(
            &data[G1Projective::UNCOMPRESSED_BYTES * i..G1Projective::UNCOMPRESSED_BYTES * (i + 1)],
        )
        .unwrap();
        let point = Option::<G1Projective>::from(G1Projective::from_uncompressed(&bytes))
            .ok_or(Error::EcOpsInvalidPoint)?;
        points.push(point);
    }

    Ok((
        &data[G1Projective::UNCOMPRESSED_BYTES * point_cnt..],
        points,
    ))
}

fn bls12381g2_points<'a>(
    data: &'a [u8],
    point_cnt: usize,
) -> Result<(&'a [u8], Vec<blsful::inner_types::G2Projective>), Error> {
    use blsful::inner_types::G2Projective;

    if G2Projective::UNCOMPRESSED_BYTES * point_cnt > data.len() {
        return Err(Error::EcOpsInvalidPoint);
    }
    let mut points = Vec::with_capacity(point_cnt);
    for i in 0..point_cnt {
        let bytes = <[u8; G2Projective::UNCOMPRESSED_BYTES]>::try_from(
            &data[G2Projective::UNCOMPRESSED_BYTES * i..G2Projective::UNCOMPRESSED_BYTES * (i + 1)],
        )
        .unwrap();
        let point = Option::<G2Projective>::from(G2Projective::from_uncompressed(&bytes))
            .ok_or(Error::EcOpsInvalidPoint)?;
        points.push(point);
    }

    Ok((
        &data[G2Projective::UNCOMPRESSED_BYTES * point_cnt..],
        points,
    ))
}

fn bls12381gt_scalar<'a>(
    data: &'a [u8],
    cnt: usize,
) -> Result<(&'a [u8], Vec<blsful::inner_types::Gt>), Error> {
    use blsful::inner_types::Gt;

    if Gt::BYTES * cnt > data.len() {
        return Err(Error::EcOpsInvalidScalar);
    }
    let mut scalars = Vec::with_capacity(cnt);
    for i in 0..cnt {
        let bytes = <[u8; Gt::BYTES]>::try_from(&data[Gt::BYTES * i..Gt::BYTES * (i + 1)]).unwrap();
        let scalar = Option::<Gt>::from(Gt::from_bytes(&bytes)).ok_or(Error::EcOpsInvalidScalar)?;
        scalars.push(scalar);
    }

    Ok((&data[Gt::BYTES * cnt..], scalars))
}

fn bls12381_scalars<'a>(
    data: &'a [u8],
    scalar_cnt: usize,
) -> Result<(&'a [u8], Vec<blsful::inner_types::Scalar>), Error> {
    if 32 * scalar_cnt > data.len() {
        return Err(Error::EcOpsInvalidScalar);
    }
    let mut scalars = Vec::with_capacity(scalar_cnt);
    for i in 0..scalar_cnt {
        let bytes = <[u8; 32]>::try_from(&data[32 * i..32 * (i + 1)]).unwrap();
        let scalar = Option::<blsful::inner_types::Scalar>::from(
            blsful::inner_types::Scalar::from_be_bytes(&bytes),
        )
        .ok_or(Error::EcOpsInvalidScalar)?;
        scalars.push(scalar);
    }
    Ok((&data[32 * scalar_cnt..], scalars))
}

fn read_usizes<'a>(data: &'a [u8], cnt: usize) -> Result<(&'a [u8], Vec<usize>), Error> {
    if 32 * cnt > data.len() {
        return Err(Error::EcOpsInvalidSize);
    }
    let mut lengths = Vec::with_capacity(cnt);
    for i in 0..cnt {
        let length = num_bigint::BigUint::from_bytes_be(&data[32 * i..32 * (i + 1)]);
        let length = length.to_usize().ok_or(Error::EcOpsInvalidSize)?;
        lengths.push(length);
    }
    Ok((&data[32 * cnt..], lengths))
}

fn secp256k1_point_out(point: &k256::ProjectivePoint) -> Vec<u8> {
    point.to_encoded_point(false).as_bytes()[1..].to_vec()
}

fn prime256v1_point_out(point: &p256::ProjectivePoint) -> Vec<u8> {
    point.to_encoded_point(false).as_bytes()[1..].to_vec()
}

fn curve25519_point_out(point: &curve25519_dalek::EdwardsPoint) -> Vec<u8> {
    let mut out = vec![0u8; 64];
    out[32..].copy_from_slice(point.compress().as_bytes());
    out
}

struct EcMultiply {}
struct EcAdd {}
struct EcNeg {}
struct EcEqual {}
struct EcIsInfinity {}
struct EcIsValid {}
struct EcHash {}
struct EcSumOfProducts {}
struct EcPairing {}
struct ScalarAdd {}
struct ScalarMul {}
struct ScalarNeg {}
struct ScalarInv {}
struct ScalarSqrt {}
struct ScalarEqual {}
struct ScalarIsZero {}
struct ScalarIsValid {}
struct ScalarFromWideBytes {}
struct ScalarHash {}
struct EcdsaVerify {}
struct SchnorrVerify1 {}
struct SchnorrVerify2 {}
struct BlsVerify {}

impl EcOps for EcMultiply {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = secp256k1_points(data, 1)?;
        let (_, scalars) = secp256k1_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(secp256k1_point_out(&point))
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = prime256v1_points(data, 1)?;
        let (_, scalars) = prime256v1_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(prime256v1_point_out(&point))
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = curve25519_points(data, 1)?;
        let (_, scalars) = curve25519_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(curve25519_point_out(&point))
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = bls12381g1_points(data, 1)?;
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = bls12381g2_points(data, 1)?;
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, points) = bls12381gt_scalar(data, 1)?;
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let point = points[0] * scalars[0];
        Ok(point.to_bytes().to_vec())
    }
}

impl EcOps for EcAdd {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = secp256k1_points(data, 2)?;
        let point = points[0] + points[1];
        Ok(secp256k1_point_out(&point))
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = prime256v1_points(data, 2)?;
        let point = points[0] + points[1];
        Ok(prime256v1_point_out(&point))
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = curve25519_points(data, 2)?;
        let point = points[0] + points[1];
        Ok(curve25519_point_out(&point))
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g1_points(data, 2)?;
        let point = points[0] + points[1];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g2_points(data, 2)?;
        let point = points[0] + points[1];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381gt_scalar(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }
}

impl EcOps for EcNeg {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = secp256k1_points(data, 1)?;
        let point = -points[0];
        Ok(secp256k1_point_out(&point))
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = prime256v1_points(data, 1)?;
        let point = -points[0];
        Ok(prime256v1_point_out(&point))
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = curve25519_points(data, 1)?;
        let point = -points[0];
        Ok(curve25519_point_out(&point))
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g1_points(data, 1)?;
        let point = -points[0];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g2_points(data, 1)?;
        let point = -points[0];
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381gt_scalar(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_bytes().to_vec())
    }
}

impl EcOps for EcEqual {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = secp256k1_points(data, 2)?;
        let res = points[0] == points[1];
        Ok(vec![res.into()])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = prime256v1_points(data, 2)?;
        let res = points[0] == points[1];
        Ok(vec![res.into()])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = curve25519_points(data, 2)?;
        let res = points[0] == points[1];
        Ok(vec![res.into()])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g1_points(data, 2)?;
        let res = points[0] == points[1];
        Ok(vec![res.into()])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g2_points(data, 2)?;
        let res = points[0] == points[1];
        Ok(vec![res.into()])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381gt_scalar(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }
}

impl EcOps for EcIsInfinity {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = secp256k1_points(data, 1)?;
        let res = points[0].is_identity().unwrap_u8();
        Ok(vec![res])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = prime256v1_points(data, 1)?;
        let res = points[0].is_identity().unwrap_u8();
        Ok(vec![res])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = curve25519_points(data, 1)?;
        let res = curve25519_dalek::traits::IsIdentity::is_identity(&points[0]);
        Ok(vec![res.into()])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g1_points(data, 1)?;
        let res = points[0].is_identity().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g2_points(data, 1)?;
        let res = points[0].is_identity().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381gt_scalar(data, 1)?;
        let res = scalars[0].is_identity().unwrap_u8();
        Ok(vec![res])
    }
}

impl EcOps for EcIsValid {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = secp256k1_points(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = prime256v1_points(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = curve25519_points(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g1_points(data, 1)?;
        let res = points[0].is_on_curve().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, points) = bls12381g2_points(data, 1)?;
        let res = points[0].is_on_curve().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, _scalars) = bls12381gt_scalar(data, 1)?;
        Ok(vec![1])
    }
}

impl EcOps for EcHash {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = k256::Secp256k1::hash_from_bytes::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&[&data[..lengths[0]]], &[b"secp256k1_XMD:SHA-256_SSWU_RO_"])
        .map_err(|_| Error::EcOpsInvalidPoint)?;
        Ok(secp256k1_point_out(&point))
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = p256::NistP256::hash_from_bytes::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&[&data[..lengths[0]]], &[b"P256_XMD:SHA-256_SSWU_RO_"])
        .map_err(|_| Error::EcOpsInvalidPoint)?;
        Ok(prime256v1_point_out(&point))
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = curve25519_dalek::EdwardsPoint::hash_to_curve::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha512>,
        >(&data[..lengths[0]], b"edwards25519_XMD:SHA-512_ELL2_RO_");
        Ok(curve25519_point_out(&point))
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = blsful::inner_types::G1Projective::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..lengths[0]], b"BLS12381G1_XMD:SHA-256_SSWU_RO_");
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = blsful::inner_types::G2Projective::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..lengths[0]], b"BLS12381G2_XMD:SHA-256_SSWU_RO_");
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let point = blsful::inner_types::G1Projective::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..lengths[0]], b"BLS12381G1_XMD:SHA-256_SSWU_RO_");
        Ok(blsful::Bls12381G1Impl::pairing(&[(
            point,
            blsful::inner_types::G2Projective::GENERATOR,
        )])
        .to_bytes()
        .to_vec())
    }
}

impl EcOps for EcSumOfProducts {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = secp256k1_points(data, cnt)?;
        let (_, scalars) = secp256k1_scalars(data, cnt)?;
        Ok(secp256k1_point_out(&sum_of_products_pippenger::<
            k256::Secp256k1,
        >(&points, &scalars)))
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = prime256v1_points(data, cnt)?;
        let (_, scalars) = prime256v1_scalars(data, cnt)?;
        Ok(prime256v1_point_out(&sum_of_products_pippenger::<
            p256::NistP256,
        >(&points, &scalars)))
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        use curve25519_dalek::traits::MultiscalarMul;

        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = curve25519_points(data, cnt)?;
        let (_, scalars) = curve25519_scalars(data, cnt)?;
        let point = EdwardsPoint::multiscalar_mul(scalars.iter(), points.iter());
        Ok(curve25519_point_out(&point))
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = bls12381g1_points(data, cnt)?;
        let (_, scalars) = bls12381_scalars(data, cnt)?;
        let point = blsful::inner_types::G1Projective::sum_of_products(&points, &scalars);
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = bls12381g2_points(data, cnt)?;
        let (_, scalars) = bls12381_scalars(data, cnt)?;
        let point = blsful::inner_types::G2Projective::sum_of_products(&points, &scalars);
        Ok(point.to_uncompressed().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points) = bls12381gt_scalar(data, cnt)?;
        let (_, scalars) = bls12381_scalars(data, cnt)?;
        let mut result = blsful::inner_types::Gt::IDENTITY;
        for i in 0..cnt {
            result += points[i] * scalars[i];
        }
        Ok(result.to_bytes().to_vec())
    }
}

impl EcOps for EcPairing {
    fn secp256k1(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsInvalidCurve)
    }

    fn prime256v1(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsInvalidCurve)
    }

    fn curve25519(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsInvalidCurve)
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points_g1) = bls12381g1_points(data, cnt)?;
        let (_, points_g2) = bls12381g2_points(data, cnt)?;
        let mut pairs = Vec::with_capacity(cnt);
        for i in 0..cnt {
            pairs.push((points_g1[i], points_g2[i]));
        }
        Ok(blsful::Bls12381G1Impl::pairing(pairs.as_slice())
            .to_bytes()
            .to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points_g1) = bls12381g1_points(data, cnt)?;
        let (_, points_g2) = bls12381g2_points(data, cnt)?;
        let mut pairs = Vec::with_capacity(cnt);
        for i in 0..cnt {
            pairs.push((points_g1[i], points_g2[i]));
        }
        Ok(blsful::Bls12381G1Impl::pairing(pairs.as_slice())
            .to_bytes()
            .to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let (data, points_g1) = bls12381g1_points(data, cnt)?;
        let (_, points_g2) = bls12381g2_points(data, cnt)?;
        let mut pairs = Vec::with_capacity(cnt);
        for i in 0..cnt {
            pairs.push((points_g1[i], points_g2[i]));
        }
        Ok(blsful::Bls12381G1Impl::pairing(pairs.as_slice())
            .to_bytes()
            .to_vec())
    }
}

impl EcOps for ScalarAdd {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] + scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for ScalarMul {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let scalar = scalars[0] * scalars[1];
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for ScalarNeg {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = -scalars[0];
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for ScalarInv {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 1)?;
        let scalar =
            Option::<k256::Scalar>::from(scalars[0].invert()).ok_or(Error::EcOpsInvalidScalar)?;
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 1)?;
        let scalar =
            Option::<p256::Scalar>::from(scalars[0].invert()).ok_or(Error::EcOpsInvalidScalar)?;
        Ok(scalar.to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 1)?;
        let scalar = scalars[0].invert();
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = Option::<blsful::inner_types::Scalar>::from(scalars[0].invert())
            .ok_or(Error::EcOpsInvalidScalar)?;
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = Option::<blsful::inner_types::Scalar>::from(scalars[0].invert())
            .ok_or(Error::EcOpsInvalidScalar)?;
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let scalar = Option::<blsful::inner_types::Scalar>::from(scalars[0].invert())
            .ok_or(Error::EcOpsInvalidScalar)?;
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for ScalarSqrt {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_be_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_be_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let (is_sqr, res) = scalars[0].sqrt_alt();
        if is_sqr.into() {
            Ok(res.to_be_bytes().to_vec())
        } else {
            Err(Error::EcOpsInvalidScalar)
        }
    }
}

impl EcOps for ScalarEqual {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 2)?;
        let res = scalars[0] == scalars[1];
        Ok(vec![res.into()])
    }
}

impl EcOps for ScalarIsZero {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = secp256k1_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = prime256v1_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = curve25519_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (_, scalars) = bls12381_scalars(data, 1)?;
        let res = scalars[0].is_zero().unwrap_u8();
        Ok(vec![res])
    }
}

impl EcOps for ScalarIsValid {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = secp256k1_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = prime256v1_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = curve25519_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = bls12381_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = bls12381_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let res = bls12381_scalars(data, 1).is_ok();
        Ok(vec![res.into()])
    }
}

impl EcOps for ScalarFromWideBytes {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let repr = k256::WideBytes::clone_from_slice(data);
        let scalar = <k256::Scalar as elliptic_curve::ops::Reduce<elliptic_curve::bigint::U512>>::reduce_bytes(&repr);
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let hi = p256::FieldBytes::from_slice(&data[..32]);
        let lo = p256::FieldBytes::from_slice(&data[32..]);

        let mut s0 = <p256::Scalar as elliptic_curve::ops::Reduce<elliptic_curve::bigint::U256>>::reduce_bytes(&hi);
        let s1 = <p256::Scalar as elliptic_curve::ops::Reduce<elliptic_curve::bigint::U256>>::reduce_bytes(&lo);
        for _ in 1..=256 {
            s0 = s0.double();
        }
        Ok((s0 + s1).to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let scalar = curve25519_dalek::Scalar::from_bytes_mod_order_wide(
            &<[u8; 64]>::try_from(data).unwrap(),
        );
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let scalar =
            blsful::inner_types::Scalar::from_bytes_wide(&<[u8; 64]>::try_from(data).unwrap());
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let scalar =
            blsful::inner_types::Scalar::from_bytes_wide(&<[u8; 64]>::try_from(data).unwrap());
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        if data.len() != 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let scalar =
            blsful::inner_types::Scalar::from_bytes_wide(&<[u8; 64]>::try_from(data).unwrap());
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for ScalarHash {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = k256::Secp256k1::hash_to_scalar::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&[&data[..cnt]], &[b"secp256k1_XMD:SHA-256_RO_"])
        .unwrap();
        Ok(scalar.to_bytes().to_vec())
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = p256::NistP256::hash_to_scalar::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&[&data[..cnt]], &[b"P256_XMD:SHA-256_RO_"])
        .unwrap();
        Ok(scalar.to_bytes().to_vec())
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = curve25519_dalek::Scalar::hash_from_bytes::<sha2::Sha512>(&data[..cnt]);
        Ok(scalar.to_bytes().to_vec())
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = blsful::inner_types::Scalar::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..cnt], b"BLS12381_XMD:SHA-256_RO_");
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = blsful::inner_types::Scalar::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..cnt], b"BLS12381_XMD:SHA-256_RO_");
        Ok(scalar.to_be_bytes().to_vec())
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        let scalar = blsful::inner_types::Scalar::hash::<
            elliptic_curve::hash2curve::ExpandMsgXmd<sha2::Sha256>,
        >(&data[..cnt], b"BLS12381_XMD:SHA-256_RO_");
        Ok(scalar.to_be_bytes().to_vec())
    }
}

impl EcOps for EcdsaVerify {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, scalars) = secp256k1_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = secp256k1_points(data, 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let signature = k256::ecdsa::Signature::from_slice(&data[..64])
            .map_err(|_| Error::EcOpsInvalidSignature)?;

        if verify_ecdsa(&points[0], &scalars[0], &signature).is_ok() {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, scalars) = prime256v1_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = prime256v1_points(data, 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSize);
        }
        let signature = p256::ecdsa::Signature::from_slice(&data[..64])
            .map_err(|_| Error::EcOpsInvalidSignature)?;

        if verify_ecdsa(&points[0], &scalars[0], &signature).is_ok() {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn curve25519(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn bls12381g1(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn bls12381g2(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn bls12381gt(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }
}

impl EcOps for SchnorrVerify1 {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = secp256k1_points(&data[32..], 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }

        let r_bytes = (&data[..32]).into();
        let r = Option::<k256::FieldElement>::from(k256::FieldElement::from_bytes(r_bytes))
            .ok_or(Error::EcOpsInvalidScalar)?;
        if r.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let s =
            k256::NonZeroScalar::try_from(&data[32..]).map_err(|_| Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(r_bytes, &points[0].to_bytes()[1..], &msg);
        let e = <k256::Scalar as Reduce<k256::U256>>::reduce_bytes((&e_bytes[..]).into());

        let big_r = (k256::ProjectivePoint::GENERATOR * s.as_ref() - points[0] * e).to_affine();

        if big_r.is_identity().into() || &big_r.x() != r_bytes {
            Ok(vec![0u8])
        } else {
            Ok(vec![1u8])
        }
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = prime256v1_points(&data[32..], 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }

        let r_bytes = (&data[..32]).into();
        let r = Option::<p256::FieldElement>::from(p256::FieldElement::from_bytes(r_bytes))
            .ok_or(Error::EcOpsInvalidScalar)?;
        if r.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let s =
            p256::NonZeroScalar::try_from(&data[32..]).map_err(|_| Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(r_bytes, &points[0].to_bytes()[1..], &msg);
        let e = <p256::Scalar as Reduce<p256::U256>>::reduce_bytes((&e_bytes[..]).into());

        let big_r = (p256::ProjectivePoint::GENERATOR * s.as_ref() - points[0] * e).to_affine();

        if big_r.is_identity().into() || &big_r.x() != r_bytes {
            Ok(vec![0u8])
        } else {
            Ok(vec![1u8])
        }
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = curve25519_points(&data[32..], 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }
        let e_bytes = hasher.compute_challenge(&data[..32], points[0].compress().as_bytes(), &msg);
        let mut e_arr = [0u8; 64];
        e_arr[..e_bytes.len()].copy_from_slice(&e_bytes[..]);
        let e = curve25519_dalek::Scalar::from_bytes_mod_order_wide(&e_arr);
        let s_bytes = <[u8; 32]>::try_from(&data[32..64]).unwrap();
        let s = Option::<curve25519_dalek::Scalar>::from(
            curve25519_dalek::Scalar::from_canonical_bytes(s_bytes),
        )
        .ok_or(Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let r = curve25519_dalek::edwards::CompressedEdwardsY::from_slice(&data[..32])
            .map_err(|_| Error::EcOpsInvalidScalar)?;
        if curve25519_dalek::traits::IsIdentity::is_identity(&r) {
            return Err(Error::EcOpsInvalidPoint);
        }

        let big_r =
            EdwardsPoint::vartime_double_scalar_mul_basepoint(&e, &-points[0], &s).compress();
        if big_r == r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = bls12381g1_points(&data[32..], 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes =
            hasher.compute_challenge(sig_r.to_bytes().as_ref(), pk.to_bytes().as_ref(), &msg);
        let mut e_arr = [0u8; 64];
        e_arr[64 - e_bytes.len()..].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::G1Projective::GENERATOR * sig_s[0] - pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = bls12381g2_points(&data[32..], 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes =
            hasher.compute_challenge(sig_r.to_bytes().as_ref(), pk.to_bytes().as_ref(), &msg);
        let mut e_arr = [0u8; 64];
        e_arr[64 - e_bytes.len()..].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::G2Projective::GENERATOR * sig_s[0] - pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        if data.len() < 32 {
            return Err(Error::EcOpsInvalidSize);
        }
        let msg = &data[..32];
        let (data, points) = bls12381gt_scalar(&data[32..], 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes =
            hasher.compute_challenge(sig_r.to_bytes().as_ref(), pk.to_bytes().as_ref(), &msg);
        let mut e_arr = [0u8; 64];
        e_arr[64 - e_bytes.len()..].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::Gt::generator() * sig_s[0] - pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }
}

impl EcOps for SchnorrVerify2 {
    fn secp256k1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = secp256k1_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = secp256k1_points(data, 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }

        let r_bytes = (&data[..32]).into();
        let r = Option::<k256::FieldElement>::from(k256::FieldElement::from_bytes(r_bytes))
            .ok_or(Error::EcOpsInvalidScalar)?;
        if r.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let s =
            k256::NonZeroScalar::try_from(&data[32..]).map_err(|_| Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(
            r_bytes,
            &points[0].to_bytes()[..],
            &scalars[0].to_bytes()[..],
        );
        let e = <k256::Scalar as Reduce<k256::U256>>::reduce_bytes((&e_bytes[..]).into());

        let big_r = (k256::ProjectivePoint::GENERATOR * s.as_ref() + points[0] * e).to_affine();

        if big_r.is_identity().into() || &big_r.x() != r_bytes {
            Ok(vec![0u8])
        } else {
            Ok(vec![1u8])
        }
    }

    fn prime256v1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = prime256v1_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = prime256v1_points(data, 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }

        let r_bytes = (&data[..32]).into();
        let r = Option::<p256::FieldElement>::from(p256::FieldElement::from_bytes(r_bytes))
            .ok_or(Error::EcOpsInvalidScalar)?;
        if r.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let s =
            p256::NonZeroScalar::try_from(&data[32..]).map_err(|_| Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(
            r_bytes,
            &points[0].to_bytes()[..],
            &scalars[0].to_bytes()[..],
        );
        let e = <p256::Scalar as Reduce<p256::U256>>::reduce_bytes((&e_bytes[..]).into());

        let big_r = (p256::ProjectivePoint::GENERATOR * s.as_ref() + points[0] * e).to_affine();

        if big_r.is_identity().into() || &big_r.x() != r_bytes {
            Ok(vec![0u8])
        } else {
            Ok(vec![1u8])
        }
    }

    fn curve25519(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = curve25519_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = curve25519_points(data, 1)?;
        if points[0].is_identity().into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        if data.len() < 64 {
            return Err(Error::EcOpsInvalidSignature);
        }
        let e_bytes = hasher.compute_challenge(
            &data[..32],
            points[0].compress().as_bytes(),
            scalars[0].as_bytes(),
        );
        let mut e_arr = [0u8; 64];
        e_arr[..e_bytes.len()].copy_from_slice(&e_bytes[..]);
        let e = curve25519_dalek::Scalar::from_bytes_mod_order_wide(&e_arr);
        let s_bytes = <[u8; 32]>::try_from(&data[32..64]).unwrap();
        let s = Option::<curve25519_dalek::Scalar>::from(
            curve25519_dalek::Scalar::from_canonical_bytes(s_bytes),
        )
        .ok_or(Error::EcOpsInvalidScalar)?;
        if s.is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let r = curve25519_dalek::edwards::CompressedEdwardsY::from_slice(&data[..32])
            .map_err(|_| Error::EcOpsInvalidScalar)?;
        if curve25519_dalek::traits::IsIdentity::is_identity(&r) {
            return Err(Error::EcOpsInvalidPoint);
        }

        let big_r =
            EdwardsPoint::vartime_double_scalar_mul_basepoint(&e, &points[0], &s).compress();
        if big_r == r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = bls12381_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = bls12381g1_points(data, 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(
            sig_r.to_bytes().as_ref(),
            pk.to_bytes().as_ref(),
            scalars[0].to_be_bytes().as_ref(),
        );
        let mut e_arr = [0u8; 64];
        e_arr[..e_bytes.len()].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::G1Projective::GENERATOR * sig_s[0] + pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = bls12381_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = bls12381g2_points(data, 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidPoint);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(
            sig_r.to_bytes().as_ref(),
            pk.to_bytes().as_ref(),
            scalars[0].to_be_bytes().as_ref(),
        );
        let mut e_arr = [0u8; 64];
        e_arr[..e_bytes.len()].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::G2Projective::GENERATOR * sig_s[0] + pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381gt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, hasher) = parse_hash(data)?;
        let (data, scalars) = bls12381_scalars(data, 1)?;
        if scalars[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let (data, points) = bls12381gt_scalar(data, 2)?;
        if (points[0].is_identity() | points[1].is_identity()).into() {
            return Err(Error::EcOpsInvalidScalar);
        }
        let pk = points[0];
        let sig_r = points[1];
        let (_, sig_s) = bls12381_scalars(data, 1)?;
        if sig_s[0].is_zero().into() {
            return Err(Error::EcOpsInvalidScalar);
        }

        let e_bytes = hasher.compute_challenge(
            sig_r.to_bytes().as_ref(),
            pk.to_bytes().as_ref(),
            scalars[0].to_be_bytes().as_ref(),
        );
        let mut e_arr = [0u8; 64];
        e_arr[..e_bytes.len()].copy_from_slice(&e_bytes[..]);
        let e = blsful::inner_types::Scalar::from_bytes_wide(&e_arr);

        let big_r = blsful::inner_types::Gt::generator() * sig_s[0] + pk * e;
        if big_r == sig_r {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }
}

impl EcOps for BlsVerify {
    fn secp256k1(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn prime256v1(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn curve25519(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }

    fn bls12381g1(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        if cnt > data.len() {
            return Err(Error::EcOpsInvalidSize);
        }
        let message = &data[..cnt];
        let (data, g1points) = bls12381g1_points(&data[cnt..], 1)?;
        let (_, g2points) = bls12381g2_points(data, 1)?;
        let pk = blsful::PublicKey::<blsful::Bls12381G2Impl>(g1points[0]);
        let sig = blsful::Signature::<blsful::Bls12381G2Impl>::ProofOfPossession(g2points[0]);
        if sig
            .verify(&pk, message)
            .map_err(|_| Error::EcOpsInvalidSignature)
            .is_ok()
        {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381g2(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (data, lengths) = read_usizes(data, 1)?;
        let cnt = lengths[0];
        if cnt > data.len() {
            return Err(Error::EcOpsInvalidSize);
        }
        let message = &data[..cnt];
        let (data, g1points) = bls12381g2_points(&data[cnt..], 1)?;
        let (_, g2points) = bls12381g1_points(data, 1)?;

        let pk = blsful::PublicKey::<blsful::Bls12381G1Impl>(g1points[0]);
        let sig = blsful::Signature::<blsful::Bls12381G1Impl>::ProofOfPossession(g2points[0]);
        if sig
            .verify(&pk, message)
            .map_err(|_| Error::EcOpsInvalidSignature)
            .is_ok()
        {
            Ok(vec![1u8])
        } else {
            Ok(vec![0u8])
        }
    }

    fn bls12381gt(&self, _data: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::EcOpsNotSupported)
    }
}

fn verify_ecdsa<C>(
    q: &elliptic_curve::ProjectivePoint<C>,
    z: &elliptic_curve::Scalar<C>,
    sig: &ecdsa::Signature<C>,
) -> Result<(), Error>
where
    C: PrimeCurve + CurveArithmetic,
    ecdsa::SignatureSize<C>: elliptic_curve::generic_array::ArrayLength<u8>,
{
    let (r, s) = sig.split_scalars();
    if (r.is_zero() | s.is_zero()).into() {
        return Err(Error::EcOpsInvalidSignature);
    }

    let s_inv = *s.invert_vartime();
    let u1 = *z * s_inv;
    let u2 = *r * s_inv;
    let x = (elliptic_curve::ProjectivePoint::<C>::generator() * u1 + *q * u2)
        .to_affine()
        .x();
    if *r == elliptic_curve::Scalar::<C>::reduce_bytes(&x) {
        Ok(())
    } else {
        Err(Error::EcOpsInvalidSignature)
    }
}

fn sum_of_products_pippenger<C: CurveArithmetic>(
    points: &[C::ProjectivePoint],
    scalars: &[C::Scalar],
) -> C::ProjectivePoint {
    const WINDOW: usize = 4;
    const NUM_BUCKETS: usize = 1 << WINDOW;
    const EDGE: usize = WINDOW - 1;
    const MASK: u64 = (NUM_BUCKETS - 1) as u64;

    let scalars = convert_scalars::<C>(scalars);
    let num_components = std::cmp::min(points.len(), scalars.len());
    let mut buckets = [<C::ProjectivePoint as Group>::identity(); NUM_BUCKETS];
    let mut res = C::ProjectivePoint::identity();
    let mut num_doubles = 0;
    let mut bit_sequence_index = 255usize;

    loop {
        for _ in 0..num_doubles {
            res = res.double();
        }

        let mut max_bucket = 0;
        let word_index = bit_sequence_index >> 6;
        let bit_index = bit_sequence_index & 63;

        if bit_index < EDGE {
            // we are on the edge of a word; have to look at the previous word, if it exists
            if word_index == 0 {
                // there is no word before
                let smaller_mask = ((1 << (bit_index + 1)) - 1) as u64;
                for i in 0..num_components {
                    let bucket_index: usize = (scalars[i][word_index] & smaller_mask) as usize;
                    if bucket_index > 0 {
                        buckets[bucket_index] += points[i];
                        if bucket_index > max_bucket {
                            max_bucket = bucket_index;
                        }
                    }
                }
            } else {
                // there is a word before
                let high_order_mask = ((1 << (bit_index + 1)) - 1) as u64;
                let high_order_shift = EDGE - bit_index;
                let low_order_mask = ((1 << high_order_shift) - 1) as u64;
                let low_order_shift = 64 - high_order_shift;
                let prev_word_index = word_index - 1;
                for i in 0..num_components {
                    let mut bucket_index =
                        ((scalars[i][word_index] & high_order_mask) << high_order_shift) as usize;
                    bucket_index |= ((scalars[i][prev_word_index] >> low_order_shift)
                        & low_order_mask) as usize;
                    if bucket_index > 0 {
                        buckets[bucket_index] += points[i];
                        if bucket_index > max_bucket {
                            max_bucket = bucket_index;
                        }
                    }
                }
            }
        } else {
            let shift = bit_index - EDGE;
            for i in 0..num_components {
                let bucket_index: usize = ((scalars[i][word_index] >> shift) & MASK) as usize;
                if bucket_index > 0 {
                    buckets[bucket_index] += points[i];
                    if bucket_index > max_bucket {
                        max_bucket = bucket_index;
                    }
                }
            }
        }
        res += &buckets[max_bucket];
        for i in (1..max_bucket).rev() {
            buckets[i] += buckets[i + 1];
            res += buckets[i];
            buckets[i + 1] = C::ProjectivePoint::identity();
        }
        buckets[1] = C::ProjectivePoint::identity();
        if bit_sequence_index < WINDOW {
            break;
        }
        bit_sequence_index -= WINDOW;
        num_doubles = {
            if bit_sequence_index < EDGE {
                bit_sequence_index + 1
            } else {
                WINDOW
            }
        };
    }
    res
}

#[cfg(target_pointer_width = "32")]
fn convert_scalars<C: CurveArithmetic>(scalars: &[C::Scalar]) -> Vec<[u64; 4]> {
    scalars
        .iter()
        .map(|s| {
            let mut out = [0u64; 4];
            let primitive: ScalarPrimitive<C> = (*s).into();
            let small_limbs = primitive
                .as_limbs()
                .iter()
                .map(|l| l.0 as u64)
                .collect::<Vec<_>>();
            let mut i = 0;
            let mut j = 0;
            while i < small_limbs.len() && j < out.len() {
                out[j] = small_limbs[i + 1] << 32 | small_limbs[i];
                i += 2;
                j += 1;
            }
            out
        })
        .collect::<Vec<_>>()
}

#[cfg(target_pointer_width = "64")]
fn convert_scalars<C: CurveArithmetic>(scalars: &[C::Scalar]) -> Vec<[u64; 4]> {
    scalars
        .iter()
        .map(|s| {
            let mut out = [0u64; 4];
            let primitive: ScalarPrimitive<C> = (*s).into();
            out.copy_from_slice(
                primitive
                    .as_limbs()
                    .iter()
                    .map(|l| l.0 as u64)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            out
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;
    const HASH_MSG: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    #[test]
    fn ecc_sum_of_products_secp256k1() {
        let points = vec![
            k256::ProjectivePoint::GENERATOR,
            k256::ProjectivePoint::GENERATOR,
            k256::ProjectivePoint::GENERATOR,
            k256::ProjectivePoint::GENERATOR,
        ];
        let scalars = vec![
            k256::Scalar::from(1u64),
            k256::Scalar::from(2u64),
            k256::Scalar::from(3u64),
            k256::Scalar::from(4u64),
        ];
        let expected = k256::ProjectivePoint::GENERATOR * k256::Scalar::from(10u64);
        let mut input = CURVE_NAME_SECP256K1.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(4);
        input.extend_from_slice(&secp256k1_point_out(&points[0]));
        input.extend_from_slice(&secp256k1_point_out(&points[1]));
        input.extend_from_slice(&secp256k1_point_out(&points[2]));
        input.extend_from_slice(&secp256k1_point_out(&points[3]));
        input.extend_from_slice(&&scalars[0].to_bytes()[..]);
        input.extend_from_slice(&&scalars[1].to_bytes()[..]);
        input.extend_from_slice(&&scalars[2].to_bytes()[..]);
        input.extend_from_slice(&&scalars[3].to_bytes()[..]);
        let res = ec_sum_of_products(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(
            expected.to_encoded_point(false).as_bytes()[1..].to_vec(),
            bytes
        );
    }

    #[test]
    fn ecc_sum_of_products_prime256v1() {
        let points = vec![
            p256::ProjectivePoint::GENERATOR,
            p256::ProjectivePoint::GENERATOR,
            p256::ProjectivePoint::GENERATOR,
            p256::ProjectivePoint::GENERATOR,
        ];
        let scalars = vec![
            p256::Scalar::from(1u64),
            p256::Scalar::from(2u64),
            p256::Scalar::from(3u64),
            p256::Scalar::from(4u64),
        ];
        let expected = p256::ProjectivePoint::GENERATOR * p256::Scalar::from(10u64);
        let mut input = CURVE_NAME_PRIME256V1.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(4);
        input.extend_from_slice(&prime256v1_point_out(&points[0]));
        input.extend_from_slice(&prime256v1_point_out(&points[1]));
        input.extend_from_slice(&prime256v1_point_out(&points[2]));
        input.extend_from_slice(&prime256v1_point_out(&points[3]));
        input.extend_from_slice(&&scalars[0].to_bytes()[..]);
        input.extend_from_slice(&&scalars[1].to_bytes()[..]);
        input.extend_from_slice(&&scalars[2].to_bytes()[..]);
        input.extend_from_slice(&&scalars[3].to_bytes()[..]);
        let res = ec_sum_of_products(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(
            expected.to_encoded_point(false).as_bytes()[1..].to_vec(),
            bytes
        );
    }

    #[test]
    fn ecc_sum_of_products_curve25519() {
        let points = vec![
            curve25519_dalek::constants::ED25519_BASEPOINT_POINT,
            curve25519_dalek::constants::ED25519_BASEPOINT_POINT,
            curve25519_dalek::constants::ED25519_BASEPOINT_POINT,
            curve25519_dalek::constants::ED25519_BASEPOINT_POINT,
        ];
        let scalars = vec![
            curve25519_dalek::Scalar::from(1u64),
            curve25519_dalek::Scalar::from(2u64),
            curve25519_dalek::Scalar::from(3u64),
            curve25519_dalek::Scalar::from(4u64),
        ];
        let expected = curve25519_dalek::constants::ED25519_BASEPOINT_POINT
            * curve25519_dalek::Scalar::from(10u64);
        let mut input = CURVE_NAME_CURVE25519.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(4);
        input.extend_from_slice(&curve25519_point_out(&points[0]));
        input.extend_from_slice(&curve25519_point_out(&points[1]));
        input.extend_from_slice(&curve25519_point_out(&points[2]));
        input.extend_from_slice(&curve25519_point_out(&points[3]));
        input.extend_from_slice(&&scalars[0].to_bytes()[..]);
        input.extend_from_slice(&&scalars[1].to_bytes()[..]);
        input.extend_from_slice(&&scalars[2].to_bytes()[..]);
        input.extend_from_slice(&&scalars[3].to_bytes()[..]);
        let res = ec_sum_of_products(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(&expected.compress().as_bytes()[..], &bytes[32..]);
    }

    #[test]
    fn ecc_sum_of_products_bls12381g1() {
        let points = vec![
            blsful::inner_types::G1Projective::GENERATOR,
            blsful::inner_types::G1Projective::GENERATOR,
            blsful::inner_types::G1Projective::GENERATOR,
            blsful::inner_types::G1Projective::GENERATOR,
        ];
        let scalars = vec![
            blsful::inner_types::Scalar::from(1u64),
            blsful::inner_types::Scalar::from(2u64),
            blsful::inner_types::Scalar::from(3u64),
            blsful::inner_types::Scalar::from(4u64),
        ];
        let expected =
            blsful::inner_types::G1Projective::GENERATOR * blsful::inner_types::Scalar::from(10u64);
        let mut input = CURVE_NAME_BLS12381G1.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(4);
        input.extend_from_slice(&points[0].to_uncompressed());
        input.extend_from_slice(&points[1].to_uncompressed());
        input.extend_from_slice(&points[2].to_uncompressed());
        input.extend_from_slice(&points[3].to_uncompressed());
        input.extend_from_slice(&&scalars[0].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[1].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[2].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[3].to_be_bytes()[..]);
        let res = ec_sum_of_products(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_uncompressed().to_vec(), bytes);
    }

    #[test]
    fn ecc_sum_of_products_bls12381g2() {
        let points = vec![
            blsful::inner_types::G2Projective::GENERATOR,
            blsful::inner_types::G2Projective::GENERATOR,
            blsful::inner_types::G2Projective::GENERATOR,
            blsful::inner_types::G2Projective::GENERATOR,
        ];
        let scalars = vec![
            blsful::inner_types::Scalar::from(1u64),
            blsful::inner_types::Scalar::from(2u64),
            blsful::inner_types::Scalar::from(3u64),
            blsful::inner_types::Scalar::from(4u64),
        ];
        let expected =
            blsful::inner_types::G2Projective::GENERATOR * blsful::inner_types::Scalar::from(10u64);
        let mut input = CURVE_NAME_BLS12381G2.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(4);
        input.extend_from_slice(&points[0].to_uncompressed());
        input.extend_from_slice(&points[1].to_uncompressed());
        input.extend_from_slice(&points[2].to_uncompressed());
        input.extend_from_slice(&points[3].to_uncompressed());
        input.extend_from_slice(&&scalars[0].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[1].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[2].to_be_bytes()[..]);
        input.extend_from_slice(&&scalars[3].to_be_bytes()[..]);
        let res = ec_sum_of_products(&input, 200);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_uncompressed().to_vec(), bytes);
    }

    #[test]
    fn ecc_mul_secp256k1() {
        let mut input = CURVE_NAME_SECP256K1.to_vec();
        let pt = k256::ProjectivePoint::GENERATOR.to_encoded_point(false);
        input.extend_from_slice(&pt.x().unwrap());
        input.extend_from_slice(&pt.y().unwrap());
        input.extend_from_slice(&(k256::Scalar::from(100u64)).to_bytes());
        let expected = k256::ProjectivePoint::GENERATOR * k256::Scalar::from(100u64);
        let res = ec_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(
            expected.to_encoded_point(false).as_bytes()[1..].to_vec(),
            bytes
        );
    }

    #[test]
    fn ecc_mul_prime256v1() {
        let mut input = CURVE_NAME_PRIME256V1.to_vec();
        let pt = p256::ProjectivePoint::GENERATOR.to_encoded_point(false);
        input.extend_from_slice(&pt.x().unwrap());
        input.extend_from_slice(&pt.y().unwrap());
        input.extend_from_slice(&(p256::Scalar::from(100u64)).to_bytes());
        let expected = p256::ProjectivePoint::generator() * p256::Scalar::from(100u64);
        let res = ec_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(
            expected.to_encoded_point(false).as_bytes()[1..].to_vec(),
            bytes
        );
    }

    #[test]
    fn ecc_mul_curve25519() {
        let mut input = CURVE_NAME_CURVE25519.to_vec();
        let pt = curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
        input.extend_from_slice(&[0u8; 32]);
        input.extend_from_slice(&pt.to_bytes());
        input.extend_from_slice(&(curve25519_dalek::Scalar::from(100u64)).to_bytes());
        let expected = pt * curve25519_dalek::Scalar::from(100u64);
        let res = ec_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_bytes(), bytes[32..]);
    }

    #[test]
    fn ecc_mul_bls12381g1() {
        let mut input = CURVE_NAME_BLS12381G1.to_vec();
        let pt = blsful::inner_types::G1Projective::GENERATOR;
        input.extend_from_slice(&pt.to_uncompressed());
        input.extend_from_slice(&(blsful::inner_types::Scalar::from(100u64)).to_be_bytes());
        let expected = pt * blsful::inner_types::Scalar::from(100u64);
        let res = ec_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_uncompressed().to_vec(), bytes);
    }

    #[test]
    fn ecc_mul_bls12381g2() {
        let mut input = CURVE_NAME_BLS12381G2.to_vec();
        let pt = blsful::inner_types::G2Projective::GENERATOR;
        input.extend_from_slice(&pt.to_uncompressed());
        input.extend_from_slice(&(blsful::inner_types::Scalar::from(100u64)).to_be_bytes());
        let expected = pt * blsful::inner_types::Scalar::from(100u64);
        let res = ec_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_uncompressed().to_vec(), bytes);
    }

    #[test]
    fn scalar_hash_secp256k1() {
        let mut input = CURVE_NAME_SECP256K1.to_vec();
        let length = (HASH_MSG.len() as u32).to_be_bytes();
        let mut arr = [0u8; 32];
        arr[32 - length.len()..].copy_from_slice(&length);
        input.extend_from_slice(&arr);
        input.extend_from_slice(HASH_MSG);
        let res = scalar_hash(&input, 200);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        let expected = [
            0x2f, 0xad, 0x45, 0xa6, 0x27, 0xe2, 0xa5, 0x3f, 0x58, 0xcc, 0xa3, 0x17, 0xe7, 0xc8,
            0x2f, 0x73, 0x8d, 0x09, 0x15, 0x81, 0x6d, 0xb2, 0xee, 0xcc, 0x3c, 0xa8, 0x38, 0x00,
            0xb5, 0x32, 0xac, 0xb9,
        ];
        assert_eq!(expected.to_vec(), bytes);
    }

    #[test]
    fn scalar_mul_secp256k1() {
        let sc1 = k256::Scalar::from(100u64);
        let sc2 = k256::Scalar::from(200u64);

        let mut input = CURVE_NAME_SECP256K1.to_vec();
        input.extend_from_slice(&sc1.to_bytes());
        input.extend_from_slice(&sc2.to_bytes());
        let expected = sc1 * sc2;
        let res = scalar_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_bytes().to_vec(), bytes);
    }

    #[test]
    fn scalar_mul_prime256v1() {
        let sc1 = p256::Scalar::from(100u64);
        let sc2 = p256::Scalar::from(200u64);

        let mut input = CURVE_NAME_PRIME256V1.to_vec();
        input.extend_from_slice(&sc1.to_bytes());
        input.extend_from_slice(&sc2.to_bytes());
        let expected = sc1 * sc2;
        let res = scalar_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_bytes().to_vec(), bytes);
    }

    #[test]
    fn scalar_mul_curve25519() {
        let sc1 = curve25519_dalek::Scalar::from(100u64);
        let sc2 = curve25519_dalek::Scalar::from(200u64);

        let mut input = CURVE_NAME_CURVE25519.to_vec();
        input.extend_from_slice(&sc1.to_bytes());
        input.extend_from_slice(&sc2.to_bytes());
        let expected = sc1 * sc2;
        let res = scalar_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_bytes().to_vec(), bytes);
    }

    #[test]
    fn scalar_mul_bls12381() {
        let sc1 = blsful::inner_types::Scalar::from(100u64);
        let sc2 = blsful::inner_types::Scalar::from(200u64);

        let mut input = CURVE_NAME_BLS12381G1.to_vec();
        input.extend_from_slice(&sc1.to_be_bytes());
        input.extend_from_slice(&sc2.to_be_bytes());
        let expected = sc1 * sc2;
        let res = scalar_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_be_bytes().to_vec(), bytes);

        input[..32].copy_from_slice(CURVE_NAME_BLS12381G2);
        let res = scalar_mul(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(expected.to_be_bytes().to_vec(), bytes);
    }

    #[test]
    fn ecdsa_verify_secp256k1() {
        use k256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
        use sha2::Digest;

        let sign_key = SigningKey::random(&mut rand::rngs::OsRng);
        let verify_key = VerifyingKey::from(&sign_key);
        let signature: Signature = sign_key.sign(HASH_MSG);

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);
        let hashed_message = <k256::Scalar as Reduce<k256::U256>>::reduce_bytes(&hashed_msg_bytes);

        let mut input = CURVE_NAME_SECP256K1.to_vec();
        input.extend_from_slice(&hashed_message.to_bytes());
        input.extend_from_slice(&verify_key.to_encoded_point(false).as_bytes()[1..]);
        input.extend_from_slice(&signature.to_bytes());
        let res = ecdsa_verify(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn ecdsa_verify_prime256v1() {
        use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
        use sha2::Digest;

        let sign_key = SigningKey::random(&mut rand::rngs::OsRng);
        let verify_key = VerifyingKey::from(&sign_key);
        let signature: Signature = sign_key.sign(HASH_MSG);

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);
        let hashed_message = <p256::Scalar as Reduce<k256::U256>>::reduce_bytes(&hashed_msg_bytes);

        let mut input = CURVE_NAME_PRIME256V1.to_vec();
        input.extend_from_slice(&hashed_message.to_bytes());
        input.extend_from_slice(&verify_key.to_encoded_point(false).as_bytes()[1..]);
        input.extend_from_slice(&signature.to_bytes());
        let res = ecdsa_verify(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn ecdsa_not_supported_curves() {
        let mut input = CURVE_NAME_CURVE25519.to_vec();
        input.extend_from_slice(&[0u8; 32]);
        input.extend_from_slice(&[0u8; 64]);
        input.extend_from_slice(&[0u8; 64]);
        let res = ecdsa_verify(&input, 100);
        assert!(res.is_err());

        input[..32].copy_from_slice(CURVE_NAME_BLS12381G1);
        let res = ecdsa_verify(&input, 100);
        assert!(res.is_err());

        input[..32].copy_from_slice(CURVE_NAME_BLS12381G2);
        let res = ecdsa_verify(&input, 100);
        assert!(res.is_err());
    }

    #[test]
    fn schnorr_verify_secp256k1() {
        use elliptic_curve::Field;
        use k256::{ProjectivePoint, Scalar};
        use sha2::Digest;

        let sign_key = Scalar::random(&mut rand::rngs::OsRng);
        let verify_key = ProjectivePoint::GENERATOR * sign_key;

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let little_r = Scalar::random(&mut rand::rngs::OsRng);
        let big_r = ProjectivePoint::GENERATOR * little_r;
        let mut sha256 = sha2::Sha256::new();
        sha256.update(big_r.to_affine().x());
        sha256.update(&verify_key.to_bytes()[1..]);
        sha256.update(&hashed_msg_bytes);
        let e_bytes = sha256.finalize();
        let e = <Scalar as Reduce<k256::U256>>::reduce_bytes(&e_bytes);
        let s = little_r + e * sign_key;

        let mut input = CURVE_NAME_SECP256K1.to_vec();
        input.extend_from_slice(&HASH_NAME_SHA2_256);
        input.extend_from_slice(&hashed_msg_bytes);
        input.extend_from_slice(&verify_key.to_encoded_point(false).as_bytes()[1..]);
        input.extend_from_slice(&big_r.to_affine().x());
        input.extend_from_slice(&s.to_bytes());
        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn schnorr_verify_secp256k1_taproot() {
        use k256::{
            schnorr::{signature::Signer, Signature, SigningKey},
            ProjectivePoint,
        };
        use sha2::Digest;

        let sign_key = SigningKey::random(&mut rand::rngs::OsRng);
        let verify_key = sign_key.verifying_key();
        let signature: Signature = sign_key.sign(HASH_MSG);

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let mut input = CURVE_NAME_SECP256K1.to_vec();
        input.extend_from_slice(&HASH_NAME_TAPROOT);
        input.extend_from_slice(&hashed_msg_bytes);
        let point = ProjectivePoint::from(verify_key.as_affine());
        input.extend_from_slice(&point.to_encoded_point(false).as_bytes()[1..]);
        input.extend_from_slice(&signature.to_bytes());

        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn schnorr_verify_prime256v1() {
        use elliptic_curve::Field;
        use p256::{ProjectivePoint, Scalar};
        use sha2::Digest;

        let sign_key = Scalar::random(&mut rand::rngs::OsRng);
        let verify_key = ProjectivePoint::GENERATOR * sign_key;

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let little_r = Scalar::random(&mut rand::rngs::OsRng);
        let big_r = ProjectivePoint::GENERATOR * little_r;
        let mut sha256 = sha2::Sha256::new();
        sha256.update(big_r.to_affine().x());
        sha256.update(&verify_key.to_bytes()[1..]);
        sha256.update(&hashed_msg_bytes);
        let e_bytes = sha256.finalize();
        let e = <Scalar as Reduce<k256::U256>>::reduce_bytes(&e_bytes);
        let s = little_r + e * sign_key;

        let mut input = CURVE_NAME_PRIME256V1.to_vec();
        input.extend_from_slice(&HASH_NAME_SHA2_256);
        input.extend_from_slice(&hashed_msg_bytes);
        input.extend_from_slice(&verify_key.to_encoded_point(false).as_bytes()[1..]);
        input.extend_from_slice(&big_r.to_affine().x());
        input.extend_from_slice(&s.to_bytes());
        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn schnorr_verify_curve25519() {
        use ed25519_dalek::Signer;
        use rand::Rng;
        use sha2::Digest;

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let secret_key = rand::rngs::OsRng.gen::<[u8; 32]>();
        let sign_key = ed25519_dalek::SigningKey::from_bytes(&secret_key);
        let verify_key = ed25519_dalek::VerifyingKey::from(&sign_key);

        let signature = sign_key.sign(&hashed_msg_bytes);

        let mut input = CURVE_NAME_CURVE25519.to_vec();
        input.extend_from_slice(&HASH_NAME_SHA2_512);
        input.extend_from_slice(&hashed_msg_bytes);
        input.extend_from_slice(&[0u8; 32]);
        input.extend_from_slice(&verify_key.to_bytes());
        input.extend_from_slice(&signature.to_bytes());

        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn schnorr_verify_bls12831g1() {
        use blsful::inner_types::{G1Projective, Scalar};
        use elliptic_curve::Field;
        use sha2::Digest;

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let sign_key = Scalar::random(&mut rand::rngs::OsRng);
        let verify_key = G1Projective::GENERATOR * sign_key;

        let little_r = Scalar::random(&mut rand::rngs::OsRng);
        let big_r = G1Projective::GENERATOR * little_r;
        let mut sha384 = sha2::Sha384::new();
        sha384.update(big_r.to_bytes().as_ref());
        sha384.update(&verify_key.to_bytes().as_ref());
        sha384.update(&hashed_msg_bytes);
        let e_bytes = sha384.finalize();
        let mut e_arr = [0u8; 64];
        e_arr[64 - e_bytes.len()..].copy_from_slice(&e_bytes[..]);
        let e = Scalar::from_bytes_wide(&e_arr);
        let s = little_r + e * sign_key;

        let mut input = CURVE_NAME_BLS12381G1.to_vec();
        input.extend_from_slice(&HASH_NAME_SHA2_384);
        input.extend_from_slice(&hashed_msg_bytes);
        input.extend_from_slice(&verify_key.to_uncompressed());
        input.extend_from_slice(&big_r.to_uncompressed());
        input.extend_from_slice(&s.to_be_bytes());

        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn schnorr_verify_bls12831g2() {
        use blsful::inner_types::{G2Projective, Scalar};
        use elliptic_curve::Field;
        use sha2::Digest;

        let hashed_msg_bytes = sha2::Sha256::digest(HASH_MSG);

        let sign_key = Scalar::random(&mut rand::rngs::OsRng);
        let verify_key = G2Projective::GENERATOR * sign_key;

        let little_r = Scalar::random(&mut rand::rngs::OsRng);
        let big_r = G2Projective::GENERATOR * little_r;
        let mut sha384 = sha2::Sha384::new();
        sha384.update(big_r.to_bytes().as_ref());
        sha384.update(&verify_key.to_bytes().as_ref());
        sha384.update(&hashed_msg_bytes);
        let e_bytes = sha384.finalize();
        let mut e_arr = [0u8; 64];
        e_arr[64 - e_bytes.len()..].copy_from_slice(&e_bytes[..]);
        let e = Scalar::from_bytes_wide(&e_arr);
        let s = little_r + e * sign_key;

        let mut input = CURVE_NAME_BLS12381G2.to_vec();
        input.extend_from_slice(&HASH_NAME_SHA2_384);
        input.extend_from_slice(&hashed_msg_bytes);
        input.extend_from_slice(&verify_key.to_uncompressed());
        input.extend_from_slice(&big_r.to_uncompressed());
        input.extend_from_slice(&s.to_be_bytes());

        let res = schnorr_verify1(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn bls_bls12381g1_verify() {
        use blsful::{Bls12381G2, PublicKey, SignatureSchemes};

        let sign_key = Bls12381G2::new_secret_key();
        let verify_key = PublicKey::from(&sign_key);
        let signature = sign_key
            .sign(SignatureSchemes::ProofOfPossession, &HASH_MSG)
            .unwrap();

        let mut input = CURVE_NAME_BLS12381G1.to_vec();
        let length = (HASH_MSG.len() as u32).to_be_bytes();
        let mut arr = [0u8; 32];
        arr[32 - length.len()..].copy_from_slice(&length);
        input.extend_from_slice(&arr);
        input.extend_from_slice(HASH_MSG);
        input.extend_from_slice(&verify_key.0.to_uncompressed());
        input.extend_from_slice(&signature.as_raw_value().to_uncompressed());
        let res = bls_verify(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();
        assert_eq!(bytes, vec![1u8]);
    }

    #[test]
    fn ecc_hash_curve25519() {
        let mut input = CURVE_NAME_CURVE25519.to_vec();
        input.extend_from_slice(&[0u8; 31]);
        input.push(32);
        input.extend_from_slice(&[0u8; 32]);
        let res = ec_hash(&input, 100);
        assert!(res.is_ok());
        let (_, bytes) = res.unwrap();

        let mut temp = [0u8; 32];
        hex::decode_to_slice(
            "4838562f360e7087a5b2c6e867836ab6dd3b8d20c923eb2b535902739060bf09",
            &mut temp,
        )
        .unwrap();
        let expected = curve25519_dalek::EdwardsPoint::from_bytes(&temp).unwrap();
        assert_eq!(expected.compress().as_bytes()[..], bytes[32..]);
    }
}
