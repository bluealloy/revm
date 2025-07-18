use context::TxEnv;
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use inspector::count_inspector::CountInspector;
use revm::{
    bytecode::Bytecode,
    primitives::{bytes, hex, Bytes, TxKind},
    Context, ExecuteEvm, InspectEvm, MainBuilder, MainContext,
};

pub fn run(criterion: &mut Criterion) {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .modify_cfg_chained(|c| c.disable_nonce_check = true)
        .build_mainnet()
        .with_inspector(CountInspector::new());

    let tx = TxEnv {
        caller: BENCH_CALLER,
        kind: TxKind::Call(BENCH_TARGET),
        data: bytes!("30627b7c"),
        gas_limit: 1_000_000_000,
        ..Default::default()
    };

    criterion.bench_function("snailtracer", |b| {
        b.iter_batched(
            || {
                // create a transaction input
                tx.clone()
            },
            |input| {
                let _ = evm.transact(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    criterion.bench_function("snailtracer-inspector", |b| {
        b.iter_batched(
            || {
                // create a transaction input
                tx.clone()
            },
            |input| {
                let _ = evm.inspect_tx(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

const BYTES: &str = include_str!("snailtracer.hex");
