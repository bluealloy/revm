use criterion::Criterion;
use revm_precompile::{
    primitives::SHORT_ADDRESS_CAP, u64_to_address, Precompile, PrecompileId, PrecompileResult,
    Precompiles,
};
use std::hint::black_box;

fn unused_precompile(_: &[u8], _: u64, _: u64) -> PrecompileResult {
    unreachable!("the lookup benchmark never executes the precompile")
}

pub fn benchmark_precompile_lookup(c: &mut Criterion) {
    let short_hit = u64_to_address(4);
    let short_miss = u64_to_address(SHORT_ADDRESS_CAP as u64 - 1);
    let long_hit = u64_to_address(SHORT_ADDRESS_CAP as u64);
    let long_miss = u64_to_address(SHORT_ADDRESS_CAP as u64 + 1);

    let mut precompiles = Precompiles::latest().clone();
    precompiles.extend([Precompile::new(
        PrecompileId::custom("lookup-benchmark"),
        long_hit,
        unused_precompile,
    )]);

    let mut invalidated = precompiles.clone();
    assert!(invalidated.get_mut(&short_hit).is_some());

    assert!(precompiles.get(&short_hit).is_some());
    assert!(precompiles.get(&short_miss).is_none());
    assert!(invalidated.get(&short_hit).is_some());
    assert!(precompiles.get(&long_hit).is_some());
    assert!(precompiles.get(&long_miss).is_none());

    let cases = [
        ("short_hit", &precompiles, short_hit),
        ("short_miss", &precompiles, short_miss),
        ("invalidated_short_hit", &invalidated, short_hit),
        ("long_hit", &precompiles, long_hit),
        ("long_miss", &precompiles, long_miss),
    ];

    let mut group = c.benchmark_group("precompile_lookup");
    for (name, precompiles, address) in cases {
        group.bench_function(name, |b| {
            b.iter(|| black_box(black_box(precompiles).get(black_box(&address))))
        });
    }
    group.finish();
}
