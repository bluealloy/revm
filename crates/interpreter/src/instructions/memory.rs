use crate::{interpreter::Interpreter, primitives::U256, Host, InstructionResult};

pub fn mload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 32);
    push!(
        interpreter,
        U256::from_be_bytes::<{ U256::BYTES }>(
            interpreter.memory.get_slice(index, 32).try_into().unwrap()
        )
    );
}

pub fn mstore(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 32);
    interpreter.memory.set_u256(index, value);
}

pub fn mstore8(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 1);
    let value = value.as_le_bytes()[0];
    // Safety: we resized our memory two lines above.
    unsafe { interpreter.memory.set_byte(index, value) }
}

pub fn msize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.effective_len()));
}
