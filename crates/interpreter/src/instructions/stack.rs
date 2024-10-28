use crate::{gas, interpreter::InterpreterTrait, Host, InstructionResult};
use primitives::U256;

pub fn pop<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    // can ignore return. as relative N jump is safe operation.
    let _ = interpreter.popn::<1>();
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    check!(interpreter, SHANGHAI);
    gas!(interpreter, gas::BASE);
    let _ = interpreter.push(U256::ZERO);
}

pub fn push<const N: usize, I: InterpreterTrait, H: Host + ?Sized>(
    interpreter: &mut I,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);

    // can ignore return. as relative N jump is safe opeation.
    interpreter.pushn(N);
    interpreter.relative_jump(N as isize);
}

pub fn dup<const N: usize, I: InterpreterTrait, H: Host + ?Sized>(
    interpreter: &mut I,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    interpreter.dup(N);
}

pub fn swap<const N: usize, I: InterpreterTrait, H: Host + ?Sized>(
    interpreter: &mut I,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    assert!(N != 0);
    interpreter.exchange(0, N);
}

pub fn dupn<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.read_u8();
    interpreter.dup(imm as usize + 1);
    interpreter.relative_jump(1);
}

pub fn swapn<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.read_u8();
    interpreter.exchange(0, imm as usize + 1);
    interpreter.relative_jump(1);
}

pub fn exchange<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = interpreter.read_u8();
    let n = (imm >> 4) + 1;
    let m = (imm & 0x0F) + 1;
    interpreter.exchange(n as usize, m as usize);
    interpreter.relative_jump(1);
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::Interpreter;
    use crate::{table::make_instruction_table, DummyHost, Gas, InstructionResult};
    use bytecode::opcode::{DUPN, EXCHANGE, SWAPN};
    use bytecode::Bytecode;
    use specification::hardfork::{PragueSpec, SpecId};
    use wiring::DefaultEthereumWiring;

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
