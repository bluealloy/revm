use criterion::Criterion;

use revm::{
    Context, ExecuteEvm, InspectEvm, MainBuilder, MainContext, bytecode::Bytecode, context::TxEnv, database::{BENCH_CALLER, BENCH_TARGET, BenchmarkDB}, inspector::NoOpInspector, primitives::{Bytes, TxKind, bytes, eip7825, hex}
};

pub fn run(criterion: &mut Criterion) {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .modify_cfg_chained(|c| c.disable_nonce_check = true)
        .build_mainnet()
        .with_inspector(NoOpInspector {});

    let tx = TxEnv::builder()
        .caller(BENCH_CALLER)
        .kind(TxKind::Call(BENCH_TARGET))
        .data(bytes!("30627b7c"))
        .gas_limit(eip7825::TX_GAS_LIMIT_CAP)
        .build()
        .unwrap();

    criterion.bench_function("snailtracer", |b| {
        b.iter_batched(
            || tx.clone(),
            |input| evm.transact_one(input).unwrap(),
            criterion::BatchSize::SmallInput,
        );
    });

    criterion.bench_function("snailtracer-inspect", |b| {
        b.iter_batched(
            || tx.clone(),
            |input| evm.inspect_one_tx(input),
            criterion::BatchSize::SmallInput,
        );
    });
}

const BYTES: &str = include_str!("snailtracer.hex");
