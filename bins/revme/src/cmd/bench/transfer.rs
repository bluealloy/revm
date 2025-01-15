use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    primitives::{TxKind, U256},
    transact_main, Context,
};

pub fn run() {
    let mut context = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_tx_chained(|tx| {
            // Execution globals block hash/gas_limit/coinbase/timestamp..
            tx.caller = "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap();
            tx.value = U256::from(10);
            tx.kind = TxKind::Call(
                "0x0000000000000000000000000000000000000000"
                    .parse()
                    .unwrap(),
            );
        });
    let _ = transact_main(&mut context);
}
