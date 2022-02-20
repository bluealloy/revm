use crate::{interpreter::Interpreter, Return};

pub fn pop(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    interp.stack.reduce_one()
}

pub fn push<const N: usize>(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    let start = interp.program_counter;
    // Safety: In Analazis we appended needed bytes for bytecode so that we are safe to just add without
    // checking if it is out of bound. This makes both of our unsafes block safe to do.
    let ret = interp
        .stack
        .push_slice::<N>(unsafe { core::slice::from_raw_parts(start, N) });
    interp.program_counter = unsafe { interp.program_counter.add(N) };
    ret
}

pub fn dup<const N: usize>(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    interp.stack.dup::<N>()
}

pub fn swap<const N: usize>(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    interp.stack.swap::<N>()
}
