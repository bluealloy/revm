use crate::{Address, Error, Precompile, PrecompileAddress, PrecompileResult};
use c_kzg::{Bytes32, Bytes48, KzgProof, KzgSettings};
use revm_primitives::{hex_literal::hex, Env};
use sha2::{Digest, Sha256};

pub const POINT_EVALUATION: PrecompileAddress = PrecompileAddress(ADDRESS, Precompile::Env(run));

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
fn run(input: &[u8], gas_limit: u64, env: &Env) -> PrecompileResult {
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
    Ok((GAS_COST, RETURN_VALUE.to_vec()))
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
    match KzgProof::verify_kzg_proof(commitment, z, y, proof, kzg_settings) {
        Ok(ok) => ok,
        #[cfg(not(debug_assertions))]
        Err(_) => false,
        #[cfg(debug_assertions)]
        Err(e) => {
            panic!("verify_kzg_proof returned an error: {e:?}");
        }
    }
}

#[inline(always)]
#[track_caller]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    bytes.try_into().expect("slice with incorrect length")
}

#[inline(always)]
#[track_caller]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

#[inline(always)]
#[track_caller]
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
        let env = Env::default();
        let (actual_gas, actual_output) = run(&input, gas, &env).unwrap();
        assert_eq!(actual_gas, gas);
        assert_eq!(actual_output, expected_output);
    }
}
