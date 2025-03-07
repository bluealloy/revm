use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{bytes, hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

pub fn simple_example(bytecode: Bytecode) {
    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BENCH_TARGET);
            tx.data = bytes!("30627b7c");
            tx.gas_limit = 1_000_000_000;
        })
        .build_mainnet();
    let _ = evm.replay().unwrap();
}

pub fn run() {
    println!("Running snailtracer example!");
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));
    let start = std::time::Instant::now();
    simple_example(bytecode);
    let elapsed = start.elapsed();
    println!("elapsed: {:?}", elapsed);
}

const BYTES: &str = include_str!("snailtracer.hex");
