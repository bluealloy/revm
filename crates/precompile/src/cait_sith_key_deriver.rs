use super::calc_linear_cost_u32;
use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn, Vec};
use hd_keys_ecdsa::*;
use k256::elliptic_curve::sec1::FromEncodedPoint;
use k256::{elliptic_curve::sec1::ToEncodedPoint, Secp256k1};
use k256::{AffinePoint, ProjectivePoint};

pub const DERIVE_CAIT_SITH_PUBKEY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_address(4),
    Precompile::Standard(derive_cait_sith_pubkey as StandardPrecompileFn),
);

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

    //   struct RootKey {
    //     bytes pubkey;
    //     uint256 keyType;
    // }
    // parse input into (bytes32 derivedKeyId, RootKey[] memory rootHDKeys)

    let derived_key_id = &input[0..32];
    println!("derived_key_id: {:?}", derived_key_id);

    let root_hd_keys_data = &input[32..];

    let mut root_hd_keys = Vec::new();
    let mut i = 0;
    while i < root_hd_keys_data.len() {
        let mut pubkey_len: u64 = 0;
        for &byte in root_hd_keys_data[i..i + 8].iter() {
            pubkey_len = (pubkey_len << 8) | (byte as u64);
        }
        let pubkey_len = pubkey_len as usize;
        i += 8;
        let pubkey = &root_hd_keys_data[i..i + pubkey_len];
        i += pubkey_len;
        let _key_type = &root_hd_keys_data[i..i + 32];
        i += 32;
        let projective_point = bytes_to_projective_point(pubkey);
        root_hd_keys.push(projective_point);
    }

    println!("root_hd_keys: {:?}", root_hd_keys);

    let deriver = HdKeyDeriver::<Secp256k1>::new(
        derived_key_id,
        b"LIT_HD_KEY_ID_K256_XMD:SHA-256_SSWU_RO_NUL_",
    )
    .unwrap();
    let root_hd_keys: Vec<_> = root_hd_keys.into_iter().filter_map(|x| x).collect();
    let public = deriver.compute_public_key(&root_hd_keys);

    Ok((
        gas_used,
        public
            .to_affine()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec(),
    ))
}

fn bytes_to_projective_point(data: &[u8]) -> Option<ProjectivePoint> {
    let encoded_point = k256::EncodedPoint::from_bytes(data).ok()?;
    let affine_point = AffinePoint::from_encoded_point(&encoded_point);
    if !bool::from(affine_point.is_some()) {
        return None;
    }
    Some(ProjectivePoint::from(affine_point.unwrap()))
}
