use criterion::Criterion;
use revm::{
    bytecode::Bytecode,
    context::TxEnv,
    context_interface::result::{EVMError, InvalidTransaction},
    database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET},
    database_interface::Database,
    handler::{Handler, MainnetHandler},
    primitives::{hex, Bytes, TxKind},
    Context, MainBuilder, MainContext,
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
            .modify_cfg_chained(|c| {
                c.disable_nonce_check = true;
                c.tx_gas_limit_cap = Some(u64::MAX);
            })
            .build_mainnet();

        let tx = TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(BENCH_TARGET))
            .gas_limit(1_000_000_000)
            .build()
            .unwrap();
        evm.ctx.tx = tx;

        type BenchError = EVMError<<BenchmarkDB as Database>::Error, InvalidTransaction>;
        let mut handler: MainnetHandler<_, BenchError, _> = MainnetHandler::default();

        criterion.bench_function(name, |b| {
            b.iter(|| handler.run(&mut evm).unwrap());
        });
    }
}

const SAMPLE_CSV: &str = include_str!("gas_cost_estimator_sample.csv");
