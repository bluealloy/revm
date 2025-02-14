use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    Host,
};
use core::cmp::max;
use primitives::U256;

pub fn mload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    popn_top!([], top, interpreter);
    let offset = as_usize_or_fail!(interpreter, top);
    resize_memory!(interpreter, offset, 32);
    *top = U256::try_from_be_slice(interpreter.memory.slice_len(offset, 32).as_ref()).unwrap()
}

pub fn mstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    popn!([offset, value], interpreter);
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 32);
    interpreter.memory.set(offset, &value.to_be_bytes::<32>());
}

pub fn mstore8<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    popn!([offset, value], interpreter);
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 1);
    interpreter.memory.set(offset, &[value.byte(0)]);
}

pub fn msize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.memory.size()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    check!(interpreter, CANCUN);
    popn!([dst, src, len], interpreter);

    // Into usize or fail
    let len = as_usize_or_fail!(interpreter, len);
    // Deduce gas
    gas_or_fail!(interpreter, gas::copy_cost_verylow(len));
    if len == 0 {
        return;
    }

    let dst = as_usize_or_fail!(interpreter, dst);
    let src = as_usize_or_fail!(interpreter, src);
    // Resize memory
    resize_memory!(interpreter, max(dst, src), len);
    // Copy memory in place
    interpreter.memory.copy(dst, src, len);
}
