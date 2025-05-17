use context::TxEnv;
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{bytes, hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

const BYTES: &str = include_str!("analysis.hex");

pub fn run(criterion: &mut Criterion) {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));
    // BenchmarkDB is dummy state that implements Database trait.
    let context = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .modify_cfg_chained(|c| c.disable_nonce_check = true);
    let tx = TxEnv {
        caller: BENCH_CALLER,
        kind: TxKind::Call(BENCH_TARGET),
        data: bytes!("8035F0CE"),
        ..Default::default()
    };
    let mut evm = context.build_mainnet();
    criterion.bench_function("analysis", |b| {
        b.iter(|| {
            let _ = evm.transact(tx.clone());
        });
    });
}
