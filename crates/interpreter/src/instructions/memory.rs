use crate::{
    gas,
    interpreter::NewInterpreter,
    interpreter_wiring::{InterpreterWire, LoopControl, MemoryTrait, RuntimeFlag, StackTrait},
    Host,
};
use core::cmp::max;
use primitives::U256;

pub fn mload<WIRE: InterpreterWire, H: Host + ?Sized>(
    interpreter: &mut NewInterpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    let Some(top) = interpreter.stack.top() else {
        return;
    };
    let offset = as_usize_or_fail!(interpreter, top);
    resize_memory!(interpreter, offset, 32);
    *top = U256::try_from_be_slice(interpreter.memory.slice_len(offset, 32).as_ref())
        .unwrap()
        .into();
}

pub fn mstore<WIRE: InterpreterWire, H: Host + ?Sized>(
    interpreter: &mut NewInterpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    let Some([offset, value]) = interpreter.stack.popn() else {
        return;
    };
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 32);
    interpreter.memory.set(offset, &value.to_be_bytes::<32>());
}

pub fn mstore8<WIRE: InterpreterWire, H: Host + ?Sized>(
    interpreter: &mut NewInterpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    popn!([offset, value], interpreter);
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 1);
    interpreter.memory.set(offset, &[value.byte(0)]);
}

pub fn msize<WIRE: InterpreterWire, H: Host + ?Sized>(
    interpreter: &mut NewInterpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    // result can be ignored.
    push!(interpreter, U256::from(interpreter.memory.size()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<WIRE: InterpreterWire, H: Host + ?Sized>(
    interpreter: &mut NewInterpreter<WIRE>,
    _host: &mut H,
) {
    check!(interpreter, CANCUN);
    let Some([dst, src, len]) = interpreter.stack.popn() else {
        return;
    };

    // into usize or fail
    let len = as_usize_or_fail!(interpreter, len);
    // deduce gas
    gas_or_fail!(interpreter, gas::copy_cost_verylow(len as u64));
    if len == 0 {
        return;
    }

    let dst = as_usize_or_fail!(interpreter, dst);
    let src = as_usize_or_fail!(interpreter, src);
    // resize memory
    resize_memory!(interpreter, max(dst, src), len);
    // copy memory in place
    interpreter.memory.copy(dst, src, len);
}
