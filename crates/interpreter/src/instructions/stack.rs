use crate::{
    gas,
    instructions::utility::cast_slice_to_u256,
    interpreter_types::{Immediates, InterpreterTypes, Jumps, RuntimeFlag, StackTr},
    InstructionResult,
};
use primitives::U256;

use crate::InstructionContext;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
pub fn pop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context.interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, SHANGHAI);
    gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, U256::ZERO);
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from the bytecode onto the stack.
pub fn push<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    push!(context.interpreter, U256::ZERO);
    popn_top!([], top, context.interpreter);

    let imm = context.interpreter.bytecode.read_slice(N);
    cast_slice_to_u256(imm, top);

    // Can ignore return. as relative N jump is safe operation
    context.interpreter.bytecode.relative_jump(N as isize);
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top.
pub fn dup<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    if !context.interpreter.stack.dup(N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Exchanges the top stack item with the Nth stack item.
pub fn swap<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// Implements the DUPN instruction.
///
/// Duplicates the stack item at immediate offset to the top (EOF only).
pub fn dupn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::VERYLOW);
    let imm = context.interpreter.bytecode.read_u8();
    if !context.interpreter.stack.dup(imm as usize + 1) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
    context.interpreter.bytecode.relative_jump(1);
}

/// Implements the SWAPN instruction.
///
/// Exchanges the top stack item with the item at immediate offset (EOF only).
pub fn swapn<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::VERYLOW);
    let imm = context.interpreter.bytecode.read_u8();
    if !context.interpreter.stack.exchange(0, imm as usize + 1) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
    context.interpreter.bytecode.relative_jump(1);
}

/// Implements the EXCHANGE instruction.
///
/// Exchanges two stack items at specified positions (EOF only).
pub fn exchange<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::VERYLOW);
    let imm = context.interpreter.bytecode.read_u8();
    let n = (imm >> 4) + 1;
    let m = (imm & 0x0F) + 1;
    if !context.interpreter.stack.exchange(n as usize, m as usize) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
    context.interpreter.bytecode.relative_jump(1);
}

#[cfg(test)]
mod test {
    use crate::{instruction_table, InstructionResult, Interpreter, InterpreterAction};
    use bytecode::opcode::{DUPN, EXCHANGE, STOP, SWAPN};
    use bytecode::Bytecode;
    use primitives::{Bytes, U256};

    #[test]
    fn dupn() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[DUPN, 0x00, DUPN, 0x01, DUPN, 0x02, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();

        interpreter.runtime_flag.is_eof = true;
        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(20));

        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.pop(), Ok(U256::from(20)));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.pop(), Ok(U256::from(10)));
        interpreter.step_dummy(&table);
        let gas = interpreter.gas;
        assert_eq!(
            interpreter.take_next_action(),
            InterpreterAction::new_halt(InstructionResult::StackOverflow, gas)
        );
    }

    #[test]
    fn swapn() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[SWAPN, 0x00, SWAPN, 0x01, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        interpreter.runtime_flag.is_eof = true;

        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(20));
        let _ = interpreter.stack.push(U256::from(0));

        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.peek(0), Ok(U256::from(20)));
        assert_eq!(interpreter.stack.peek(1), Ok(U256::from(0)));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.peek(0), Ok(U256::from(10)));
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(20)));
    }

    #[test]
    fn exchange() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[EXCHANGE, 0x00, EXCHANGE, 0x11, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        interpreter.runtime_flag.is_eof = true;

        let _ = interpreter.stack.push(U256::from(1));
        let _ = interpreter.stack.push(U256::from(5));
        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(15));
        let _ = interpreter.stack.push(U256::from(0));

        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.peek(1), Ok(U256::from(10)));
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(15)));
        interpreter.step_dummy(&table);
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(1)));
        assert_eq!(interpreter.stack.peek(4), Ok(U256::from(15)));
    }
}
