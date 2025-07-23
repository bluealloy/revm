//! KZG point evaluation precompile added in [`EIP-4844`](https://eips.ethereum.org/EIPS/eip-4844)
//! For more details check [`run`] function.
use crate::{Address, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        use c_kzg::{Bytes32, Bytes48};
    } else if #[cfg(feature = "kzg-rs")] {
        use kzg_rs::{Bytes32, Bytes48, KzgProof};
    } else {
        // These are not needed here, but as_bytes_48 and as_bytes32
        // won't compile without them
        pub type Bytes32 = [u8;32];
        pub type Bytes48 = [u8;48];
    }
}
use primitives::hex_literal::hex;
use sha2::{Digest, Sha256};

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
pub fn run(input: &[u8], gas_limit: u64, _crypto: &dyn crate::Crypto) -> PrecompileResult {
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
    let commitment: &[u8; 48] = commitment.try_into().unwrap();
    let z = input[32..64].try_into().unwrap();
    let y = input[64..96].try_into().unwrap();
    let proof = input[144..192].try_into().unwrap();
    if !verify_kzg_proof(commitment, z, y, proof) {
        return Err(PrecompileError::BlobVerifyKzgProofFailed);
    }

    // Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values
    Ok(PrecompileOutput::new(GAS_COST, RETURN_VALUE.into()))
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
#[inline]
pub fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
    let mut hash: [u8; 32] = Sha256::digest(commitment).into();
    hash[0] = VERSIONED_HASH_VERSION_KZG;
    hash
}

/// Verify KZG proof.
#[inline]
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    cfg_if::cfg_if! {
        if #[cfg(feature = "c-kzg")] {
            let kzg_settings = c_kzg::ethereum_kzg_settings(8);
            kzg_settings.verify_kzg_proof(as_bytes48(commitment), as_bytes32(z), as_bytes32(y), as_bytes48(proof)).unwrap_or(false)
        } else if #[cfg(feature = "kzg-rs")] {
            let env = kzg_rs::EnvKzgSettings::default();
            let kzg_settings = env.get();
            KzgProof::verify_kzg_proof(as_bytes48(commitment), as_bytes32(z), as_bytes32(y), as_bytes48(proof), kzg_settings).unwrap_or(false)
        } else {
            bls12_381_backend::verify_kzg_proof(commitment, z, y, proof)
        }
    }
}

/// BLS12-381 backend implementation for KZG verification
#[cfg(not(any(feature = "c-kzg", feature = "kzg-rs")))]
mod bls12_381_backend {
    use super::*;
    use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine};
    use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
    use ark_ff::{BigInteger, One, PrimeField};
    use ark_serialize::CanonicalDeserialize;
    use core::ops::Neg;
    use std::string::ToString;

    /// Verify KZG proof using BLS12-381 implementation.
    ///
    /// https://github.com/ethereum/EIPs/blob/4d2a00692bb131366ede1a16eced2b0e25b1bf99/EIPS/eip-4844.md?plain=1#L203
    /// https://github.com/ethereum/consensus-specs/blob/master/specs/deneb/polynomial-commitments.md#verify_kzg_proof_impl
    #[inline]
    pub(super) fn verify_kzg_proof(
        commitment: &Bytes48,
        z: &Bytes32,
        y: &Bytes32,
        proof: &Bytes48,
    ) -> bool {
        // Parse the commitment (G1 point)
        let Ok(commitment_point) = parse_g1_compressed(&commitment) else {
            return false;
        };

        // Parse the proof (G1 point)
        let Ok(proof_point) = parse_g1_compressed(&proof) else {
            return false;
        };

        // Parse z and y as field elements (Fr, scalar field)
        // We expect 32-byte big-endian scalars that must be canonical
        let Ok(z_fr) = read_scalar_canonical(&z) else {
            return false;
        };
        let Ok(y_fr) = read_scalar_canonical(&y) else {
            return false;
        };

        // Get the trusted setup G2 point [τ]₂
        // TODO: This only needs to be done once and cached
        let tau_g2 = get_trusted_setup_g2();

        // Verify KZG proof that p(z) == y where p(z) is the polynomial represented by the commitment
        // Following the reference implementation from the consensus specs

        // Get generators
        let g1 = G1Affine::generator();
        let g2 = G2Affine::generator();

        // Compute P_minus_y = commitment - [y]G₁
        let minus_y = y_fr.neg();
        let minus_y_g1 = g1.mul_bigint(minus_y.into_bigint()).into_affine();
        let p_minus_y = (commitment_point.into_group() + minus_y_g1.into_group()).into_affine();

        // Compute X_minus_z = [τ]G₂ - [z]G₂
        let minus_z = z_fr.neg();
        let minus_z_g2 = g2.mul_bigint(minus_z.into_bigint()).into_affine();
        let x_minus_z = (tau_g2.into_group() + minus_z_g2.into_group()).into_affine();

        // Verify: P - y = Q * (X - z)
        // Using pairing_check([[P_minus_y, -G₂], [proof, X_minus_z]]) == 1
        let neg_g2 = g2.neg();

        let g1_points = [p_minus_y, proof_point];
        let g2_points = [neg_g2, x_minus_z];

        let pairing_result = Bls12_381::multi_pairing(&g1_points, &g2_points);
        pairing_result.0.is_one()
    }

    /// Get the trusted setup G2 point [τ]₂ from the Ethereum KZG ceremony.
    /// This is g2_monomial_1 from trusted_setup_4096.json
    fn get_trusted_setup_g2() -> G2Affine {
        // The trusted setup G2 point [τ]₂ from the Ethereum KZG ceremony (compressed format)
        // Taken from: https://github.com/ethereum/consensus-specs/blob/adc514a1c29532ebc1a67c71dc8741a2fdac5ed4/presets/mainnet/trusted_setups/trusted_setup_4096.json#L8200C6-L8200C200
        const TRUSTED_SETUP_TAU_G2_BYTES: &[u8; 96] = &hex!(
            "b5bfd7dd8cdeb128843bc287230af38926187075cbfbefa81009a2ce615ac53d2914e5870cb452d2afaaab24f3499f72185cbfee53492714734429b7b38608e23926c911cceceac9a36851477ba4c60b087041de621000edc98edada20c1def2"
        );

        // Parse the compressed G2 point using unchecked deserialization since we trust this point
        // This should never fail since we're using a known valid point from the trusted setup
        G2Affine::deserialize_compressed_unchecked(&TRUSTED_SETUP_TAU_G2_BYTES[..])
            .expect("Failed to parse trusted setup G2 point")
    }

    /// Parse a G1 point from compressed format (48 bytes)
    fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<G1Affine, PrecompileError> {
        G1Affine::deserialize_compressed(&bytes[..])
            .map_err(|_| PrecompileError::Other("Invalid compressed G1 point".to_string()))
    }

    /// Read a scalar field element from bytes and verify it's canonical
    fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<Fr, PrecompileError> {
        let fr = Fr::from_be_bytes_mod_order(bytes);

        // Check if the field element is canonical by serializing back and comparing
        let bytes_roundtrip = fr.into_bigint().to_bytes_be();

        if bytes_roundtrip.as_slice() != bytes {
            return Err(PrecompileError::Other(
                "Non-canonical scalar field element".to_string(),
            ));
        }

        Ok(fr)
    }
}

/// Convert a slice to an array of a specific size.
#[inline]
#[track_caller]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    bytes.try_into().expect("slice with incorrect length")
}

/// Convert a slice to a 32 byte big endian array.
#[inline]
#[track_caller]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

/// Convert a slice to a 48 byte big endian array.
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
        // Test data from: https://github.com/ethereum/c-kzg-4844/blob/main/tests/verify_kzg_proof/kzg-mainnet/verify_kzg_proof_case_correct_proof_4_4/data.yaml

        let commitment = hex!("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").to_vec();
        let mut versioned_hash = Sha256::digest(&commitment).to_vec();
        versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
        let z = hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").to_vec();
        let y = hex!("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").to_vec();
        let proof = hex!("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").to_vec();

        let input = [versioned_hash, z, y, commitment, proof].concat();

        let expected_output = hex!("000000000000000000000000000000000000000000000000000000000000100073eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");
        let gas = 50000;
        let output = run(&input, gas, &crate::DefaultCrypto).unwrap();
        assert_eq!(output.gas_used, gas);
        assert_eq!(output.bytes[..], expected_output);
    }
}
