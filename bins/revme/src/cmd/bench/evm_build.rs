use criterion::Criterion;
use revm::{Context, MainBuilder, MainContext};

pub fn run(criterion: &mut Criterion) {
    criterion.bench_function("evm-build", |b| {
        b.iter(|| {
            let _ = Context::mainnet().build_mainnet();
        });
    });
}
