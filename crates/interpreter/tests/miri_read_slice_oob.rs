//! Miri reproducer: OOB read via `ExtBytecode::read_slice` with a manually crafted
//! `Bytecode::new_analyzed` that has insufficient padding.
//!
//! Run: `cargo +nightly miri test --test miri_read_slice_oob -p revm-interpreter`
//!
//! This test intentionally violates the safety contract of `new_analyzed` to
//! demonstrate that Miri catches the resulting UB. It is gated behind `#[cfg(miri)]`
//! so it does not run under normal `cargo test`.

#![cfg(miri)]

use bytecode::bitvec::{bitvec, order::Lsb0};
use bytecode::{opcode, Bytecode, JumpTable};
use primitives::{hardfork::SpecId, Bytes};
use revm_interpreter::{
    host::DummyHost,
    instructions::instruction_table,
    interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
    Interpreter,
};

/// Demonstrates that `Bytecode::new_analyzed` with insufficient padding causes
/// an OOB read in `ExtBytecode::read_slice`, caught by Miri.
///
/// Bytecode: [PUSH32, STOP]
/// original_len = 2, bytecode len = 2 (NO padding at all).
/// When the interpreter executes PUSH32, `read_slice(32)` will try to read
/// 32 bytes starting from offset 1, but the backing `Bytes` is only 2 bytes
/// long → UB (out-of-bounds read).
#[test]
fn read_slice_oob_via_new_analyzed() {
    // Construct a 2-byte bytecode: PUSH32 followed by STOP
    let raw = Bytes::from_static(&[opcode::PUSH32, opcode::STOP]);
    let original_len = raw.len(); // 2

    // Jump table: 2 bits, no valid jump destinations
    let jump_table = JumpTable::new(bitvec![u8, Lsb0; 0; original_len]);

    // new_analyzed only checks:
    //   - original_len <= bytecode.len()  ✓ (2 <= 2)
    //   - jump_table.len() >= original_len ✓ (2 >= 2)
    //   - bytecode non-empty               ✓
    // It does NOT check that PUSH32 has 32 bytes of immediate data available.
    //
    // SAFETY: intentionally violating the padding invariant to demonstrate UB.
    let bytecode = unsafe { Bytecode::new_analyzed(raw, original_len, jump_table) };

    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(bytecode),
        InputsImpl::default(),
        false,
        SpecId::PRAGUE,
        u64::MAX,
    );

    let table = instruction_table::<EthInterpreter, DummyHost>();
    let mut host = DummyHost::new(SpecId::PRAGUE);

    // This triggers read_slice(32) on a 2-byte buffer → UB (OOB read).
    // Miri should catch this.
    interpreter.run_plain(&table, &mut host);
}
