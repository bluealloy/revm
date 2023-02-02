use crate::{interpreter::Interpreter, Host};

pub fn pop(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    if let Some(ret) = interpreter.stack.reduce_one() {
        interpreter.instruction_result = ret;
    }
}

pub fn push<const N: usize>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    let start = interpreter.instruction_pointer;
    // Safety: In Analysis we appended needed bytes for bytecode so that we are safe to just add without
    // checking if it is out of bound. This makes both of our unsafes block safe to do.
    if let Some(ret) = interpreter
        .stack
        .push_slice::<N>(unsafe { core::slice::from_raw_parts(start, N) })
    {
        interpreter.instruction_result = ret;
        return;
    }
    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.add(N) };
}

pub fn dup<const N: usize>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    if let Some(ret) = interpreter.stack.dup::<N>() {
        interpreter.instruction_result = ret;
    }
}

pub fn swap<const N: usize>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    if let Some(ret) = interpreter.stack.swap::<N>() {
        interpreter.instruction_result = ret;
    }
}
