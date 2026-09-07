//! Fixed-work counterparts of the three flagged Criterion benchmarks for `perf stat`.
//! Usage: account_extension_perf extcodehash|commit|batch ITERATIONS
use revm::{
    context::TxEnv,
    database::{BenchmarkDB, InMemoryDB, BENCH_CALLER, BENCH_TARGET},
    interpreter::instructions::utility::IntoAddress,
    primitives::{hex, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
};
use std::{hint::black_box, time::Instant};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = args.get(1).expect("case: extcodehash, commit, or batch");
    let iterations: usize = args.get(2).expect("iterations").parse().unwrap();
    assert!(iterations > 0);
    if case == "extcodehash" {
        let mut reader = csv::Reader::from_reader(
            include_bytes!("../src/cmd/bench/gas_cost_estimator_sample.csv").as_slice(),
        );
        let record = reader
            .records()
            .map(Result::unwrap)
            .find(|r| &r[0] == "EXTCODEHASH_50")
            .unwrap();
        let code = Bytecode::new_raw(hex::decode(&record[3]).unwrap().into());
        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(code))
            .modify_cfg_chained(|cfg| {
                cfg.disable_nonce_check = true;
                cfg.tx_gas_limit_cap = Some(u64::MAX);
            })
            .build_mainnet();
        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(1_000_000_000)
            .build()
            .unwrap();
        assert!(evm.transact_one(tx.clone()).unwrap().is_success());
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(evm.transact_one(tx.clone()).unwrap());
        }
        println!("{case},{iterations},{}", start.elapsed().as_nanos());
        return;
    }
    assert!(case == "commit" || case == "batch");
    let mut db = InMemoryDB::default();
    for i in 0..10000 {
        db.insert_account_info(
            (U256::from(10000 + i)).into_address(),
            AccountInfo::from_balance(U256::from(3_000_000_000u32)),
        );
    }
    for address in [BENCH_CALLER, BENCH_TARGET] {
        db.insert_account_info(
            address,
            AccountInfo::from_balance(U256::from(3_000_000_000u32)),
        );
    }
    let mut evm = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.disable_nonce_check = true)
        .build_mainnet();
    let txs: Vec<_> = (0..1000)
        .map(|i| {
            TxEnv::builder()
                .caller(BENCH_CALLER)
                .kind(TxKind::Call(U256::from(10000 + i).into_address()))
                .value(U256::from(1))
                .gas_price(0)
                .gas_priority_fee(None)
                .gas_limit(30_000)
                .build()
                .unwrap()
        })
        .collect();
    assert!(evm.transact_commit(txs[0].clone()).unwrap().is_success());
    let start = Instant::now();
    for _ in 0..iterations {
        for (i, tx) in txs.iter().enumerate() {
            if case == "commit" {
                black_box(evm.transact_commit(tx.clone()).unwrap());
            } else {
                black_box(evm.transact_one(tx.clone()).unwrap());
                // Match the existing benchmark's boundary, including its first-tx commit.
                if i.is_multiple_of(40) {
                    evm.commit_inner();
                }
            }
        }
        if case == "batch" {
            evm.commit_inner();
        }
    }
    println!("{case},{iterations},{}", start.elapsed().as_nanos());
}
