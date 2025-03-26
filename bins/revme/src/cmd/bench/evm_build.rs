use criterion::Criterion;
use revm::{Context, ExecuteEvm, MainBuilder, MainContext};

pub fn run(criterion: &mut Criterion) {
    let mut evm = Context::mainnet().build_mainnet();

    criterion.bench_function("evm_build", |b| {
        b.iter(|| {
            let _ = evm.replay().unwrap();
        });
    });
}
