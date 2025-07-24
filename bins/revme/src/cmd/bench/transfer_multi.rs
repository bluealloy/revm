use context::TxEnv;
use criterion::Criterion;
use database::{InMemoryDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    interpreter::instructions::utility::IntoAddress,
    primitives::{TxKind, U256},
    Context, ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
};
use state::AccountInfo;

pub fn run(criterion: &mut Criterion) {
    let mut db = InMemoryDB::default();

    let address = U256::from(10000);
    for i in 0..10000 {
        db.insert_account_info(
            (address + U256::from(i)).into_address(),
            AccountInfo::from_balance(U256::from(3_000_000_000u32)),
        );
    }
    db.insert_account_info(
        BENCH_TARGET,
        AccountInfo::from_balance(U256::from(3_000_000_000u32)),
    );

    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(3_000_000_000u32)),
    );

    let mut evm = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.disable_nonce_check = true)
        .build_mainnet();

    let target = U256::from(10000);
    let mut txs = Vec::with_capacity(1000);

    for i in 0..1000 {
        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call((target + U256::from(i)).into_address()))
            .value(U256::from(1))
            .gas_price(0)
            .gas_priority_fee(None)
            .gas_limit(30_000)
            .build()
            .unwrap();
        txs.push(tx);
    }

    criterion.bench_function("transact_commit_1000txs", |b| {
        b.iter_batched(
            || {
                // create transaction inputs
                txs.clone()
            },
            |inputs| {
                for tx in inputs {
                    let _ = evm.transact_commit(tx).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    criterion.bench_function("transact_1000tx_commit_inner_every_40", |b| {
        b.iter_batched(
            || {
                // create transaction inputs
                txs.clone()
            },
            |inputs| {
                for (i, tx) in inputs.into_iter().enumerate() {
                    let _ = evm.transact_one(tx).unwrap();
                    if i.is_multiple_of(40) {
                        evm.commit_inner();
                    }
                }
                evm.commit_inner();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}
