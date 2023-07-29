use super::prelude::*;

pub(super) fn mload(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 32);
    push!(
        interpreter,
        U256::from_be_bytes::<32>(interpreter.memory.get_slice(index, 32).try_into().unwrap())
    );
}

pub(super) fn mstore(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 32);
    interpreter.memory.set_u256(index, value);
}

pub(super) fn mstore8(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index);
    memory_resize!(interpreter, index, 1);
    interpreter.memory.set_byte(index, value.byte(0))
}

pub(super) fn msize(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.len()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub(super) fn mcopy(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, CANCUN));
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
    memory_resize!(interpreter, src, dst.max(len).saturating_add(len));
    // copy memory in place
    interpreter.memory.copy(src, dst, len);
}
