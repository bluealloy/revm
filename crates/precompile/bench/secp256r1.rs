//! Benchmarks for the secp256r1 (P256) precompile
use criterion::{measurement::Measurement, BenchmarkGroup};
use p256::ecdsa::{signature::hazmat::PrehashSigner, SigningKey};
use primitives::Bytes;
use revm_precompile::secp256r1::p256_verify;

/// Add benches for the secp256r1 precompile
pub fn add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    // Generate a valid P256 signature using p256's own OsRng (rand_core 0.6)
    let signing_key = SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let verifying_key = signing_key.verifying_key();

    let msg_hash = [0xabu8; 32];

    let (signature, _) = signing_key.sign_prehash(&msg_hash).unwrap();
    let sig_bytes = signature.to_bytes();

    let pk_encoded = verifying_key.to_encoded_point(false);
    // Uncompressed point is 0x04 || x || y, skip the 0x04 prefix
    let pk_bytes = &pk_encoded.as_bytes()[1..];

    // Input layout: msg_hash (32) || r (32) || s (32) || pubkey_x (32) || pubkey_y (32)
    let mut input = Vec::with_capacity(160);
    input.extend_from_slice(&msg_hash);
    input.extend_from_slice(&sig_bytes);
    input.extend_from_slice(pk_bytes);

    let input = Bytes::from(input);

    group.bench_function("p256verify precompile", |b| {
        b.iter(|| p256_verify(&input, u64::MAX).unwrap())
    });
}
