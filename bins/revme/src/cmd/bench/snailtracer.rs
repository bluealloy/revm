use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    handler::EthHandler,
    primitives::{address, bytes, hex, Bytes, TxKind},
    Context, MainEvm,
};

pub fn simple_example(bytecode: Bytecode) {
    let context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(bytecode.clone()))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.kind = TxKind::Call(address!("0000000000000000000000000000000000000000"));
            tx.data = bytes!("30627b7c");
            tx.gas_limit = 1_000_000_000;
        });
    let mut evm = MainEvm::new(context, EthHandler::default());
    let _ = evm.transact().unwrap();
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
