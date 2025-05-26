//! Benchmarks for the KZG point evaluation precompile
use criterion::{measurement::Measurement, BenchmarkGroup};
use primitives::{eip4844::VERSIONED_HASH_VERSION_KZG, hex};
use revm_precompile::kzg_point_evaluation::run;
use sha2::{Digest, Sha256};

/// Add benches for the KZG point evaluation precompile
pub fn add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    // KZG Point Evaluation Precompile
    let commitment = hex!("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").to_vec();
    let mut versioned_hash = Sha256::digest(&commitment).to_vec();
    versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
    let z = hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").to_vec();
    let y = hex!("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").to_vec();
    let proof = hex!("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").to_vec();

    let kzg_input = [versioned_hash, z, y, commitment, proof].concat();
    let gas = 50000;

    group.bench_function("kzg precompile", |b| {
        b.iter(|| run(&kzg_input, gas).unwrap())
    });
}
