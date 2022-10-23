use crate::{interpreter::Interpreter, Return};
use ruint::aliases::U256;

pub fn mload(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    pop!(interp, index);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(interp, index, 32);
    push!(
        interp,
        U256::try_from_be_slice(interp.memory.get_slice(index, 32)).unwrap()
    );
    Return::Continue
}

pub fn mstore(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    pop!(interp, index, value);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(interp, index, 32);
    interp.memory.set_u256(index, value);
    Return::Continue
}

pub fn mstore8(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    pop!(interp, index, value);
    let index = as_usize_or_fail!(index, Return::OutOfGas);
    memory_resize!(interp, index, 1);
    let value = (value.as_limbs()[0] as u32 & 0xff) as u8;
    // Safety: we resized our memory two lines above.
    unsafe { interp.memory.set_byte(index, value) }
    Return::Continue
}

pub fn msize(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, U256::from(interp.memory.effective_len()));
    Return::Continue
}
