use criterion::{criterion_group, criterion_main, Criterion};
use primitives::{hardfork::SpecId, U256};
use revm_interpreter::{
    host::DummyHost,
    instructions::{
        arithmetic::mul,
        bitwise::{clz, iszero},
    },
    InstructionContext, Interpreter,
};
use std::time::{Duration, Instant};

fn bench_isolated_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("isolated_operations");

    let test_value = U256::from(0x1_u128);

    group.bench_function("iszero_only", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                // Setup (not timed)
                let mut interpreter = Interpreter::default();
                let _ = interpreter.stack.push(test_value);
                let mut host = DummyHost;
                let context = InstructionContext {
                    host: &mut host,
                    interpreter: &mut interpreter,
                };

                let start = Instant::now();
                iszero(context);
                total_duration += start.elapsed();
            }

            total_duration
        });
    });

    group.bench_function("clz_only", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                // Setup (not timed)
                let mut interpreter = Interpreter::default();
                interpreter.set_spec_id(SpecId::OSAKA);
                let _ = interpreter.stack.push(test_value);
                let mut host = DummyHost;
                let context = InstructionContext {
                    host: &mut host,
                    interpreter: &mut interpreter,
                };

                let start = Instant::now();
                clz(context);
                total_duration += start.elapsed();
            }

            total_duration
        });
    });

    group.bench_function("mul_only", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                // Setup (not timed)
                let mut interpreter = Interpreter::default();
                let _ = interpreter.stack.push(test_value);
                let _ = interpreter.stack.push(test_value);
                let mut host = DummyHost;
                let context = InstructionContext {
                    host: &mut host,
                    interpreter: &mut interpreter,
                };

                let start = Instant::now();
                mul(context);
                total_duration += start.elapsed();
            }

            total_duration
        });
    });

    group.finish();
}

criterion_group!(benches, bench_isolated_operations);
criterion_main!(benches);
