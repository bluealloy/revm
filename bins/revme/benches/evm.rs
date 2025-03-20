use criterion::{criterion_group, criterion_main, Criterion};
use revme::cmd::{
    bench::{self, BenchName},
    MainCmd,
};

fn evm(c: &mut Criterion) {
    // call analysis to init static data.
    revme::cmd::bench::analysis::run();

    for &bench_name in BenchName::ALL {
        let cmd = MainCmd::Bench(bench::Cmd {
            name: bench_name,
            warmup: 10.0,
            measurement_time: 5.0,
        });
        c.bench_function(bench_name.as_str(), |b| {
            b.iter(|| cmd.run().unwrap());
        });
    }
}

criterion_group!(benches, evm);
criterion_main!(benches);
