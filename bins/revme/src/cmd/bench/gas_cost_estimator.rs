use criterion::Criterion;
use revm::{
    bytecode::Bytecode,
    context::TxEnv,
    database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET},
    primitives::{eip7825, hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::io::Cursor;

pub fn run(criterion: &mut Criterion) {
    //let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    let mut rdr = csv::Reader::from_reader(Cursor::new(SAMPLE_CSV));
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

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(eip7825::TX_GAS_LIMIT_CAP)
            .build()
            .unwrap();

        criterion.bench_function(name, |b| {
            b.iter_batched(
                || tx.clone(),
                |input| evm.transact_one(input).unwrap(),
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

const SAMPLE_CSV: &str = include_str!("gas_cost_estimator_sample.csv");
