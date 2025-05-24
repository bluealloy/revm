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

    let tx = TxEnv {
        caller: BENCH_CALLER,
        kind: TxKind::Call(BENCH_TARGET),
        value: U256::from(1),
        gas_price: 0,
        gas_priority_fee: None,
        gas_limit: 30_000,
        ..Default::default()
    };

    let target = U256::from(10000);
    let mut txs = vec![tx.clone(); 1000];

    for (i, tx_mut) in txs.iter_mut().enumerate() {
        tx_mut.kind = TxKind::Call((target + U256::from(i)).into_address());
    }

    criterion.bench_function("transact_commit_1000txs", |b| {
        b.iter(|| {
            for tx in txs.iter() {
                let _ = evm.transact_commit(tx.clone()).unwrap();
            }
        })
    });

    criterion.bench_function("transact_1000tx_commit_inner_every_40", |b| {
        b.iter(|| {
            for (i, tx) in txs.iter().enumerate() {
                let _ = evm.transact(tx.clone()).unwrap();
                if i % 40 == 0 {
                    evm.commit_inner();
                }
            }
            evm.commit_inner();
        })
    });
}
