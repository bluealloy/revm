//! Benchmarks for the BN128 precompiles
use criterion::{measurement::Measurement, BenchmarkGroup};
use primitives::hex;
use primitives::Bytes;
use revm_precompile::bn128::{
    add::ISTANBUL_ADD_GAS_COST,
    mul::ISTANBUL_MUL_GAS_COST,
    pair::{ISTANBUL_PAIR_BASE, ISTANBUL_PAIR_PER_POINT},
    run_add, run_mul, run_pair,
};

/// Add benches for the BN128 add precompile
pub fn add_bn128_add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let ecadd_input = hex::decode(
        "\
         18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9\
         063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266\
         07c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed\
         06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7",
    )
    .unwrap();
    let input = Bytes::from(ecadd_input);

    group.bench_function("bn128 add precompile", |b| {
        b.iter(|| run_add(&input, ISTANBUL_ADD_GAS_COST, 150).unwrap())
    });
}

/// Add benches for the BN128 mul precompile
pub fn add_bn128_mul_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let ecmul_input = hex::decode(
        "\
         18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9\
         063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266\
         06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7",
    )
    .unwrap();
    let input = Bytes::from(ecmul_input);

    group.bench_function("bn128 mul precompile", |b| {
        b.iter(|| run_mul(&input, ISTANBUL_MUL_GAS_COST, 6000).unwrap())
    });
}

/// Add benches for the BN128 pair precompile
pub fn add_bn128_pair_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let ecpair_input = hex::decode(
        "\
        1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
        3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
        209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
        04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
        2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
        120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
        111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
        2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
        198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
        1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
        090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
        12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
    )
    .unwrap();
    let input = Bytes::from(ecpair_input);

    group.bench_function("ecpairing precompile", |b| {
        b.iter(|| {
            run_pair(
                &input,
                ISTANBUL_PAIR_PER_POINT,
                ISTANBUL_PAIR_BASE,
                u64::MAX,
            )
            .unwrap()
        })
    });
}
