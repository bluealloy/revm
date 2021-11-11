use super::gas;
use crate::{
    machine::Machine,
    Return, Spec,
    SpecId::*,
};
use primitive_types::{H256, U256};

#[inline(always)]
pub fn codesize(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code_size);
    push_u256!(machine, size);
    Return::Continue
}

#[inline(always)]
pub fn codecopy(machine: &mut Machine) -> Return {
    pop_u256!(machine, memory_offset, code_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .copy_large(memory_offset, code_offset, len, &machine.contract.code)
}

#[inline(always)]
pub fn calldataload(machine: &mut Machine) -> Return {
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
    Return::Continue
}

#[inline(always)]
pub fn calldatasize(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    push_u256!(machine, len);
    Return::Continue
}

#[inline(always)]
pub fn calldatacopy(machine: &mut Machine) -> Return {
    pop_u256!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);

    if len == U256::zero() {
        return Return::Continue;
    }

    machine
        .memory
        .copy_large(memory_offset, data_offset, len, &machine.contract.input)
}

#[inline(always)]
pub fn pop(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);
    pop!(machine, _val);
    Return::Continue
}

#[inline(always)]
pub fn mload(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);
    pop_u256!(machine, index);

    // memory aditional gas checked here
    memory_resize!(machine, index, U256::from(32));
    let index = as_usize_or_fail!(index);
    let value = H256::from_slice(&machine.memory.get(index, 32)[..]);
    push!(machine, value);
    Return::Continue
}

#[inline(always)]
pub fn mstore(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index);
    pop!(machine, value);

    memory_resize!(machine, index, U256::from(32));
    let index = as_usize_or_fail!(index);
    machine.memory.set(index, &value[..], Some(32))
}

#[inline(always)]
pub fn mstore8(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);

    pop_u256!(machine, index, value);

    // memory aditional gas checked here
    memory_resize!(machine, index, U256::one());
    let index = as_usize_or_fail!(index);
    let value = (value.low_u32() & 0xff) as u8;
    machine.memory.set(index, &[value], Some(1))
}

#[inline(always)]
pub fn jump(machine: &mut Machine) -> Return {
    gas!(machine, gas::MID);

    pop_u256!(machine, dest);
    let dest = as_usize_or_fail!(dest, Return::InvalidJump);

    if machine.contract.is_valid_jump(dest) {
        machine.program_counter = dest;
        Return::Continue
    } else {
        Return::InvalidJump
    }
}

#[inline(always)]
pub fn jumpi(machine: &mut Machine) -> Return {
    gas!(machine, gas::HIGH);

    pop_u256!(machine, dest);
    pop!(machine, value);

    if value != H256::zero() {
        let dest = as_usize_or_fail!(dest, Return::InvalidJump);
        if machine.contract.is_valid_jump(dest) {
            machine.program_counter = dest;
            Return::Continue
        } else {
            Return::InvalidJump
        }
    } else {
        Return::Continue
    }
}

#[inline(always)]
pub fn jumpdest(machine: &mut Machine) -> Return {
    gas!(machine, gas::JUMPDEST);
    Return::Continue
}

#[inline(always)]
pub fn pc(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);
    push_u256!(machine, U256::from(machine.program_counter - 1));
    Return::Continue
}

#[inline(always)]
pub fn msize(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);
    push_u256!(machine, machine.memory.effective_len());
    Return::Continue
}

// code padding is needed for contracts
#[inline(always)]
pub fn push<const N: usize>(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);
    let slice = &machine.contract.code[machine.program_counter..machine.program_counter + N];

    try_or_fail!(machine.stack.push_slice::<N>(slice));
    machine.program_counter += N;
    Return::Continue
}

#[inline(always)]
pub fn dup<const N: usize>(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);

    machine.stack.dup::<N>()
}

#[inline(always)]
pub fn swap<const N: usize>(machine: &mut Machine) -> Return {
    gas!(machine, gas::VERYLOW);
    machine.stack.swap::<N>()
}

#[inline(always)]
pub fn ret(machine: &mut Machine) -> Return {
    // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Return::Return
}

#[inline(always)]
pub fn revert<SPEC: Spec>(machine: &mut Machine) -> Return {
    check!(SPEC::enabled(BYZANTINE)); // EIP-140: REVERT instruction
                                      // zero gas cost gas!(machine,gas::ZERO);
    pop_u256!(machine, start, len);
    memory_resize!(machine, start, len);
    machine.return_range = start..(start + len);
    Return::Revert
}
