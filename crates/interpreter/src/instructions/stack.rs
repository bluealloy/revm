use crate::{gas, Host, Interpreter};
use primitives::U256;
use specification::hardfork::Spec;

pub fn pop<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    if let Err(result) = interpreter.stack.pop() {
        interpreter.instruction_result = result;
    }
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, SHANGHAI);
    gas!(interpreter, gas::BASE);
    if let Err(result) = interpreter.stack.push(U256::ZERO) {
        interpreter.instruction_result = result;
    }
}

pub fn push<const N: usize, H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    // SAFETY: In analysis we append trailing bytes to the bytecode so that this is safe to do
    // without bounds checking.
    let ip = interpreter.instruction_pointer;
    if let Err(result) = interpreter
        .stack
        .push_slice(unsafe { core::slice::from_raw_parts(ip, N) })
    {
        interpreter.instruction_result = result;
        return;
    }
    interpreter.instruction_pointer = unsafe { ip.add(N) };
}

pub fn dup<const N: usize, H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    if let Err(result) = interpreter.stack.dup(N) {
        interpreter.instruction_result = result;
    }
}

pub fn swap<const N: usize, H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    if let Err(result) = interpreter.stack.swap(N) {
        interpreter.instruction_result = result;
    }
}

pub fn dupn<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = unsafe { *interpreter.instruction_pointer };
    if let Err(result) = interpreter.stack.dup(imm as usize + 1) {
        interpreter.instruction_result = result;
    }
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

pub fn swapn<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = unsafe { *interpreter.instruction_pointer };
    if let Err(result) = interpreter.stack.swap(imm as usize + 1) {
        interpreter.instruction_result = result;
    }
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

pub fn exchange<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    let imm = unsafe { *interpreter.instruction_pointer };
    let n = (imm >> 4) + 1;
    let m = (imm & 0x0F) + 1;
    if let Err(result) = interpreter.stack.exchange(n as usize, m as usize) {
        interpreter.instruction_result = result;
    }

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        opcode::{make_instruction_table, DUPN, EXCHANGE, SWAPN},
        DummyHost, Gas, InstructionResult,
    };
    use bytecode::Bytecode;
    use primitives::Bytes;
    use specification::hardfork::PragueSpec;
    use wiring::DefaultEthereumWiring;

    #[test]
    fn dupn() {
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([
            DUPN, 0x00, DUPN, 0x01, DUPN, 0x02,
        ])));
        interp.is_eof = true;
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
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp =
            Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([SWAPN, 0x00, SWAPN, 0x01])));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);

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
        let table = make_instruction_table::<DummyHost<DefaultEthereumWiring>, PragueSpec>();
        let mut host = DummyHost::default();
        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(Bytes::from([
            EXCHANGE, 0x00, EXCHANGE, 0x11,
        ])));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);

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
