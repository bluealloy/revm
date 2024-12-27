use database::BenchmarkDB;
use revm::{
    bytecode::Bytecode,
    handler::EthHandler,
    primitives::{TxKind, U256},
    Context, MainEvm,
};

pub fn run() {
    let context = Context::builder()
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
    let mut evm = MainEvm::new(context, EthHandler::default());

    let _ = evm.transact().unwrap();
}
