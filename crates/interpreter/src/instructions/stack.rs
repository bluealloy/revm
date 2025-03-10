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

// TODO : Tests
/*
#[cfg(test)]
mod test {

    use super::*;
    use crate::{table::make_instruction_table, DummyHost, Gas, InstructionResult};
    use bytecode::opcode::{DUPN, EXCHANGE, SWAPN};
    use bytecode::Bytecode;
    use primitives::hardfork::SpecId;
    use context_interface::DefaultEthereumWiring;

    #[test]
    fn dupn() {
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(
            [DUPN, 0x00, DUPN, 0x01, DUPN, 0x02].into(),
        ));
        interp.is_eof = true;
        interp.spec_id = SpecId::PRAGUE;
        interp.gas = Gas::new(10000);

        interp.stack.push(U256::from(10)).unwrap();
        interp.stack.push(U256::from(20)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.pop(), Ok(U256::from(20)));
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.pop(), Ok(U256::from(10)));
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::StackUnderflow);
    }

    #[test]
    fn swapn() {
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
        let mut host = DummyHost::default();
        let mut interp =
            Interpreter::new_bytecode(Bytecode::LegacyRaw([SWAPN, 0x00, SWAPN, 0x01].into()));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);
        interp.spec_id = SpecId::PRAGUE;

        interp.stack.push(U256::from(10)).unwrap();
        interp.stack.push(U256::from(20)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.peek(0), Ok(U256::from(20)));
        assert_eq!(interp.stack.peek(1), Ok(U256::from(0)));
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.peek(0), Ok(U256::from(10)));
        assert_eq!(interp.stack.peek(2), Ok(U256::from(20)));
    }

    #[test]
    fn exchange() {
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
        let mut host = DummyHost::default();
        let mut interp =
            Interpreter::new_bytecode(Bytecode::LegacyRaw([EXCHANGE, 0x00, EXCHANGE, 0x11].into()));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);
        interp.spec_id = SpecId::PRAGUE;

        interp.stack.push(U256::from(1)).unwrap();
        interp.stack.push(U256::from(5)).unwrap();
        interp.stack.push(U256::from(10)).unwrap();
        interp.stack.push(U256::from(15)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();

        interp.step(&table, &mut host);
        assert_eq!(interp.stack.peek(1), Ok(U256::from(10)));
        assert_eq!(interp.stack.peek(2), Ok(U256::from(15)));
        interp.step(&table, &mut host);
        assert_eq!(interp.stack.peek(2), Ok(U256::from(1)));
        assert_eq!(interp.stack.peek(4), Ok(U256::from(15)));
    }
}
*/
