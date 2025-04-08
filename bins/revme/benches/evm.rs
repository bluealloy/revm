use criterion::{criterion_group, criterion_main, Criterion};
use revme::cmd::bench;

fn evm(c: &mut Criterion) {
    bench::analysis::run(c);
    bench::burntpix::run(c);
    bench::snailtracer::run(c);
    bench::transfer::run(c);
    bench::evm_build::run(c);
}
criterion_group!(benches, evm);
criterion_main!(benches);
