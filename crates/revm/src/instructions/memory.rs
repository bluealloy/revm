use crate::{interpreter::Interpreter, Return};
use primitive_types::U256;

pub fn mload(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    pop!(machine, index);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 32);
    push!(
        machine,
        U256::from_big_endian(machine.memory.get_slice(index, 32))
    );
    Return::Continue
}

pub fn mstore(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    pop!(machine, index, value);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 32);
    machine.memory.set_u256(index, value);
    Return::Continue
}

pub fn mstore8(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    pop!(machine, index, value);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(machine, index, 1);
    let value = (value.low_u32() & 0xff) as u8;
    // Safety: we resized our memory two lines above.
    unsafe { machine.memory.set_byte(index, value) }
    Return::Continue
}

pub fn msize(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, U256::from(machine.memory.effective_len()));
    Return::Continue
}
