use crate::{
    interpreter_types::{Immediates, InterpreterTypes, Jumps, RuntimeFlag, StackTr},
    InstructionResult,
};
use primitives::U256;

use crate::InstructionContext;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
pub fn pop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context.interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, SHANGHAI);
    push!(context.interpreter, U256::ZERO);
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from bytecode onto the stack as a 32-byte value.
pub fn push<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    let slice = context.interpreter.bytecode.read_slice(N);
    if !context.interpreter.stack.push_slice(slice) {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }

    context.interpreter.bytecode.relative_jump(N as isize);
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top of the stack.
pub fn dup<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    if !context.interpreter.stack.dup(N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Swaps the top stack item with the Nth stack item.
pub fn swap<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// Implements the DUPN instruction.
///
/// Duplicates the Nth stack item to the top of the stack, with N given by an immediate.
pub fn dupn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some(n) = decode_single(x) {
        if !context.interpreter.stack.dup(n) {
            context.interpreter.halt(InstructionResult::StackOverflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        context
            .interpreter
            .halt(InstructionResult::InvalidImmediateEncoding);
    }
}

/// Implements the SWAPN instruction.
///
/// Swaps the top stack item with the N+1th stack item, with N given by an immediate.
pub fn swapn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some(n) = decode_single(x) {
        if !context.interpreter.stack.exchange(0, n) {
            context.interpreter.halt(InstructionResult::StackOverflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        context
            .interpreter
            .halt(InstructionResult::InvalidImmediateEncoding);
    }
}

/// Implements the EXCHANGE instruction.
///
/// Swaps the N+1th stack item with the M+1th stack item, with N, M given by an immediate.
pub fn exchange<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some((n, m)) = decode_pair(x) {
        if !context.interpreter.stack.exchange(n, m - n) {
            context.interpreter.halt(InstructionResult::StackOverflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        context
            .interpreter
            .halt(InstructionResult::InvalidImmediateEncoding);
    }
}

fn decode_single(x: usize) -> Option<usize> {
    if x <= 90 {
        Some(x + 17)
    } else if x >= 128 {
        Some(x - 20)
    } else {
        None
    }
}

fn decode_pair(x: usize) -> Option<(usize, usize)> {
    let k = if x <= 79 {
        x
    } else if x >= 128 {
        x - 48
    } else {
        return None;
    };
    let q = k / 16;
    let r = k % 16;
    if q < r {
        Some((q + 1, r + 1))
    } else {
        Some((r + 1, 29 - q))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        gas::params::GasParams,
        host::DummyHost,
        instructions::instruction_table,
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
        interpreter_types::LoopControl,
        Interpreter,
    };
    use bytecode::Bytecode;
    use primitives::{hardfork::SpecId, Bytes, U256};

    fn run_bytecode(code: &[u8]) -> Interpreter {
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(code));
        let mut interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::OSAKA,
            u64::MAX,
            GasParams::default(),
        );
        let table = instruction_table::<EthInterpreter, DummyHost>();
        let mut host = DummyHost;
        interpreter.run_plain(&table, &mut host);
        interpreter
    }

    #[test]
    fn test_dupn() {
        let interpreter = run_bytecode(&[
            0x60, 0x01, 0x60, 0x00, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80, 0x80, 0xe6, 0x00,
        ]);
        assert_eq!(interpreter.stack.len(), 18);
        assert_eq!(interpreter.stack.data()[17], U256::from(1));
        assert_eq!(interpreter.stack.data()[0], U256::from(1));
        for i in 1..17 {
            assert_eq!(interpreter.stack.data()[i], U256::ZERO);
        }
    }

    #[test]
    fn test_swapn() {
        let interpreter = run_bytecode(&[
            0x60, 0x01, 0x60, 0x00, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80, 0x80, 0x60, 0x02, 0xe7, 0x00,
        ]);
        assert_eq!(interpreter.stack.len(), 18);
        assert_eq!(interpreter.stack.data()[17], U256::from(1));
        assert_eq!(interpreter.stack.data()[0], U256::from(2));
        for i in 1..17 {
            assert_eq!(interpreter.stack.data()[i], U256::ZERO);
        }
    }

    #[test]
    fn test_exchange() {
        let interpreter = run_bytecode(&[0x60, 0x00, 0x60, 0x01, 0x60, 0x02, 0xe8, 0x01]);
        assert_eq!(interpreter.stack.len(), 3);
        assert_eq!(interpreter.stack.data()[2], U256::from(2));
        assert_eq!(interpreter.stack.data()[1], U256::from(0));
        assert_eq!(interpreter.stack.data()[0], U256::from(1));
    }

    #[test]
    fn test_swapn_invalid_immediate() {
        let mut interpreter = run_bytecode(&[0xe7, 0x5b]);
        assert!(interpreter.bytecode.instruction_result().is_none());
    }

    #[test]
    fn test_jump_over_invalid_dupn() {
        let interpreter = run_bytecode(&[0x60, 0x04, 0x56, 0xe6, 0x5b]);
        assert!(interpreter.bytecode.is_not_end());
    }

    #[test]
    fn test_exchange_with_iszero() {
        let interpreter = run_bytecode(&[0x60, 0x00, 0x60, 0x00, 0x60, 0x00, 0xe8, 0x01, 0x15]);
        assert_eq!(interpreter.stack.len(), 3);
        assert_eq!(interpreter.stack.data()[2], U256::from(1));
        assert_eq!(interpreter.stack.data()[1], U256::ZERO);
        assert_eq!(interpreter.stack.data()[0], U256::ZERO);
    }
}
