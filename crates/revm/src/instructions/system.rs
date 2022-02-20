use crate::{gas, interpreter::Interpreter, Return};
use crate::{Spec, SpecId::*};
use bytes::Bytes;
use primitive_types::{H256, U256};

use sha3::{Digest, Keccak256};

pub fn sha3(machine: &mut Interpreter) -> Return {
    pop!(machine, from, len);
    gas_or_fail!(machine, gas::sha3_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    let data = if len == 0 {
        Bytes::new()
        // TODO optimization, we can return hardcoded value of keccak256:digest(&[])
    } else {
        let from = as_usize_or_fail!(from, Return::OutOfGas);
        memory_resize!(machine, from, len);
        Bytes::copy_from_slice(machine.memory.get_slice(from, len))
    };

    let ret = Keccak256::digest(data.as_ref());
    push_h256!(machine, H256::from_slice(ret.as_slice()));
    Return::Continue
}

pub fn address(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    let ret = H256::from(machine.contract.address);
    push_h256!(machine, ret);
    Return::Continue
}

pub fn caller(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    let ret = H256::from(machine.contract.caller);
    push_h256!(machine, ret);
    Return::Continue
}

pub fn codesize(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code_size);
    push!(machine, size);
    Return::Continue
}

pub fn codecopy(machine: &mut Interpreter) -> Return {
    pop!(machine, memory_offset, code_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(machine, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    machine
        .memory
        .set_data(memory_offset, code_offset, len, &machine.contract.code);
    Return::Continue
}

pub fn calldataload(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::VERYLOW);
    pop!(machine, index);
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
    push_h256!(machine, H256::from(load));
    Return::Continue
}

pub fn calldatasize(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    let len = U256::from(machine.contract.input.len());
    push!(machine, len);
    Return::Continue
}

pub fn callvalue(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    let mut ret = H256::default();
    machine.contract.value.to_big_endian(&mut ret[..]);
    push_h256!(machine, ret);
    Return::Continue
}

pub fn calldatacopy(machine: &mut Interpreter) -> Return {
    pop!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(machine, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    machine
        .memory
        .set_data(memory_offset, data_offset, len, &machine.contract.input);
    Return::Continue
}

pub fn returndatasize<SPEC: Spec>(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(SPEC::enabled(BYZANTINE));
    let size = U256::from(machine.return_data_buffer.len());
    push!(machine, size);
    Return::Continue
}

pub fn returndatacopy<SPEC: Spec>(machine: &mut Interpreter) -> Return {
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(SPEC::enabled(BYZANTINE));
    pop!(machine, memory_offset, offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let data_offset = as_usize_saturated!(offset);
    memory_resize!(machine, memory_offset, len);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > machine.return_data_buffer.len() {
        return Return::OutOfOffset;
    }
    machine.memory.set(
        memory_offset,
        &machine.return_data_buffer[data_offset..data_end],
    );
    Return::Continue
}

pub fn gas(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, U256::from(machine.gas.remaining()));
    machine.add_next_gas_block(machine.program_counter() - 1)
}
