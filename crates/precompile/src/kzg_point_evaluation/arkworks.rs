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
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
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
    let tau_g2 = get_trusted_setup_g2();

    // Get generators
    let g1 = G1Affine::generator();
    let g2 = G2Affine::generator();

    // Compute P_minus_y = commitment - [y]G₁
    let minus_y = y_fr.neg();
    let minus_y_g1 = g1.mul_bigint(minus_y.into_bigint());
    let p_minus_y = (commitment_point.into_group() + minus_y_g1).into_affine();

    // Compute X_minus_z = [τ]G₂ - [z]G₂
    let minus_z = z_fr.neg();
    let minus_z_g2 = g2.mul_bigint(minus_z.into_bigint());
    let x_minus_z = (tau_g2.into_group() + minus_z_g2).into_affine();

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
