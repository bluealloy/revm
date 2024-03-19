use crate::{Address, Error, Precompile, PrecompileResult, PrecompileWithAddress};
use c_kzg::{Bytes32, Bytes48, KzgProof, KzgSettings};
use revm_primitives::{hex_literal::hex, Bytes, Env};
use sha2::{Digest, Sha256};

pub const POINT_EVALUATION: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Env(run));

const ADDRESS: Address = crate::u64_to_address(0x0A);
const GAS_COST: u64 = 50_000;
const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// `U256(FIELD_ELEMENTS_PER_BLOB).to_be_bytes() ++ BLS_MODULUS.to_bytes32()`
const RETURN_VALUE: &[u8; 64] = &hex!(
    "0000000000000000000000000000000000000000000000000000000000001000"
    "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
);

/// Run kzg point evaluation precompile.
///
/// The Env has the KZGSettings that is needed for evaluation.
///
/// The input is encoded as follows:
/// | versioned_hash |  z  |  y  | commitment | proof |
/// |     32         | 32  | 32  |     48     |   48  |
/// with z and y being padded 32 byte big endian values
pub fn run(input: &Bytes, gas_limit: u64, env: &Env) -> PrecompileResult {
    if gas_limit < GAS_COST {
        return Err(Error::OutOfGas);
    }

    // Verify input length.
    if input.len() != 192 {
        return Err(Error::BlobInvalidInputLength);
    }

    // Verify commitment matches versioned_hash
    let versioned_hash = &input[..32];
    let commitment = &input[96..144];
    if kzg_to_versioned_hash(commitment) != versioned_hash {
        return Err(Error::BlobMismatchedVersion);
    }

    // Verify KZG proof with z and y in big endian format
    let commitment = as_bytes48(commitment);
    let z = as_bytes32(&input[32..64]);
    let y = as_bytes32(&input[64..96]);
    let proof = as_bytes48(&input[144..192]);
    if !verify_kzg_proof(commitment, z, y, proof, env.cfg.kzg_settings.get()) {
        return Err(Error::BlobVerifyKzgProofFailed);
    }

    // Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values
    Ok((GAS_COST, RETURN_VALUE.into()))
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
#[inline]
fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
    let mut hash: [u8; 32] = Sha256::digest(commitment).into();
    hash[0] = VERSIONED_HASH_VERSION_KZG;
    hash
}

#[inline]
fn verify_kzg_proof(
    commitment: &Bytes48,
    z: &Bytes32,
    y: &Bytes32,
    proof: &Bytes48,
    kzg_settings: &KzgSettings,
) -> bool {
    KzgProof::verify_kzg_proof(commitment, z, y, proof, kzg_settings).unwrap_or(false)
}

#[inline]
#[track_caller]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    bytes.try_into().expect("slice with incorrect length")
}

#[inline]
#[track_caller]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

#[inline]
#[track_caller]
fn as_bytes48(bytes: &[u8]) -> &Bytes48 {
    // SAFETY: `#[repr(C)] Bytes48([u8; 48])`
    unsafe { &*as_array::<48>(bytes).as_ptr().cast() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        // test data from: https://github.com/ethereum/c-kzg-4844/blob/main/tests/verify_kzg_proof/kzg-mainnet/verify_kzg_proof_case_correct_proof_31ebd010e6098750/data.yaml

        let commitment = hex!("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").to_vec();
        let mut versioned_hash = Sha256::digest(&commitment).to_vec();
        versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
        let z = hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").to_vec();
        let y = hex!("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").to_vec();
        let proof = hex!("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").to_vec();

        let input = [versioned_hash, z, y, commitment, proof].concat();

        let expected_output = hex!("000000000000000000000000000000000000000000000000000000000000100073eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");
        let gas = 50000;
        let env = Env::default();
        let (actual_gas, actual_output) = run(&input.into(), gas, &env).unwrap();
        assert_eq!(actual_gas, gas);
        assert_eq!(actual_output[..], expected_output);
    }
}
