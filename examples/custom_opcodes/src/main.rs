//! Custom opcodes example
//!
//! Demonstrates two techniques:
//! 1. Registering a custom opcode via the instruction table
//! 2. Using an `Inspector::step()` hook to run logic **before** a special opcode executes
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use revm::{
    Context, InspectEvm, MainContext, bytecode::opcode, context::{Evm, TxEnv}, database::{BENCH_TARGET, BenchmarkDB}, handler::{EthPrecompiles, instructions::EthInstructions}, inspector::{Inspector, inspectors::TracerEip3155}, interpreter::{
        Instruction, InstructionContext, Interpreter, interpreter::EthInterpreter, interpreter_types::{Immediates, Jumps}
    }, primitives::TxKind, state::Bytecode
};

/// Opcode hex value
const MY_STATIC_JUMP: u8 = 0x0C;

/// A custom inspector that hooks into `before_opcode()` to execute logic
/// **before** a specific opcode runs.
struct OpcodeHookInspector;

impl<CTX> Inspector<CTX, EthInterpreter> for OpcodeHookInspector {
    fn before_opcode(
        &mut self,
        interp: &mut Interpreter<EthInterpreter>,
        _context: &mut CTX,
        opcode: u8,
    ) {
        if opcode == MY_STATIC_JUMP {
            // This runs BEFORE the opcode executes.
            // You can inspect or modify interpreter state here.
            println!(
                "[hook] About to execute custom opcode 0x{:02X} at pc={}",
                opcode,
                interp.bytecode.pc(),
            );
        }
    }
}

/// Demonstrates how to implement and use custom opcodes in REVM.
/// This example shows how to create a custom static jump opcode that reads
/// a 16-bit offset from the bytecode and performs a relative jump,
/// with an inspector hook that fires before the opcode executes.
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
        Instruction::new(
            |ctx: InstructionContext<'_, _, EthInterpreter>| {
                let offset = ctx.interpreter.bytecode.read_i16();
                ctx.interpreter.bytecode.relative_jump(offset as isize);
            },
            0,
        ),
    );

    // Create a new EVM instance.
    // Combine the pre-opcode hook inspector with the tracer using a tuple.
    let mut evm = Evm::new(ctx, instructions, EthPrecompiles::default()).with_inspector((
        OpcodeHookInspector,
        TracerEip3155::new_stdout().without_summary(),
    ));

    // inspect the transaction.
    let _ = evm.inspect_one_tx(
        TxEnv::builder()
            .kind(TxKind::Call(BENCH_TARGET))
            .build()
            .unwrap(),
    );

    // Expected output — note the [hook] line fires BEFORE the custom opcode:
    /*
    [hook] About to execute custom opcode 0x0C at pc=0
    {"pc":0,"op":12,"gas":"0x1c97178","gasCost":"0x0","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0"}
    {"pc":4,"op":91,"gas":"0x1c97178","gasCost":"0x1","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0","opName":"JUMPDEST"}
    {"pc":5,"op":0,"gas":"0x1c97177","gasCost":"0x0","stack":[],"depth":1,"returnData":"0x","refund":"0x0","memSize":"0x0","opName":"STOP"}
    */
}
