use context::TxEnv;
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::io::Cursor;

pub fn run(criterion: &mut Criterion) {
    //let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    let mut rdr = csv::Reader::from_reader(Cursor::new(BYTES));
    for result in rdr.records() {
        let result = result.expect("Failed to read record");
        let name = &result[0];
        let bytecode_hex = &result[3];
        let Ok(hex) = hex::decode(bytecode_hex) else {
            continue;
        };
        let bytecode = Bytecode::new_raw(Bytes::from(hex));

        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
            .modify_cfg_chained(|c| c.disable_nonce_check = true)
            .build_mainnet();

        let tx = TxEnv {
            caller: BENCH_CALLER,
            kind: TxKind::Call(BENCH_TARGET),
            gas_limit: 1_000_000_000,
            ..Default::default()
        };

        criterion.bench_function(name, |b| {
            b.iter_batched(
                || {
                    // create a transaction input
                    tx.clone()
                },
                |input| {
                    let _ = evm.transact_one(input).unwrap();
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

const BYTES: &str = include_str!("gas_cost_estimator_sample.hex");
