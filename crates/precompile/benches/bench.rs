// `criterion_group!` throws the missing docs error.
#![allow(missing_docs)]
//! Benchmarks for the crypto precompiles
/// `ecrecover` benchmarks
pub mod ecrecover;
/// `eip1962` benchmarks
pub mod eip1962;
/// `eip2537` benchmarks
pub mod eip2537;
/// `eip4844` benchmarks
pub mod eip4844;

use criterion::{criterion_group, criterion_main, Criterion};

/// Benchmarks different cryptography-related precompiles.
pub fn benchmark_crypto_precompiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto Precompile benchmarks");

    // Run BLS12-381 benchmarks (EIP-2537)
    eip2537::add_g1_add_benches(&mut group);
    eip2537::add_g2_add_benches(&mut group);
    eip2537::add_g1_msm_benches(&mut group);
    eip2537::add_g2_msm_benches(&mut group);
    eip2537::add_pairing_benches(&mut group);
    eip2537::add_map_fp_to_g1_benches(&mut group);
    eip2537::add_map_fp2_to_g2_benches(&mut group);

    // Run BN128 benchmarks
    eip1962::add_bn128_add_benches(&mut group);
    eip1962::add_bn128_mul_benches(&mut group);
    eip1962::add_bn128_pair_benches(&mut group);

    // Run secp256k1 benchmarks
    ecrecover::add_benches(&mut group);

    // Run KZG point evaluation benchmarks
    eip4844::add_benches(&mut group);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_crypto_precompiles
}

criterion_main!(benches);
