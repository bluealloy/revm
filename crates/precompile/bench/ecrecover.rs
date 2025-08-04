//! Benchmarks for the ecrecover precompile
use criterion::{measurement::Measurement, BenchmarkGroup};
use primitives::{hex, keccak256, Bytes};
use revm_precompile::secp256k1::ec_recover_run;
use secp256k1::{Message, SecretKey, SECP256K1};

/// Add benches for the ecrecover precompile
pub fn add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    // Generate secp256k1 signature
    let data = hex::decode("1337133713371337").unwrap();
    let hash = keccak256(data);
    let secret_key = SecretKey::new(&mut secp256k1::rand::rng());

    let message = Message::from_digest_slice(&hash[..]).unwrap();
    let s = SECP256K1.sign_ecdsa_recoverable(message, &secret_key);
    let (rec_id, data) = s.serialize_compact();
    let rec_id = i32::from(rec_id) as u8 + 27;

    let mut message_and_signature = [0u8; 128];
    message_and_signature[0..32].copy_from_slice(&hash[..]);

    // Fit signature into format the precompile expects
    message_and_signature[63] = rec_id;
    message_and_signature[64..128].copy_from_slice(&data);

    let message_and_signature = Bytes::from(message_and_signature);

    group.bench_function("ecrecover precompile", |b| {
        b.iter(|| ec_recover_run(&message_and_signature, u64::MAX).unwrap())
    });
}
