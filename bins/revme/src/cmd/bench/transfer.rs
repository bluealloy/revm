use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{TxKind, U256},
    Context, ExecuteEvm, MainBuilder, MainContext,
};
use std::time::{Duration, Instant};

pub fn run() {
    let time = Instant::now();
    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BENCH_TARGET);
            tx.value = U256::from(10);
        })
        .build_mainnet();
    println!("Init: {:?}", time.elapsed());

    let time = Instant::now();
    let _ = evm.transact_previous();
    println!("First run: {:?}", time.elapsed());

    // Microbenchmark
    let bench_options = microbench::Options::default().time(Duration::from_secs(1));

    microbench::bench(&bench_options, "Run bytecode", || {
        let _ = evm.transact_previous().unwrap();
    });
}
