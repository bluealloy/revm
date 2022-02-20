use crate::{interpreter::Interpreter, Return};

pub fn pop(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    machine.stack.reduce_one()
}

pub fn push<const N: usize>(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    let start = machine.program_counter;
    // Safety: In Analazis we appended needed bytes for bytecode so that we are safe to just add without
    // checking if it is out of bound. This makes both of our unsafes block safe to do.
    let ret = machine
        .stack
        .push_slice::<N>(unsafe { core::slice::from_raw_parts(start, N) });
    machine.program_counter = unsafe { machine.program_counter.add(N) };
    ret
}

pub fn dup<const N: usize>(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    machine.stack.dup::<N>()
}

pub fn swap<const N: usize>(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    machine.stack.swap::<N>()
}
