use crate::{
    gas,
    instructions::utility::cast_slice_to_u256,
    interpreter_types::{Immediates, InterpreterTypes, Jumps, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

use super::context::InstructionContext;

pub fn pop<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], ctx.interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    check!(ctx.interpreter, SHANGHAI);
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, U256::ZERO);
}

pub fn push<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::VERYLOW);
    push!(ctx.interpreter, U256::ZERO);
    popn_top!([], top, ctx.interpreter);

    let imm = ctx.interpreter.bytecode.read_slice(N);
    cast_slice_to_u256(imm, top);

    // Can ignore return. as relative N jump is safe operation
    ctx.interpreter.bytecode.relative_jump(N as isize);
}

pub fn dup<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::VERYLOW);
    if !ctx.interpreter.stack.dup(N) {
        ctx.interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
}

pub fn swap<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::VERYLOW);
    assert!(N != 0);
    if !ctx.interpreter.stack.exchange(0, N) {
        ctx.interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
}

pub fn dupn<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::VERYLOW);
    let imm = ctx.interpreter.bytecode.read_u8();
    if !ctx.interpreter.stack.dup(imm as usize + 1) {
        ctx.interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    ctx.interpreter.bytecode.relative_jump(1);
}

pub fn swapn<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::VERYLOW);
    let imm = ctx.interpreter.bytecode.read_u8();
    if !ctx.interpreter.stack.exchange(0, imm as usize + 1) {
        ctx.interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    ctx.interpreter.bytecode.relative_jump(1);
}

pub fn exchange<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    require_eof!(ctx.interpreter);
    gas!(ctx.interpreter, gas::VERYLOW);
    let imm = ctx.interpreter.bytecode.read_u8();
    let n = (imm >> 4) + 1;
    let m = (imm & 0x0F) + 1;
    if !ctx.interpreter.stack.exchange(n as usize, m as usize) {
        ctx.interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    ctx.interpreter.bytecode.relative_jump(1);
}

#[cfg(test)]
mod test {

    use crate::instructions::context::InstructionContext;
    use crate::Interpreter;
    use crate::{host::DummyHost, instruction_table, InstructionResult};
    use bytecode::opcode::{DUPN, EXCHANGE, STOP, SWAPN};
    use bytecode::Bytecode;
    use primitives::{Bytes, U256};

    #[test]
    fn dupn() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[DUPN, 0x00, DUPN, 0x01, DUPN, 0x02, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        let mut host = DummyHost;

        interpreter.runtime_flag.is_eof = true;
        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(20));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.pop(), Ok(U256::from(20)));
        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.pop(), Ok(U256::from(10)));
        ctx.step(&table);
        assert_eq!(
            interpreter.control.instruction_result,
            InstructionResult::StackOverflow
        );
    }

    #[test]
    fn swapn() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[SWAPN, 0x00, SWAPN, 0x01, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        let mut host = DummyHost;
        interpreter.runtime_flag.is_eof = true;

        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(20));
        let _ = interpreter.stack.push(U256::from(0));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.peek(0), Ok(U256::from(20)));
        assert_eq!(ctx.interpreter.stack.peek(1), Ok(U256::from(0)));
        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.peek(0), Ok(U256::from(10)));
        assert_eq!(ctx.interpreter.stack.peek(2), Ok(U256::from(20)));
    }

    #[test]
    fn exchange() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[EXCHANGE, 0x00, EXCHANGE, 0x11, STOP]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        let mut host = DummyHost;
        interpreter.runtime_flag.is_eof = true;

        let _ = interpreter.stack.push(U256::from(1));
        let _ = interpreter.stack.push(U256::from(5));
        let _ = interpreter.stack.push(U256::from(10));
        let _ = interpreter.stack.push(U256::from(15));
        let _ = interpreter.stack.push(U256::from(0));

        let mut ctx = InstructionContext {
            interpreter: &mut interpreter,
            host: &mut host,
        };

        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.peek(1), Ok(U256::from(10)));
        assert_eq!(ctx.interpreter.stack.peek(2), Ok(U256::from(15)));
        ctx.step(&table);
        assert_eq!(ctx.interpreter.stack.peek(2), Ok(U256::from(1)));
        assert_eq!(ctx.interpreter.stack.peek(4), Ok(U256::from(15)));
    }
}
