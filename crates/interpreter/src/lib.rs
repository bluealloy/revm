//! # revm-interpreter
//!
//! Interpreter is part of the project that executes EVM instructions.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(explicit_tail_calls)]
#![allow(incomplete_features)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// TODO(dani): rename `context` to `cx`
// TODO(dani): we require `ip`-accessing methods to be used through context instead of directly on the bytecode

#[macro_use]
mod macros;

/// a
#[no_mangle]
#[cfg(feature = "asm")]
pub fn instruction_tables() -> impl Sized {
    type W = crate::interpreter::EthInterpreter;
    type H = context_interface::DummyHost;
    (
        instructions::instruction_table::<W, H>(),
        instructions::instruction_table_tail::<W, H>(),
    )
}
/// b
pub type EEEInterpreter = crate::interpreter::Interpreter<crate::interpreter::EthInterpreter>;

/// Gas calculation utilities and constants.
pub mod gas;
/// Context passed to instruction implementations.
pub mod instruction_context;
/// Instruction execution results and success/error types.
mod instruction_result;
/// EVM instruction implementations organized by category.
pub mod instructions;
/// Core interpreter implementation for EVM bytecode execution.
pub mod interpreter;
/// Types for interpreter actions like calls and contract creation.
pub mod interpreter_action;
/// Type traits and definitions for interpreter customization.
pub mod interpreter_types;

// Reexport primary types.
pub use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    CreateScheme,
};
pub use context_interface::{host, Host};
pub use gas::{Gas, InitialAndFloorGas};
pub use instruction_context::{InstructionContext, InstructionContextTr};
pub use instruction_result::*;
pub use instructions::{instruction_table, Instruction, InstructionTable};
pub use interpreter::{
    num_words, InputsImpl, Interpreter, InterpreterResult, SharedMemory, Stack, STACK_LIMIT,
};
pub use interpreter_action::{
    CallInput, CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome,
    FrameInput, InterpreterAction,
};
pub use interpreter_types::InterpreterTypes;
pub use primitives::{eip7907::MAX_CODE_SIZE, eip7907::MAX_INITCODE_SIZE};

/// asdf
///
/// # Safety
///
/// Not safe
pub unsafe fn extend_lt<'a, T: ?Sized>(x: &T) -> &'a T {
    std::mem::transmute(x)
}

/// asdf
///
/// # Safety
///
/// Not safe
pub unsafe fn extend_lt_mut<'a, T: ?Sized>(x: &mut T) -> &'a mut T {
    std::mem::transmute(x)
}
