use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    handler::EthHandler,
    primitives::{address, bytes, hex, Bytes, TxKind},
    Context, MainEvm,
};
use std::time::Instant;

const BYTES: &'static str = include_str!("analysis.hex");

pub fn run() {
    let bytecode = Bytecode::new_raw(Bytes::from(hex::decode(BYTES).unwrap()));

    // BenchmarkDB is dummy state that implements Database trait.
    let context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(bytecode))
        .modify_tx_chained(|tx| {
            // execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.transact_to = TxKind::Call(address!("0000000000000000000000000000000000000000"));
            //evm.env.tx.data = Bytes::from(hex::decode("30627b7c").unwrap());
            tx.data = bytes!("8035F0CE");
        });
    let mut evm = MainEvm::new(context, EthHandler::default());

    // Just to warm up the processor.
    for _ in 0..10000 {
        let _ = evm.transact().unwrap();
    }

    let timer = Instant::now();
    for _ in 0..30000 {
        let _ = evm.transact().unwrap();
    }
    let time = timer.elapsed();
    println!("Elapsed time: {:?}", time);
}
