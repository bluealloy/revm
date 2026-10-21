//! KZG point evaluation precompile using Arkworks BLS12-381 implementation.
use crate::{bls12_381_const::TRUSTED_SETUP_TAU_G2_BYTES, PrecompileHalt};
use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::{BigInteger, One, PrimeField};
use ark_serialize::CanonicalDeserialize;
use core::ops::Neg;
use primitives::OnceLock;

/// Miller-loop-prepared G2 point (the `Pairing` associated type; not a root export).
type G2Prepared = <Bls12_381 as Pairing>::G2Prepared;

/// Verify KZG proof using BLS12-381 implementation.
///
/// <https://github.com/ethereum/EIPs/blob/4d2a00692bb131366ede1a16eced2b0e25b1bf99/EIPS/eip-4844.md?plain=1#L203>
/// <https://github.com/ethereum/consensus-specs/blob/master/specs/deneb/polynomial-commitments.md#verify_kzg_proof_impl>
#[inline]
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    // Parse the commitment and proof (G1 points)
    let Ok(commitment_point) = parse_g1_compressed(commitment) else {
        return false;
    };
    let Ok(proof_point) = parse_g1_compressed(proof) else {
        return false;
    };

    // Parse z and y as canonical scalar field elements (Fr)
    let Ok(z_fr) = read_scalar_canonical(z) else {
        return false;
    };
    let Ok(y_fr) = read_scalar_canonical(y) else {
        return false;
    };

    // Reformulated single-proof KZG check. Starting from the standard
    //   e(commitment - [y]G1, -G2) · e(proof, [τ]G2 - [z]G2) == 1
    // and applying bilinearity `e(proof, -[z]G2) = e([z]·proof, -G2)` to move z
    // out of G2, both G2 pairing inputs collapse to the compile-time constants
    // -G2 and [τ]G2:
    //   e(D, -G2) · e(proof, [τ]G2) == 1,   D = commitment - [y]G1 + [z]·proof
    // This trades the per-call G2 scalar multiplication for a cheaper G1 one and
    // lets the two constant G2 points be prepared for the Miller loop just once.
    let g1 = G1Affine::generator();
    let y_g1 = g1.mul_bigint(y_fr.into_bigint());
    let z_proof = proof_point.mul_bigint(z_fr.into_bigint());
    // Accumulate in projective coordinates and normalize to affine exactly once.
    let d = (commitment_point.into_group() - y_g1 + z_proof).into_affine();

    // `multi_miller_loop` drops any pair whose G1 or G2 point is the identity, so
    // an infinity `d` or `proof` is handled here (the two G2 points are never
    // infinity). One shared final exponentiation covers both pairs.
    let mll = Bls12_381::multi_miller_loop(
        [d, proof_point],
        [neg_g2_prepared().clone(), tau_g2_prepared().clone()],
    );
    Bls12_381::final_exponentiation(mll)
        .map(|out| out.0.is_one())
        .unwrap_or(false)
}

/// Miller-loop preparation of the constant `-G2` generator, computed once.
fn neg_g2_prepared() -> &'static G2Prepared {
    static NEG_G2: OnceLock<G2Prepared> = OnceLock::new();
    NEG_G2.get_or_init(|| G2Prepared::from(G2Affine::generator().neg()))
}

/// Miller-loop preparation of the trusted-setup point `[τ]G2`, computed once.
fn tau_g2_prepared() -> &'static G2Prepared {
    static TAU_G2: OnceLock<G2Prepared> = OnceLock::new();
    TAU_G2.get_or_init(|| G2Prepared::from(*get_trusted_setup_g2()))
}

/// Get the trusted setup G2 point `[τ]₂` from the Ethereum KZG ceremony.
/// This is g2_monomial_1 from trusted_setup_4096.json
fn get_trusted_setup_g2() -> &'static G2Affine {
    static TAU_G2: OnceLock<G2Affine> = OnceLock::new();
    TAU_G2.get_or_init(|| {
        // Parse the compressed G2 point using unchecked deserialization since we trust this point.
        // This should never fail since we're using a known valid point from the trusted setup.
        G2Affine::deserialize_compressed_unchecked(&TRUSTED_SETUP_TAU_G2_BYTES[..])
            .expect("Failed to parse trusted setup G2 point")
    })
}

/// Parse a G1 point from compressed format (48 bytes)
fn parse_g1_compressed(bytes: &[u8; 48]) -> Result<G1Affine, PrecompileHalt> {
    G1Affine::deserialize_compressed(&bytes[..]).map_err(|_| PrecompileHalt::KzgInvalidG1Point)
}

/// Read a scalar field element from bytes and verify it's canonical
fn read_scalar_canonical(bytes: &[u8; 32]) -> Result<Fr, PrecompileHalt> {
    let fr = Fr::from_be_bytes_mod_order(bytes);

    // Check if the field element is canonical by serializing back and comparing
    let bytes_roundtrip = fr.into_bigint().to_bytes_be();

    if bytes_roundtrip.as_slice() != bytes {
        return Err(PrecompileHalt::NonCanonicalFp);
    }

    Ok(fr)
}
