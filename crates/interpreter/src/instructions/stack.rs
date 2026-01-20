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

    // Can ignore return. as relative N jump is safe operation
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

/// Decodes a single immediate byte for DUPN/SWAPN according to EIP-8024.
///
/// Returns `None` if the immediate is in the forbidden range (91..=127).
/// Otherwise returns the decoded depth value (17-235).
#[inline]
const fn decode_single(x: u8) -> Option<usize> {
    // Forbidden range: 90 < x < 128 (i.e., 91..=127)
    if x > 90 && x < 128 {
        return None;
    }
    // Decode: if x <= 90: return x + 17, else return x - 20
    Some(if x <= 90 {
        x as usize + 17
    } else {
        x as usize - 20
    })
}

/// Decodes a pair immediate byte for EXCHANGE according to EIP-8024.
///
/// Returns `None` if the immediate is in the forbidden range (80..=127).
/// Otherwise returns the decoded pair (n, m) where n and m are 1-indexed depths.
#[inline]
const fn decode_pair(x: u8) -> Option<(usize, usize)> {
    // Forbidden range: 79 < x < 128 (i.e., 80..=127)
    if x > 79 && x < 128 {
        return None;
    }
    // k = x if x <= 79 else x - 48
    let k = if x <= 79 { x as usize } else { x as usize - 48 };
    let q = k / 16;
    let r = k % 16;
    // if q < r: return (q + 1, r + 1), else return (r + 1, 29 - q)
    Some(if q < r { (q + 1, r + 1) } else { (r + 1, 29 - q) })
}

/// EIP-8024: DUPN instruction
///
/// Duplicates the n'th stack item at the top of the stack, where n is decoded
/// from a single immediate byte.
pub fn dupn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let imm = context.interpreter.bytecode.read_u8();
    context.interpreter.bytecode.relative_jump(1);

    let Some(n) = decode_single(imm) else {
        context.interpreter.halt(InstructionResult::InvalidOperandOOG);
        return;
    };

    if !context.interpreter.stack.dup(n) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// EIP-8024: SWAPN instruction
///
/// Swaps the top stack item with the n+1'th stack item, where n is decoded
/// from a single immediate byte.
pub fn swapn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let imm = context.interpreter.bytecode.read_u8();
    context.interpreter.bytecode.relative_jump(1);

    let Some(n) = decode_single(imm) else {
        context.interpreter.halt(InstructionResult::InvalidOperandOOG);
        return;
    };

    // SWAPN swaps top (index 0) with item at index n
    if !context.interpreter.stack.exchange(0, n) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// EIP-8024: EXCHANGE instruction
///
/// Swaps the n+1'th stack item with the m+1'th stack item, where (n, m) are
/// decoded from a single immediate byte.
pub fn exchange<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, AMSTERDAM);
    let imm = context.interpreter.bytecode.read_u8();
    context.interpreter.bytecode.relative_jump(1);

    let Some((n, m)) = decode_pair(imm) else {
        context.interpreter.halt(InstructionResult::InvalidOperandOOG);
        return;
    };

    // EXCHANGE swaps the (n+1)'th stack item with the (m+1)'th stack item
    // Per EIP-8024: n+1'th item is at 0-indexed position n, m+1'th item is at position m
    // stack.exchange(a, b) swaps position a with position a+b
    // So we need: a = n, a + b = m, thus b = m - n
    // But m could be less than n, so we need to handle that
    let (a, b) = if n <= m {
        (n, m - n)
    } else {
        (m, n - m)
    };

    // Note: exchange requires b > 0, but decode_pair can return equal values in edge cases
    // However per EIP-8024 spec, n and m should always be different
    if b == 0 || !context.interpreter.stack.exchange(a, b) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        host::DummyHost,
        instructions::instruction_table,
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, Interpreter, SharedMemory},
    };
    use bytecode::{opcode, Bytecode};
    use primitives::{hardfork::SpecId, Bytes};

    /// Helper to create an interpreter with given bytecode for AMSTERDAM hardfork
    fn create_interpreter(code: &[u8]) -> Interpreter {
        let bytecode = Bytecode::new_raw(Bytes::from(code.to_vec()));
        Interpreter::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::AMSTERDAM,
            u64::MAX,
        )
    }

    #[test]
    fn test_decode_single() {
        // Test valid range: 0..=90 maps to 17..=107
        assert_eq!(decode_single(0), Some(17));
        assert_eq!(decode_single(1), Some(18));
        assert_eq!(decode_single(90), Some(107));

        // Test forbidden range: 91..=127 returns None
        for x in 91..=127 {
            assert_eq!(decode_single(x), None, "Expected None for forbidden value {}", x);
        }

        // Test valid range: 128..=255 maps to 108..=235
        assert_eq!(decode_single(128), Some(108));
        assert_eq!(decode_single(129), Some(109));
        assert_eq!(decode_single(255), Some(235));
    }

    #[test]
    fn test_decode_pair() {
        // Test valid range: 0..=79
        // x=0: k=0, q=0, r=0, q>=r so (r+1, 29-q) = (1, 29)
        assert_eq!(decode_pair(0), Some((1, 29)));

        // x=1: k=1, q=0, r=1, q<r so (q+1, r+1) = (1, 2)
        assert_eq!(decode_pair(1), Some((1, 2)));

        // x=17: k=17, q=1, r=1, q>=r so (r+1, 29-q) = (2, 28)
        assert_eq!(decode_pair(17), Some((2, 28)));

        // x=79: k=79, q=4, r=15, q<r so (q+1, r+1) = (5, 16)
        assert_eq!(decode_pair(79), Some((5, 16)));

        // Test forbidden range: 80..=127 returns None
        for x in 80..=127 {
            assert_eq!(decode_pair(x), None, "Expected None for forbidden value {}", x);
        }

        // Test valid range: 128..=255 (k = x - 48, so 80..=207)
        // x=128: k=80, q=5, r=0, q>=r so (r+1, 29-q) = (1, 24)
        assert_eq!(decode_pair(128), Some((1, 24)));

        // x=255: k=207, q=12, r=15, q<r so (q+1, r+1) = (13, 16)
        assert_eq!(decode_pair(255), Some((13, 16)));
    }

    #[test]
    fn test_dupn_basic() {
        // Test DUPN with immediate 0 (decodes to n=17)
        // We need at least 17 items on stack, and it will duplicate the 17th item
        let code = [opcode::DUPN, 0x00, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push 17 values: 1, 2, 3, ..., 17 (17 is at top)
        for i in 1..=17 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        // Run the instruction using the instruction table
        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        // Stack should now have 18 items, with value 1 (17th from top) duplicated at top
        assert!(action.is_return());
        assert_eq!(interpreter.stack.len(), 18);
        let top = interpreter.stack.pop().unwrap();
        assert_eq!(top, U256::from(1)); // 17th item from original top was value 1
    }

    #[test]
    fn test_dupn_forbidden_immediate() {
        // Test DUPN with forbidden immediate (91)
        let code = [opcode::DUPN, 91, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push enough values
        for i in 1..=20 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        // Should have halted with InvalidOperandOOG
        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::InvalidOperandOOG)
        );
    }

    #[test]
    fn test_swapn_basic() {
        // Test SWAPN with immediate 0 (decodes to n=17)
        // SWAPN swaps top with item at position n (17)
        let code = [opcode::SWAPN, 0x00, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push 18 values: 1, 2, 3, ..., 18 (18 is at top)
        // Position 17 from top has value 2
        for i in 1..=18 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        // Stack should still have 18 items
        assert!(action.is_return());
        assert_eq!(interpreter.stack.len(), 18);

        // Top should now be value 1 (was at position 17)
        // Position 17 should now be value 18 (was at top)
        let top = interpreter.stack.pop().unwrap();
        assert_eq!(top, U256::from(1));
    }

    #[test]
    fn test_swapn_forbidden_immediate() {
        // Test SWAPN with forbidden immediate (100)
        let code = [opcode::SWAPN, 100, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        for i in 1..=20 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::InvalidOperandOOG)
        );
    }

    #[test]
    fn test_exchange_basic() {
        // Test EXCHANGE with immediate 1 (decodes to n=1, m=2)
        // EXCHANGE swaps item at position n with item at position m
        let code = [opcode::EXCHANGE, 0x01, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push 5 values: 1, 2, 3, 4, 5 (5 is at top)
        // Position 1 from top: value 4
        // Position 2 from top: value 3
        for i in 1..=5 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        // Stack should still have 5 items
        assert!(action.is_return());
        assert_eq!(interpreter.stack.len(), 5);

        // Check that positions 1 and 2 are swapped
        // Stack from bottom: 1, 2, 4, 3, 5 (swapped 3 and 4)
        let v5 = interpreter.stack.pop().unwrap();
        let v4_swapped = interpreter.stack.pop().unwrap();
        let v3_swapped = interpreter.stack.pop().unwrap();
        assert_eq!(v5, U256::from(5)); // top unchanged
        assert_eq!(v4_swapped, U256::from(3)); // was 4, now 3
        assert_eq!(v3_swapped, U256::from(4)); // was 3, now 4
    }

    #[test]
    fn test_exchange_forbidden_immediate() {
        // Test EXCHANGE with forbidden immediate (80)
        let code = [opcode::EXCHANGE, 80, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        for i in 1..=30 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::InvalidOperandOOG)
        );
    }

    #[test]
    fn test_dupn_stack_underflow() {
        // Test DUPN when stack doesn't have enough items
        let code = [opcode::DUPN, 0x00, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push only 10 values, but DUPN with imm=0 needs 17
        for i in 1..=10 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        // Should have halted with StackOverflow (used for underflow in dup)
        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn test_swapn_stack_underflow() {
        // Test SWAPN when stack doesn't have enough items
        let code = [opcode::SWAPN, 0x00, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push only 10 values, but SWAPN with imm=0 needs 18 (top + 17)
        for i in 1..=10 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::StackOverflow)
        );
    }

    #[test]
    fn test_exchange_stack_underflow() {
        // Test EXCHANGE when stack doesn't have enough items
        // decode_pair(0) = (1, 29), needs 29 items
        let code = [opcode::EXCHANGE, 0x00, opcode::STOP];
        let mut interpreter = create_interpreter(&code);
        let mut host = DummyHost::new(SpecId::AMSTERDAM);

        // Push only 10 values
        for i in 1..=10 {
            assert!(interpreter.stack.push(U256::from(i)));
        }

        let table = instruction_table::<EthInterpreter, DummyHost>();
        let action = interpreter.run_plain(&table, &mut host);

        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::StackOverflow)
        );
    }
}
