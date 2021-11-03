use super::{gas, Control};
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed},
    machine::Machine,
    Spec,
    SpecId::*,
};
use core::cmp::min;
use primitive_types::{H256, U256};

#[inline(always)]
pub fn codesize<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code.len());
    push_u256!(machine, size);
    Control::Continue
}

#[inline(always)]
pub fn codecopy<S: Spec>(machine: &mut Machine) -> Control {
    pop_u256!(machine, memory_offset, code_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);

    try_or_fail!(machine.memory.copy_large(
        memory_offset,
        code_offset,
        len,
        &machine.contract.code
    ));
    Control::Continue
}

#[inline(always)]
pub fn calldataload<S: Spec>(machine: &mut Machine) -> Control {
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

#[inline(always)]
pub fn calldatasize<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    push_u256!(machine, len);
    Control::Continue
}

#[inline(always)]
pub fn calldatacopy<S: Spec>(machine: &mut Machine) -> Control {
    pop_u256!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);

    if len == U256::zero() {
        return Control::Continue;
    }

    try_or_fail!(machine.memory.copy_large(
        memory_offset,
        data_offset,
        len,
        &machine.contract.input
    ));
    Control::Continue
}

#[inline(always)]
pub fn pop<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    pop!(machine, _val);
    Control::Continue
}

#[inline(always)]
pub fn mload<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);
    pop_u256!(machine, index);

    // memory aditional gas checked here
    memory_resize!(machine, index, U256::from(32));
    let index = as_usize_or_fail!(index);
    let value = H256::from_slice(&machine.memory.get(index, 32)[..]);
    push!(machine, value);
    Control::Continue
}

#[inline(always)]
pub fn mstore<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index);
    pop!(machine, value);

    memory_resize!(machine, index, U256::from(32));
    let index = as_usize_or_fail!(index);
    try_or_fail!(machine.memory.set(index, &value[..], Some(32)));
    Control::Continue
}

#[inline(always)]
pub fn mstore8<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index, value);

    // memory aditional gas checked here
    memory_resize!(machine, index, U256::one());
    let index = as_usize_or_fail!(index);
    let value = (value.low_u32() & 0xff) as u8;
    try_or_fail!(machine.memory.set(index, &[value], Some(1)));
    Control::Continue
}

#[inline(always)]
pub fn jump<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::MID);

    pop_u256!(machine, dest);
    let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

    if machine.contract.is_valid_jump(dest) {
        Control::Jump(dest)
    } else {
        Control::Exit(ExitError::InvalidJump.into())
    }
}

#[inline(always)]
pub fn jumpi<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::HIGH);

    pop_u256!(machine, dest);
    pop!(machine, value);

    if value != H256::zero() {
        let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
        if machine.contract.is_valid_jump(dest) {
            Control::Jump(dest)
        } else {
            Control::Exit(ExitError::InvalidJump.into())
        }
    } else {
        Control::Continue
    }
}

#[inline(always)]
pub fn jumpdest<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::JUMPDEST);
    Control::Continue
}

#[inline(always)]
pub fn pc<S: Spec>(machine: &mut Machine, position: usize) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, U256::from(position));
    Control::Continue
}

#[inline(always)]
pub fn msize<S: Spec>(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, machine.memory.effective_len());
    Control::Continue
}

#[inline(always)]
pub fn push<S: Spec>(machine: &mut Machine, n: usize, position: usize) -> Control {
    gas!(machine, gas::VERYLOW);
    let end = min(position + 1 + n, machine.contract.code.len());
    let slice = &machine.contract.code[(position + 1)..end];
    let mut val = [0u8; 32];
    val[(32 - slice.len())..32].copy_from_slice(slice);

    push!(machine, H256(val));
    Control::ContinueN(1 + n)
}

#[inline(always)]
pub fn dup<S: Spec>(machine: &mut Machine, n: usize) -> Control {
    gas!(machine, gas::VERYLOW);

    let value = try_or_fail!(machine.stack.peek(n - 1));
    push!(machine, value);
    Control::Continue
}

#[inline(always)]
pub fn swap<S: Spec>(machine: &mut Machine, n: usize) -> Control {
    gas!(machine, gas::VERYLOW);

    let val1 = try_or_fail!(machine.stack.peek(0));
    let val2 = try_or_fail!(machine.stack.peek(n));
    try_or_fail!(machine.stack.set(0, val2));
    try_or_fail!(machine.stack.set(n, val1));
    Control::Continue
}

#[inline(always)]
pub fn ret<S: Spec>(machine: &mut Machine) -> Control {
    // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Control::Exit(ExitSucceed::Returned.into())
}

#[inline(always)]
pub fn revert<S: Spec>(machine: &mut Machine) -> Control {
    check!(S::enabled(BYZANTINE)); // EIP-140: REVERT instruction
                                      // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Control::Exit(ExitRevert::Reverted.into())
}
