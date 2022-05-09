use crate::{gas, interpreter::Interpreter, Return, Spec, SpecId::*, KECCAK_EMPTY};
use primitive_types::{H256, U256};

use sha3::{Digest, Keccak256};

pub fn sha3(interp: &mut Interpreter) -> Return {
    pop!(interp, from, len);
    gas_or_fail!(interp, gas::sha3_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    let h256 = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(from, Return::OutOfGas);
        memory_resize!(interp, from, len);
        H256::from_slice(Keccak256::digest(interp.memory.get_slice(from, len)).as_slice())
    };

    push_h256!(interp, h256);
    Return::Continue
}

pub fn address(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    let ret = H256::from(interp.contract.address);
    push_h256!(interp, ret);
    Return::Continue
}

pub fn caller(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    let ret = H256::from(interp.contract.caller);
    push_h256!(interp, ret);
    Return::Continue
}

pub fn codesize(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    let size = U256::from(interp.contract.code_size);
    push!(interp, size);
    Return::Continue
}

pub fn codecopy(interp: &mut Interpreter) -> Return {
    pop!(interp, memory_offset, code_offset, len);
    gas_or_fail!(interp, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(interp, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interp
        .memory
        .set_data(memory_offset, code_offset, len, &interp.contract.code);
    Return::Continue
}

pub fn calldataload(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    pop!(interp, index);
    let mut load = [0u8; 32];
    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
        if let Some(p) = index.checked_add(U256::from(i)) {
            if p <= U256::from(usize::MAX) {
                let p = p.as_usize();
                if p < interp.contract.input.len() {
                    load[i] = interp.contract.input[p];
                }
            }
        }
    }
    push_h256!(interp, H256::from(load));
    Return::Continue
}

pub fn calldatasize(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    let len = U256::from(interp.contract.input.len());
    push!(interp, len);
    Return::Continue
}

pub fn callvalue(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    let mut ret = H256::default();
    interp.contract.value.to_big_endian(&mut ret[..]);
    push_h256!(interp, ret);
    Return::Continue
}

pub fn calldatacopy(interp: &mut Interpreter) -> Return {
    pop!(interp, memory_offset, data_offset, len);
    gas_or_fail!(interp, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(interp, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interp
        .memory
        .set_data(memory_offset, data_offset, len, &interp.contract.input);
    Return::Continue
}

pub fn returndatasize<SPEC: Spec>(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(SPEC::enabled(BYZANTINE));
    let size = U256::from(interp.return_data_buffer.len());
    push!(interp, size);
    Return::Continue
}

pub fn returndatacopy<SPEC: Spec>(interp: &mut Interpreter) -> Return {
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(SPEC::enabled(BYZANTINE));
    pop!(interp, memory_offset, offset, len);
    gas_or_fail!(interp, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let data_offset = as_usize_saturated!(offset);
    memory_resize!(interp, memory_offset, len);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > interp.return_data_buffer.len() {
        return Return::OutOfOffset;
    }
    interp.memory.set(
        memory_offset,
        &interp.return_data_buffer[data_offset..data_end],
    );
    Return::Continue
}

pub fn gas(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, U256::from(interp.gas.remaining()));
    interp.add_next_gas_block(interp.program_counter() - 1)
}
