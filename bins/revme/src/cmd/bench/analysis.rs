use std::time::Instant;

use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{bytes, hex, Bytes, TxKind},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

const BYTES: &str = include_str!("analysis.hex");

pub fn run() {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    // BenchmarkDB is dummy state that implements Database trait.
    let context = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BENCH_TARGET);
            tx.data = bytes!("8035F0CE");
        });

    let mut evm = context.build_mainnet();

    let time = Instant::now();
    let _ = evm.replay();
    println!("First init: {:?}", time.elapsed());

    let time = Instant::now();
    let _ = evm.replay();
    println!("Run: {:?}", time.elapsed());
}
