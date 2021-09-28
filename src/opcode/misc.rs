use super::{gas, Control};
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed},
    machine::Machine,
    opcode::gas::verylowcopy_cost,
    Spec,
};
use core::cmp::min;
use primitive_types::{H256, U256};

#[inline]
pub fn codesize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code.len());
    push_u256!(machine, size);
    Control::Continue
}

#[inline]
pub fn codecopy(machine: &mut Machine) -> Control {
    pop_u256!(machine, memory_offset, code_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));

    try_or_fail!(machine.memory.resize_offset(memory_offset, len));
    match machine
        .memory
        .copy_large(memory_offset, code_offset, len, &machine.contract.code)
    {
        Ok(()) => Control::Continue,
        Err(e) => Control::Exit(e.into()),
    }
}

#[inline]
pub fn calldataload(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index);

    let mut load = [0u8; 32];
    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
        if let Some(p) = index.checked_add(U256::from(i)) {
            if p <= U256::from(usize::MAX) {
                let p = p.as_usize();
                if p < machine.contract.input.len() {
                    load[i] = machine.contract.input[p];
                }
            }
        }
    }

    push!(machine, H256::from(load));
    Control::Continue
}

#[inline]
pub fn calldatasize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    push_u256!(machine, len);
    Control::Continue
}

#[inline]
pub fn calldatacopy(machine: &mut Machine) -> Control {
    pop_u256!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));

    try_or_fail!(machine.memory.resize_offset(memory_offset, len));
    if len == U256::zero() {
        return Control::Continue;
    }

    match machine
        .memory
        .copy_large(memory_offset, data_offset, len, &machine.contract.input)
    {
        Ok(()) => Control::Continue,
        Err(e) => Control::Exit(e.into()),
    }
}

#[inline]
pub fn pop(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    pop!(machine, _val);
    Control::Continue
}

#[inline]
pub fn mload(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);
    pop_u256!(machine, index);

    // memory aditional gas checked here
    try_or_fail!(machine.memory.resize_offset(index, U256::from(32)));
    let index = as_usize_or_fail!(index);
    let value = H256::from_slice(&machine.memory.get(index, 32)[..]);
    push!(machine, value);
    Control::Continue
}

#[inline]
pub fn mstore(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index);
    pop!(machine, value);

    // memory aditional gas checked here
    try_or_fail!(machine.memory.resize_offset(index, U256::from(32)));
    let index = as_usize_or_fail!(index);
    match machine.memory.set(index, &value[..], Some(32)) {
        Ok(()) => Control::Continue,
        Err(e) => Control::Exit(e.into()),
    }
}

#[inline]
pub fn mstore8(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index, value);

    // memory aditional gas checked here
    try_or_fail!(machine.memory.resize_offset(index, U256::one()));
    let index = as_usize_or_fail!(index);
    let value = (value.low_u32() & 0xff) as u8;
    match machine.memory.set(index, &[value], Some(1)) {
        Ok(()) => Control::Continue,
        Err(e) => Control::Exit(e.into()),
    }
}

#[inline]
pub fn jump(machine: &mut Machine) -> Control {
    gas!(machine, gas::MID);

    pop_u256!(machine, dest);
    let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

    if machine.contract.jumpdest.is_valid(dest) {
        Control::Jump(dest)
    } else {
        Control::Exit(ExitError::InvalidJump.into())
    }
}

#[inline]
pub fn jumpi(machine: &mut Machine) -> Control {
    gas!(machine, gas::HIGH);

    pop_u256!(machine, dest);
    pop!(machine, value);
    let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

    if value != H256::zero() {
        if machine.contract.jumpdest.is_valid(dest) {
            Control::Jump(dest)
        } else {
            Control::Exit(ExitError::InvalidJump.into())
        }
    } else {
        Control::Continue
    }
}

#[inline]
pub fn pc(machine: &mut Machine, position: usize) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, U256::from(position));
    Control::Continue
}

#[inline]
pub fn msize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, machine.memory.effective_len());
    Control::Continue
}

#[inline]
pub fn push(machine: &mut Machine, n: usize, position: usize) -> Control {
    gas!(machine, gas::VERYLOW);
    let end = min(position + 1 + n, machine.contract.code.len());
    let slice = &machine.contract.code[(position + 1)..end];
    let mut val = [0u8; 32];
    val[(32 - slice.len())..32].copy_from_slice(slice);

    push!(machine, H256(val));
    Control::ContinueN(1 + n)
}

#[inline]
pub fn dup(machine: &mut Machine, n: usize) -> Control {
    gas!(machine, gas::VERYLOW);

    let value = match machine.stack.peek(n - 1) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e.into()),
    };
    push!(machine, value);
    Control::Continue
}

#[inline]
pub fn swap(machine: &mut Machine, n: usize) -> Control {
    gas!(machine, gas::VERYLOW);

    let val1 = match machine.stack.peek(0) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e.into()),
    };
    let val2 = match machine.stack.peek(n) {
        Ok(value) => value,
        Err(e) => return Control::Exit(e.into()),
    };
    match machine.stack.set(0, val2) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    }
    match machine.stack.set(n, val1) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    }
    Control::Continue
}

#[inline]
pub fn ret(machine: &mut Machine) -> Control {
    // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    // memory gas calculated inside resize_offset
    try_or_fail!(machine.memory.resize_offset(start, len));
    machine.return_range = start..(start + len);
    Control::Exit(ExitSucceed::Returned.into())
}

#[inline]
pub fn revert<SPEC: Spec>(machine: &mut Machine) -> Control {
    enabled!(SPEC::has_revert);
    // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    // memory gas calculated inside resize_offset
    try_or_fail!(machine.memory.resize_offset(start, len));
    machine.return_range = start..(start + len);
    Control::Exit(ExitRevert::Reverted.into())
}
