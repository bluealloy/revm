use context::{ContextTr, TxEnv};
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{TxKind, U256},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

pub fn run(criterion: &mut Criterion) {
    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_cfg_chained(|cfg| cfg.disable_nonce_check = true)
        .build_mainnet();

    let tx = TxEnv {
        caller: BENCH_CALLER,
        kind: TxKind::Call(BENCH_TARGET),
        value: U256::from(1),
        gas_price: 1,
        gas_priority_fee: None,
        ..Default::default()
    };

    criterion.bench_function("transfer", |b| {
        b.iter(|| {
            let _ = evm.transact(tx.clone()).unwrap();
            // clear caller and target, beneficiary stays the same.
            // this effect the the benchmark results.
            evm.journal().state.remove(&BENCH_CALLER);
            evm.journal().state.remove(&BENCH_TARGET);
        })
    });

    criterion.bench_function("transfer_finalize", |b| {
        b.iter(|| {
            let _ = evm.transact_finalize(tx.clone()).unwrap();
        })
    });
}
