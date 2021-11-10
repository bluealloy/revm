use super::{gas, Control};
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed},
    machine::Machine,
    Spec,
    SpecId::*,
};
use primitive_types::{H256, U256};

#[inline(always)]
pub fn codesize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code.len());
    push_u256!(machine, size);
    Control::Continue
}

#[inline(always)]
pub fn codecopy(machine: &mut Machine) -> Control {
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

#[inline(always)]
pub fn calldatasize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    push_u256!(machine, len);
    Control::Continue
}

#[inline(always)]
pub fn calldatacopy(machine: &mut Machine) -> Control {
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
pub fn pop(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    pop!(machine, _val);
    Control::Continue
}

#[inline(always)]
pub fn mload(machine: &mut Machine) -> Control {
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
pub fn mstore(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index);
    pop!(machine, value);

    memory_resize!(machine, index, U256::from(32));
    let index = as_usize_or_fail!(index);
    try_or_fail!(machine.memory.set(index, &value[..], Some(32)));
    Control::Continue
}

#[inline(always)]
pub fn mstore8(machine: &mut Machine) -> Control {
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
pub fn jump(machine: &mut Machine) -> Control {
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
pub fn jumpi(machine: &mut Machine) -> Control {
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
pub fn jumpdest(machine: &mut Machine) -> Control {
    gas!(machine, gas::JUMPDEST);
    Control::Continue
}

#[inline(always)]
pub fn pc(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, U256::from(machine.program_counter));
    Control::Continue
}

#[inline(always)]
pub fn msize(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, machine.memory.effective_len());
    Control::Continue
}

// code padding is needed for contracts
#[inline(always)]
pub fn push<const N: usize>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);
    let position = machine.program_counter+1;
    let slice = &machine.contract.code[position..position + N];

    try_or_fail!(machine.stack.push_slice::<N>(slice));
    Control::ContinueN(N + 1)
}

#[inline(always)]
pub fn dup<const N: usize>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);

    try_or_fail!(machine.stack.dup::<N>());
    Control::Continue
}

#[inline(always)]
pub fn swap<const N: usize>(machine: &mut Machine) -> Control {
    gas!(machine, gas::VERYLOW);
    try_or_fail!(machine.stack.swap::<N>());
    Control::Continue
}

#[inline(always)]
pub fn ret(machine: &mut Machine) -> Control {
    // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Control::Exit(ExitSucceed::Returned.into())
}

#[inline(always)]
pub fn revert<SPEC: Spec>(machine: &mut Machine) -> Control {
    check!(SPEC::enabled(BYZANTINE)); // EIP-140: REVERT instruction
                                      // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Control::Exit(ExitRevert::Reverted.into())
}
