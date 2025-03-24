use criterion::{criterion_group, criterion_main, Criterion};
use revme::cmd::bench;

fn evm(c: &mut Criterion) {
    bench::analysis::run(&mut c.benchmark_group("revme"));
    bench::burntpix::run(&mut c.benchmark_group("revme"));
    bench::snailtracer::run(&mut c.benchmark_group("revme"));
    bench::transfer::run(&mut c.benchmark_group("revme"));
}
criterion_group!(benches, evm);
criterion_main!(benches);
