use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
use revm::{
    bytecode::Bytecode,
    primitives::{bytes, hex, Bytes, TxKind},
    transact_main, Context,
};

const BYTES: &str = include_str!("analysis.hex");

pub fn run() {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    // BenchmarkDB is dummy state that implements Database trait.
    let mut context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BENCH_TARGET);
            //evm.env.tx.data = Bytes::from(hex::decode("30627b7c").unwrap());
            tx.data = bytes!("8035F0CE");
        });
    let _ = transact_main(&mut context);
}
