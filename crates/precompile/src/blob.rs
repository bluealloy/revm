use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, B160};
use c_kzg::{bindings, Bytes32, Bytes48, CkzgError};
use revm_primitives::hex_literal::hex;
use sha2::{Digest, Sha256};

pub mod kzg_settings;

pub const POINT_EVALUATION: PrecompileAddress =
    PrecompileAddress(ADDRESS, Precompile::Standard(run));

const ADDRESS: B160 = crate::u64_to_b160(0x0A);
const GAS_COST: u64 = 50_000;
const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// `U256(FIELD_ELEMENTS_PER_BLOB).to_be_bytes() ++ BLS_MODULUS.to_bytes32()`
const RETURN_VALUE: &[u8; 64] = &hex!(
    "0000000000000000000000000000000000000000000000000000000000001000"
    "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
);

fn run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if gas_limit < GAS_COST {
        return Err(Error::OutOfGas);
    }
    if input.len() != 192 {
        return Err(Error::BlobInvalidInputLength);
    }

    // Verify commitment matches versioned_hash
    let commitment = &input[96..144];
    let versioned_hash = &input[0..32];
    if kzg_to_versioned_hash(commitment) != versioned_hash {
        return Err(Error::BlobMismatchedVersion);
    }

    // Verify KZG proof
    let commitment = as_bytes48(commitment);
    let z = as_bytes32(&input[32..64]);
    let y = as_bytes32(&input[64..96]);
    let proof = as_bytes48(&input[144..192]);
    if !verify_kzg_proof(commitment, z, y, proof) {
        return Err(Error::BlobVerifyKzgProofFailed);
    }

    Ok((GAS_COST, RETURN_VALUE.to_vec()))
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
    let mut hash: [u8; 32] = Sha256::digest(commitment).into();
    hash[0] = VERSIONED_HASH_VERSION_KZG;
    hash
}

fn verify_kzg_proof(commitment: &Bytes48, z: &Bytes32, y: &Bytes32, proof: &Bytes48) -> bool {
    // note: we use the bindings directly to avoid copying the input bytes.
    // If `KzgProof::verify_kzg_proof` ever changes to take references, we should use that instead.
    let mut ok = false;
    let ret = unsafe {
        bindings::verify_kzg_proof(
            &mut ok,
            commitment,
            z,
            y,
            proof,
            kzg_settings::get_global_or_default(),
        )
    };
    if ret != CkzgError::C_KZG_OK {
        #[cfg(debug_assertions)]
        panic!("verify_kzg_proof returned an error: {ret:?}");

        #[cfg(not(debug_assertions))]
        return false;
    }
    ok
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    debug_assert_eq!(bytes.len(), N);
    // SAFETY: Length is checked above
    unsafe { &*bytes.as_ptr().cast() }
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_bytes48(bytes: &[u8]) -> &Bytes48 {
    // SAFETY: `#[repr(C)] Bytes48([u8; 48])`
    unsafe { &*as_array::<48>(bytes).as_ptr().cast() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://github.com/ethereum/go-ethereum/blob/41ee96fdfee5924004e8fbf9bbc8aef783893917/core/vm/testdata/precompiles/pointEvaluation.json
    #[test]
    fn basic_test() {
        let input = hex!("01d18459b334ffe8e2226eef1db874fda6db2bdd9357268b39220af2d59464fb564c0a11a0f704f4fc3e8acfe0f8245f0ad1347b378fbf96e206da11a5d3630624d25032e67a7e6a4910df5834b8fe70e6bcfeeac0352434196bdf4b2485d5a1978a0d595c823c05947b1156175e72634a377808384256e9921ebf72181890be2d6b58d4a73a880541d1656875654806942307f266e636553e94006d11423f2688945ff3bdf515859eba1005c1a7708d620a94d91a1c0c285f9584e75ec2f82a");
        let expected_output = hex!("000000000000000000000000000000000000000000000000000000000000100073eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");
        let gas = 50000;
        let (actual_gas, actual_output) = run(&input, gas).unwrap();
        assert_eq!(actual_gas, gas);
        assert_eq!(actual_output, expected_output);
    }
}
