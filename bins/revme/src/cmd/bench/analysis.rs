use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    primitives::{address, bytes, hex, Bytes, TxKind},
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
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.kind = TxKind::Call(address!("0000000000000000000000000000000000000000"));
            //evm.env.tx.data = Bytes::from(hex::decode("30627b7c").unwrap());
            tx.data = bytes!("8035F0CE");
        });
    let _ = transact_main(&mut context);
}
