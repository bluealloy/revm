use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    handler::EthHandler,
    primitives::{TxKind, U256},
    Context, MainEvm,
};
use std::time::Duration;

pub fn run() {
    let context = Context::default()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_tx_chained(|tx| {
            // execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap();
            tx.value = U256::from(10);
            tx.transact_to = TxKind::Call(
                "0x0000000000000000000000000000000000000000"
                    .parse()
                    .unwrap(),
            );
        });
    let mut evm = MainEvm::new(context, EthHandler::default());

    // Microbenchmark
    let bench_options = microbench::Options::default().time(Duration::from_secs(3));

    microbench::bench(&bench_options, "Simple value transfer", || {
        let _ = evm.transact().unwrap();
    });
}
