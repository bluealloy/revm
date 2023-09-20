use crate::{
    gas,
    primitives::{Spec, U256},
    Host, InstructionResult, Interpreter,
};
use core::cmp::max;

pub fn mload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 32);
    push!(
        interpreter,
        U256::from_be_bytes::<32>(interpreter.memory.slice(index, 32).try_into().unwrap())
    );
}

pub fn mstore(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 32);
    interpreter.memory.set_u256(index, value);
}

pub fn mstore8(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 1);
    interpreter.memory.set_byte(index, value.byte(0))
}

pub fn msize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.len()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    check!(interpreter, CANCUN);
    pop!(interpreter, dst, src, len);

    // into usize or fail
    let len = as_usize_or_fail!(interpreter, len);
    // deduce gas
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }

    let dst = as_usize_or_fail!(interpreter, dst);
    let src = as_usize_or_fail!(interpreter, src);
    // resize memory
    memory_resize!(interpreter, max(dst, src), len);
    // copy memory in place
    interpreter.memory.copy(dst, src, len);
}
