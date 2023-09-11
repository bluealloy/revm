use super::calc_linear_cost_u32;
use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn, Vec};
use elliptic_curve::{
    group::cofactor::CofactorGroup,
    hash2curve::{FromOkm, GroupDigest},
    sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint},
    Curve, CurveArithmetic,
};
use hd_keys_ecdsa::*;

pub const DERIVE_CAIT_SITH_PUBKEY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_address(100),
    Precompile::Standard(derive_cait_sith_pubkey as StandardPrecompileFn),
);

const MIN_LENGTH: usize = 81;
/// The base cost of the operation.
const IDENTITY_BASE: u64 = 15;
/// The cost per word.
const IDENTITY_PER_WORD: u64 = 3;

fn derive_cait_sith_pubkey(input: &[u8], gas_limit: u64) -> PrecompileResult {
    println!("derive_cait_sith_pubkey");
    let gas_used = calc_linear_cost_u32(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
    if gas_used > gas_limit {
        return Err(Error::OutOfGas);
    }

    for i in 0..input.len() {
        match input[i] {
            0 => {
                if let Ok(params) = DeriveParams::<p256::NistP256>::try_from(&input[i + 1..]) {
                    let deriver =
                        HdKeyDeriver::<p256::NistP256>::new(&params.id, &params.cxt).unwrap();

                    println!("root_hd_keys: {:?}", params.root_hd_keys);
                    let public = deriver.compute_public_key(&params.root_hd_keys);

                    return Ok((
                        gas_used,
                        public
                            .to_affine()
                            .to_encoded_point(false)
                            .as_bytes()
                            .to_vec(),
                    ));
                }
            }
            1 => {
                if let Ok(params) = DeriveParams::<k256::Secp256k1>::try_from(&input[i + 1..]) {
                    let deriver =
                        HdKeyDeriver::<k256::Secp256k1>::new(&params.id, &params.cxt).unwrap();

                    println!("root_hd_keys: {:?}", params.root_hd_keys);
                    let public = deriver.compute_public_key(&params.root_hd_keys);

                    return Ok((
                        gas_used,
                        public
                            .to_affine()
                            .to_encoded_point(false)
                            .as_bytes()
                            .to_vec(),
                    ));
                }
            }
            _ => {}
        }

        if input.len() - i < MIN_LENGTH {
            break;
        }
    }
    return Err(Error::OutOfGas);
}

fn bytes_to_projective_point<C>(data: &[u8]) -> Option<C::ProjectivePoint>
where
    C: GroupDigest,
    <C as CurveArithmetic>::ProjectivePoint: CofactorGroup,
    <C as CurveArithmetic>::AffinePoint: FromEncodedPoint<C>,
    <C as CurveArithmetic>::Scalar: FromOkm,
    <C as Curve>::FieldBytesSize: ModulusSize,
{
    let encoded_point = EncodedPoint::<C>::from_bytes(data).ok()?;
    let point = <C::AffinePoint as FromEncodedPoint<C>>::from_encoded_point(&encoded_point)
        .map(C::ProjectivePoint::from);
    Option::<C::ProjectivePoint>::from(point)
}

struct DeriveParams<C>
where
    C: GroupDigest,
    <C as CurveArithmetic>::ProjectivePoint: CofactorGroup,
    <C as CurveArithmetic>::AffinePoint: FromEncodedPoint<C>,
    <C as CurveArithmetic>::Scalar: FromOkm,
    <C as Curve>::FieldBytesSize: ModulusSize,
{
    id: Vec<u8>,
    cxt: Vec<u8>,
    root_hd_keys: Vec<C::ProjectivePoint>,
}

impl<C> TryFrom<&[u8]> for DeriveParams<C>
where
    C: GroupDigest,
    <C as CurveArithmetic>::ProjectivePoint: CofactorGroup,
    <C as CurveArithmetic>::AffinePoint: FromEncodedPoint<C>,
    <C as CurveArithmetic>::Scalar: FromOkm,
    <C as Curve>::FieldBytesSize: ModulusSize,
{
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let err = Err(format!("invalid length for derive params: {}", value.len()));
        if value.len() < MIN_LENGTH {
            return err;
        }

        let mut offset = 0;
        if offset + 4 > value.len() {
            return err;
        }
        let id_len = u32::from_be_bytes([
            value[offset],
            value[offset + 1],
            value[offset + 2],
            value[offset + 3],
        ]) as usize;
        offset += 4;
        if offset + id_len > value.len() || id_len == 0 {
            return err;
        }
        let id = value[offset..offset + id_len].to_vec();
        offset += id_len;
        if offset + 4 > value.len() {
            return err;
        }
        let cxt_len = u32::from_be_bytes([
            value[offset],
            value[offset + 1],
            value[offset + 2],
            value[offset + 3],
        ]) as usize;
        offset += 4;
        if offset + cxt_len > value.len() || cxt_len == 0 {
            return err;
        }
        let cxt = value[offset..offset + cxt_len].to_vec();
        let pks_cnt = u32::from_be_bytes([
            value[offset],
            value[offset + 1],
            value[offset + 2],
            value[offset + 3],
        ]) as usize;

        if pks_cnt == 0 || (offset + pks_cnt * 33) > value.len() {
            return err;
        }

        let root_hd_keys = extract_points::<C>(&value[offset..], pks_cnt)?;
        Ok(DeriveParams {
            id,
            cxt,
            root_hd_keys,
        })
    }
}

fn extract_points<C>(data: &[u8], pks_cnt: usize) -> Result<Vec<C::ProjectivePoint>, String>
where
    C: GroupDigest,
    <C as CurveArithmetic>::ProjectivePoint: CofactorGroup,
    <C as CurveArithmetic>::AffinePoint: FromEncodedPoint<C>,
    <C as CurveArithmetic>::Scalar: FromOkm,
    <C as Curve>::FieldBytesSize: ModulusSize,
{
    let mut offset = 0;
    let mut points = Vec::with_capacity(pks_cnt);
    while offset < data.len() && points.len() < pks_cnt {
        let point = match data[offset] {
            0x04 => {
                // Uncompressed form
                if offset + 65 > data.len() {
                    return Err(format!(
                        "invalid length for uncompressed point: {}",
                        data.len()
                    ));
                }
                let point = bytes_to_projective_point::<C>(&data[offset..offset + 65]);
                offset += 65;
                point
            }
            0x03 | 0x02 => {
                // Compressed form
                if offset + 33 > data.len() {
                    return Err(format!(
                        "invalid length for compressed point: {}",
                        data.len()
                    ));
                }
                let point = bytes_to_projective_point::<C>(&data[offset..offset + 33]);
                offset += 33;
                point
            }
            _ => {
                if offset + 64 > data.len() {
                    return Err(format!("invalid length for hybrid point: {}", data.len()));
                }
                let mut tmp = [4u8; 65];
                tmp[1..].copy_from_slice(&data[offset..offset + 64]);
                let point = bytes_to_projective_point::<C>(&data[offset..offset + 65]);
                offset += 65;
                point
            }
        };
        if point.is_none() {
            return Err(format!("invalid point at offset {}", offset));
        }
        points.push(point.unwrap());
    }
    Ok(points)
}
