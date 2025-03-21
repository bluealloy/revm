//! Custom opcodes example
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use revm::{
    bytecode::opcode,
    context::{Evm, TxEnv},
    database::{BenchmarkDB, BENCH_TARGET},
    handler::{instructions::EthInstructions, EthPrecompiles},
    inspector::inspectors::TracerEip3155,
    interpreter::{
        interpreter::EthInterpreter,
        interpreter_types::{Immediates, Jumps},
        Interpreter,
    },
    primitives::TxKind,
    state::Bytecode,
    Context, InspectEvm, MainContext,
};

/// Opcode hex value
const MY_STATIC_JUMP: u8 = 0x0C;

pub fn main() {
    let ctx = Context::mainnet().with_db(BenchmarkDB::new_bytecode(Bytecode::new_raw(
        [
            MY_STATIC_JUMP,
            0x00,
            0x03,
            opcode::STOP,
            opcode::JUMPDEST,
            opcode::STOP,
        ]
        .into(),
    )));

    // Create a new instruction set with our mainnet opcodes.
    let mut instructions = EthInstructions::new_mainnet();
    // insert our custom opcode
    instructions.insert_instruction(
        MY_STATIC_JUMP,
        |interpreter: &mut Interpreter<EthInterpreter>, _| {
            let offset = interpreter.bytecode.read_i16();
            interpreter.bytecode.relative_jump(offset as isize);
        },
    );

    // Create a new EVM instance.
    let mut evm = Evm::new(ctx, instructions, EthPrecompiles::default())
        .with_inspector(TracerEip3155::new_stdout().without_summary());

    // inspect the transaction.
    let _ = evm.inspect_with_tx(TxEnv {
        kind: TxKind::Call(BENCH_TARGET),
        ..Default::default()
    });

    // Expected output where we can see that JUMPDEST is called.
    /*
    "{"pc":0,"op":12,"gas":"0x1c97178","gasCost":"0x0","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0"}
    {"pc":4,"op":91,"gas":"0x1c97178","gasCost":"0x1","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0","opName":"JUMPDEST"}
    {"pc":5,"op":0,"gas":"0x1c97177","gasCost":"0x0","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0","opName":"STOP"}
    */
}
