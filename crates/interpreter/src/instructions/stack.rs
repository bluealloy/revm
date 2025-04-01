use crate::{
    gas,
    instructions::utility::cast_slice_to_u256,
    interpreter::Interpreter,
    interpreter_types::{Immediates, InterpreterTypes, Jumps, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

pub fn pop<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    check!(interpreter, SHANGHAI);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::ZERO);
}

pub fn push<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    push!(interpreter, U256::ZERO);
    popn_top!([], top, interpreter);

    let imm = interpreter.bytecode.read_slice(N);
    cast_slice_to_u256(imm, top);

    // Can ignore return. as relative N jump is safe operation
    interpreter.bytecode.relative_jump(N as isize);
}

pub fn dup<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    if !interpreter.stack.dup(N) {
        interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
}

pub fn swap<const N: usize, WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    assert!(N != 0);
    if !interpreter.stack.exchange(0, N) {
        interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
}

pub fn dupn<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.bytecode.read_u8();
    if !interpreter.stack.dup(imm as usize + 1) {
        interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    interpreter.bytecode.relative_jump(1);
}

pub fn swapn<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.bytecode.read_u8();
    if !interpreter.stack.exchange(0, imm as usize + 1) {
        interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    interpreter.bytecode.relative_jump(1);
}

pub fn exchange<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.bytecode.read_u8();
    let n = (imm >> 4) + 1;
    let m = (imm & 0x0F) + 1;
    if !interpreter.stack.exchange(n as usize, m as usize) {
        interpreter
            .control
            .set_instruction_result(crate::InstructionResult::StackOverflow);
    }
    interpreter.bytecode.relative_jump(1);
}

#[cfg(test)]
mod test {

    use super::*;
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
        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.pop(), Ok(U256::from(20)));
        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.pop(), Ok(U256::from(10)));
        interpreter.step(&table, &mut host);
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
        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.peek(0), Ok(U256::from(20)));
        assert_eq!(interpreter.stack.peek(1), Ok(U256::from(0)));
        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.peek(0), Ok(U256::from(10)));
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(20)));
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

        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.peek(1), Ok(U256::from(10)));
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(15)));
        interpreter.step(&table, &mut host);
        assert_eq!(interpreter.stack.peek(2), Ok(U256::from(1)));
        assert_eq!(interpreter.stack.peek(4), Ok(U256::from(15)));
    }
}
