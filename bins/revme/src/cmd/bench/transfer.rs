use std::time::Instant;

use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{TxKind, U256},
    Context, ExecuteEvm,
};

pub fn run() {
    let mut context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BENCH_TARGET);
            tx.value = U256::from(10);
        });
    let time = Instant::now();
    let _ = context.exec_previous();
    println!("First init: {:?}", time.elapsed());

    let time = Instant::now();
    let _ = context.exec_previous();
    println!("Run: {:?}", time.elapsed());
}
