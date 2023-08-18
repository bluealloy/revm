use core::cmp::max;

use revm_primitives::SpecId::CANCUN;

use crate::{
    gas,
    interpreter::Interpreter,
    primitives::{Spec, U256},
    Host, InstructionResult,
};

pub fn mload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
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
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 32);
    interpreter.memory.set_u256(index, value);
}

pub fn mstore8(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index, value);
    let index = as_usize_or_fail!(interpreter, index, InstructionResult::InvalidOperandOOG);
    memory_resize!(interpreter, index, 1);
    let value = value.as_le_bytes()[0];
    // Safety: we resized our memory two lines above.
    unsafe { interpreter.memory.set_byte(index, value) }
}

pub fn msize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.effective_len()));
}

// From EIP-5656 MCOPY
pub fn mcopy<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // Opcode enabled in Cancun.
    // EIP-5656: MCOPY - Memory copying instruction
    check!(interpreter, SPEC::enabled(CANCUN));
    // get src and dest and length from stack
    pop!(interpreter, dest, src, len);

    // into usize or fail
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    // deduce gas
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }

    let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidOperandOOG);
    let src = as_usize_or_fail!(interpreter, src, InstructionResult::InvalidOperandOOG);
    // resize memory
    memory_resize!(interpreter, max(dest, src), len);
    // copy memory in place
    interpreter.memory.copy(dest, src, len);
}
