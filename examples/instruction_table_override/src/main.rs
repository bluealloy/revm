use database::{BenchmarkDB, FFADDRESS};
use revm::{
    context::Context,
    primitives::TxKind,
    state::Bytecode,
};
use crate::exec::transact_custom_opcode;

pub mod exec;
pub mod handler;
pub mod instructions;
fn main() {
    let bytecode = Bytecode::new_legacy(
        [
            0x60, 0x01, // PUSH1 1 -> value to clz
            0x5f, // CLZ -> should be 255
            0x60, 0x00, // PUSH1 0 -> memory starting position
            0x52, // MSTORE -> store 255 to memory at location 0
            0x60, 0x20, // PUSH1 32 (length to return)
            0x60, 0x00, // PUSH1 0 (memory position)
            0xf3, // RETURN
        ]
        .into(),
    );


    let mut ctx = Context::builder()
        .with_db(BenchmarkDB::new_bytecode(bytecode)) // Store bytecode in DB
        .modify_tx_chained(|tx| {
            tx.kind = TxKind::Call(FFADDRESS); // Call the address where bytecode is stored
        });

    let result = transact_custom_opcode(&mut ctx).expect("execution failed");
    println!("Should return 0xff (255 in decimal) which is the clz of 256 bit value 0x01. The actual return is: {}", result.result.output().unwrap());
}
