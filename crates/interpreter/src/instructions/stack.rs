use crate::{
    interpreter_types::{Immediates, InterpreterTypes as ITy, Jumps, RuntimeFlag, StackTr},
    InstructionContext as Ictx, InstructionExecResult as Result, InstructionResult,
};
use primitives::U256;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
pub fn pop<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context.interpreter);
    Ok(())
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, SHANGHAI);
    push!(context.interpreter, U256::ZERO);
    Ok(())
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from bytecode onto the stack as a 32-byte value.
pub fn push<const N: usize, IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    let slice = context.interpreter.bytecode.read_slice(N);
    if !context.interpreter.stack.push_slice(slice) {
        return Err(InstructionResult::StackOverflow);
    }

    context.interpreter.bytecode.relative_jump(N as isize);
    Ok(())
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top of the stack.
pub fn dup<const N: usize, IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    if !context.interpreter.stack.dup(N) {
        return Err(InstructionResult::StackOverflow);
    }
    Ok(())
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Swaps the top stack item with the Nth stack item.
pub fn swap<const N: usize, IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        return Err(InstructionResult::StackUnderflow);
    }
    Ok(())
}

/// Implements the DUPN instruction.
///
/// Duplicates the Nth stack item to the top of the stack, with N given by an immediate.
pub fn dupn<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some(n) = decode_single(x) {
        if !context.interpreter.stack.dup(n) {
            return Err(InstructionResult::StackOverflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        return Err(InstructionResult::InvalidImmediateEncoding);
    }
    Ok(())
}

/// Implements the SWAPN instruction.
///
/// Swaps the top stack item with the N+1th stack item, with N given by an immediate.
pub fn swapn<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some(n) = decode_single(x) {
        if !context.interpreter.stack.exchange(0, n) {
            return Err(InstructionResult::StackUnderflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        return Err(InstructionResult::InvalidImmediateEncoding);
    }
    Ok(())
}

/// Implements the EXCHANGE instruction.
///
/// Swaps the N+1th stack item with the M+1th stack item, with N, M given by an immediate.
pub fn exchange<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, AMSTERDAM);
    let x: usize = context.interpreter.bytecode.read_u8().into();
    if let Some((n, m)) = decode_pair(x) {
        if !context.interpreter.stack.exchange(n, m - n) {
            return Err(InstructionResult::StackUnderflow);
        }
        context.interpreter.bytecode.relative_jump(1);
    } else {
        return Err(InstructionResult::InvalidImmediateEncoding);
    }
    Ok(())
}

const fn decode_single(x: usize) -> Option<usize> {
    if x <= 90 || x >= 128 {
        Some((x + 145) % 256)
    } else {
        None
    }
}

const fn decode_pair(x: usize) -> Option<(usize, usize)> {
    if x > 81 && x < 128 {
        return None;
    }
    let k = x ^ 143;
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
        host::DummyHost,
        instructions::{gas_table, instruction_table},
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
        interpreter_types::LoopControl,
        Interpreter,
    };
    use bytecode::opcode::*;
    use bytecode::Bytecode;
    use primitives::{hardfork::SpecId, Bytes, U256};

    fn run_bytecode(code: &[u8]) -> Interpreter {
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(code));
        let mut interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::AMSTERDAM,
            u64::MAX,
        );
        let table = instruction_table::<EthInterpreter, DummyHost>();
        let gas = gas_table();
        let mut host = DummyHost::new(SpecId::AMSTERDAM);
        interpreter.run_plain(&table, &gas, &mut host);
        interpreter
    }

    #[test]
    fn test_dupn() {
        let interpreter = run_bytecode(&[
            PUSH1, 0x01, PUSH1, 0x00, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1,
            DUP1, DUP1, DUP1, DUP1, DUP1, DUPN, 0x80,
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
            PUSH1, 0x01, PUSH1, 0x00, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1,
            DUP1, DUP1, DUP1, DUP1, DUP1, PUSH1, 0x02, SWAPN, 0x80,
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
        let interpreter = run_bytecode(&[PUSH1, 0x00, PUSH1, 0x01, PUSH1, 0x02, EXCHANGE, 0x8E]);
        assert_eq!(interpreter.stack.len(), 3);
        assert_eq!(interpreter.stack.data()[2], U256::from(2));
        assert_eq!(interpreter.stack.data()[1], U256::from(0));
        assert_eq!(interpreter.stack.data()[0], U256::from(1));
    }

    #[test]
    fn test_swapn_invalid_immediate() {
        let mut interpreter = run_bytecode(&[SWAPN, JUMPDEST]);
        assert!(interpreter.bytecode.instruction_result().is_none());
    }

    #[test]
    fn test_jump_over_invalid_dupn() {
        let interpreter = run_bytecode(&[PUSH1, 0x04, JUMP, DUPN, JUMPDEST]);
        assert!(interpreter.bytecode.is_not_end());
    }

    #[test]
    fn test_exchange_with_iszero() {
        let interpreter = run_bytecode(&[
            PUSH1, 0x00, PUSH1, 0x00, PUSH1, 0x00, EXCHANGE, 0x8E, ISZERO,
        ]);
        assert_eq!(interpreter.stack.len(), 3);
        assert_eq!(interpreter.stack.data()[2], U256::from(1));
        assert_eq!(interpreter.stack.data()[1], U256::ZERO);
        assert_eq!(interpreter.stack.data()[0], U256::ZERO);
    }
}
