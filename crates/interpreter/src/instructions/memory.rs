use crate::{gas, interpreter::InterpreterTrait, Host};
use core::cmp::max;
use primitives::U256;

pub fn mload<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    let Some(top) = interpreter.top() else { return };
    let offset = as_usize_or_fail!(interpreter, top);
    resize_memory!(interpreter, offset, 32);
    *top = interpreter
        .mem_slice_len(offset, 32)
        .try_into::<[u8; 32]>()
        .unwrap()
        .into();
}

pub fn mstore<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    let Some([offset, value]) = interpreter.popn() else {
        return;
    };
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 32);
    interpreter.mem_set(offset, &value.to_be_bytes::<32>());
}

pub fn mstore8<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    let Some([offset, value]) = interpreter.popn() else {
        return;
    };
    let offset = as_usize_or_fail!(interpreter, offset);
    resize_memory!(interpreter, offset, 1);
    interpreter.mem_set(offset, value.byte(0))
}

pub fn msize<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.mem_size()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, _host: &mut H) {
    check!(interpreter, CANCUN);
    let Some([dst, src, len]) = interpreter.popn() else {
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
    interpreter.mem_copy(dst, src, len);
}
