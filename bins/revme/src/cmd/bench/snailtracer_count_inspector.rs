use context::TxEnv;
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    inspector::{inspectors::CountInspector, InspectEvm},
    primitives::{bytes, hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

pub fn run(criterion: &mut Criterion) {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    let mut group = criterion.benchmark_group("snailtracer_inspector_comparison");

    // Benchmark with CountInspector
    group.bench_function("with_count_inspector", |b| {
        b.iter_batched(
            || {
                // Create a fresh CountInspector and transaction for each iteration
                let count_inspector = CountInspector::new();

                // Create transaction
                let tx = TxEnv::builder()
                    .caller(BENCH_CALLER)
                    .kind(TxKind::Call(BENCH_TARGET))
                    .data(bytes!("30627b7c"))
                    .gas_limit(1_000_000_000)
                    .build()
                    .unwrap();

                (count_inspector, tx)
            },
            |(mut count_inspector, tx)| {
                // Setup EVM with CountInspector
                let mut evm = Context::mainnet()
                    .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
                    .modify_cfg_chained(|c| c.disable_nonce_check = true)
                    .build_mainnet_with_inspector(&mut count_inspector);

                // Execute transaction with inspector
                let _ = evm.inspect_one_tx(tx).unwrap();

                // Return the count_inspector to prevent it from being optimized away
                count_inspector
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Benchmark without inspector (baseline)
    group.bench_function("no_inspector", |b| {
        b.iter_batched(
            || {
                // Create transaction
                TxEnv::builder()
                    .caller(BENCH_CALLER)
                    .kind(TxKind::Call(BENCH_TARGET))
                    .data(bytes!("30627b7c"))
                    .gas_limit(1_000_000_000)
                    .build()
                    .unwrap()
            },
            |tx| {
                // Setup EVM without inspector
                let mut evm = Context::mainnet()
                    .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
                    .modify_cfg_chained(|c| c.disable_nonce_check = true)
                    .build_mainnet();

                // Execute transaction without inspector
                let _ = evm.transact_one(tx).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

const BYTES: &str = include_str!("snailtracer.hex");
