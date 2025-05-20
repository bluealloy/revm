use context::{
    result::{EVMError, InvalidTransaction},
    ContextTr, TxEnv,
};
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET, BENCH_TARGET_BALANCE};
use revm::{
    bytecode::Bytecode,
    context_interface::JournalTr,
    handler::{EthFrame, Handler, MainnetHandler},
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

    let mut i = 0;
    criterion.bench_function("transfer", |b| {
        b.iter(|| {
            i += 1;
            let _ = evm.transact(tx.clone()).unwrap();
            // caller, target and beneficiary stay loaded inside journal so they
            // are not loaded again.
        })
    });

    let balance = evm
        .journal()
        .load_account(BENCH_TARGET)
        .unwrap()
        .data
        .info
        .balance;

    if balance != BENCH_TARGET_BALANCE + U256::from(i) {
        panic!("balance of transfers is not correct");
    }

    // drop the journal
    let _ = evm.finalize();

    criterion.bench_function("transfer_finalize", |b| {
        b.iter(|| {
            let mut t =
                MainnetHandler::<_, EVMError<_, InvalidTransaction>, EthFrame<_, _, _>>::default();
            t.run(&mut evm).unwrap();
            evm.journal().finalize();
        })
    });
}
