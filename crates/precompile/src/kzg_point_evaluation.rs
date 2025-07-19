//! KZG point evaluation precompile added in [`EIP-4844`](https://eips.ethereum.org/EIPS/eip-4844)
//! For more details check [`run`] function.
use crate::{
    crypto_provider::get_provider, Address, PrecompileError, PrecompileOutput, PrecompileResult,
    PrecompileWithAddress,
};
use primitives::hex_literal::hex;

/// KZG point evaluation precompile, containing address and function to run.
pub const POINT_EVALUATION: PrecompileWithAddress = PrecompileWithAddress(ADDRESS, run);

/// Address of the KZG point evaluation precompile.
pub const ADDRESS: Address = crate::u64_to_address(0x0A);

/// Gas cost of the KZG point evaluation precompile.
pub const GAS_COST: u64 = 50_000;

/// Versioned hash version for KZG.
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// `U256(FIELD_ELEMENTS_PER_BLOB).to_be_bytes() ++ BLS_MODULUS.to_bytes32()`
pub const RETURN_VALUE: &[u8; 64] = &hex!(
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
pub fn run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if gas_limit < GAS_COST {
        return Err(PrecompileError::OutOfGas);
    }

    // Verify input length.
    if input.len() != 192 {
        return Err(PrecompileError::BlobInvalidInputLength);
    }

    // Verify commitment matches versioned_hash
    let versioned_hash = &input[..32];
    let commitment = &input[96..144];
    if kzg_to_versioned_hash(commitment) != versioned_hash {
        return Err(PrecompileError::BlobMismatchedVersion);
    }

    // Verify KZG proof with z and y in big endian format
    let commitment: &[u8; 48] = commitment
        .try_into()
        .expect("expected `commitment` to be 48 bytes");
    let z = input[32..64]
        .try_into()
        .expect("expected `z` to be 32 bytes");
    let y = input[64..96]
        .try_into()
        .expect("expected `y` to be 32 bytes");
    let proof = input[144..192]
        .try_into()
        .expect("expected `proof` to be 48 bytes");

    if !crate::crypto_provider::get_provider().kzg_verify_proof(commitment, z, y, proof) {
        return Err(PrecompileError::BlobVerifyKzgProofFailed);
    }

    // Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values
    Ok(PrecompileOutput::new(GAS_COST, RETURN_VALUE.into()))
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
#[inline]
pub fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
    let mut hash = get_provider().sha256(commitment);
    hash[0] = VERSIONED_HASH_VERSION_KZG;
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        // Test data from: https://github.com/ethereum/c-kzg-4844/blob/main/tests/verify_kzg_proof/kzg-mainnet/verify_kzg_proof_case_correct_proof_4_4/data.yaml

        let commitment = hex!("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").to_vec();
        let versioned_hash = kzg_to_versioned_hash(&commitment).to_vec();
        let z = hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").to_vec();
        let y = hex!("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").to_vec();
        let proof = hex!("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").to_vec();

        let input = [versioned_hash, z, y, commitment, proof].concat();

        let expected_output = hex!("000000000000000000000000000000000000000000000000000000000000100073eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");
        let gas = 50000;
        let output = run(&input, gas).unwrap();
        assert_eq!(output.gas_used, gas);
        assert_eq!(output.bytes[..], expected_output);
    }
}
