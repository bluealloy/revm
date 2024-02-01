use revm::{
    db::BenchmarkDB,
    primitives::{Bytecode, TransactTo, U256},
    Evm,
};
use std::time::Duration;
use std::fs;
use std::env;
extern crate alloc;


fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path).unwrap_or_else(|error| {
        panic!("Couldn't read file: {:?}", error);
    });
    let contents_str = contents.to_string();
    let bytecode = hex::decode(contents_str.trim()).unwrap_or_else(|error| {
        panic!("Couldn't decode contents: {:?}", error);
    });

    let ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

    // BenchmarkDB is dummy state that implements Database trait.
    // the bytecode is deployed at zero address.
    let mut evm = Evm::builder()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_raw(bytecode.into())))
        .modify_tx_env(|tx| {
            // execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap();
            tx.transact_to = TransactTo::Call(
                ZERO_ADDRESS
                    .parse()
                    .unwrap(),
            );
        })
        .build();

    // Microbenchmark
    let bench_options = microbench::Options::default().time(Duration::from_secs(3));

    microbench::bench(&bench_options, "Run bytecode", || {
        let _ = evm.transact().unwrap();
    });
}
