use crate::{
    common::keccak256, gas, interpreter::Interpreter, Return, Spec, SpecId::*, KECCAK_EMPTY,
};
use primitive_types::H256;
use ruint::aliases::U256;
use std::cmp::min;

pub fn sha3(interp: &mut Interpreter) -> Return {
    pop!(interp, from, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    gas_or_fail!(interp, gas::sha3_cost(len as u64));
    let h256 = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(from, Return::OutOfGas);
        memory_resize!(interp, from, len);
        keccak256(interp.memory.get_slice(from, len))
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
    let size = U256::from(interp.contract.bytecode.len());
    push!(interp, size);
    Return::Continue
}

pub fn codecopy(interp: &mut Interpreter) -> Return {
    pop!(interp, memory_offset, code_offset, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    gas_or_fail!(interp, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(interp, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interp.memory.set_data(
        memory_offset,
        code_offset,
        len,
        interp.contract.bytecode.original_bytecode_slice(),
    );
    Return::Continue
}

pub fn calldataload(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::VERYLOW);
    pop!(interp, index);
    let index = as_usize_saturated!(index);

    let load = if index < interp.contract.input.len() {
        let mut load = H256::zero();
        let have_bytes = min(interp.contract.input.len() - index, 32);
        load.0[..have_bytes].copy_from_slice(&interp.contract.input[index..index + have_bytes]);
        load
    } else {
        H256::zero()
    };

    push_h256!(interp, load);
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
    push_h256!(interp, interp.contract.value.to_be_bytes().into());
    Return::Continue
}

pub fn calldatacopy(interp: &mut Interpreter) -> Return {
    pop!(interp, memory_offset, data_offset, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    gas_or_fail!(interp, gas::verylowcopy_cost(len as u64));
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
    check!(SPEC::enabled(BYZANTIUM));
    let size = U256::from(interp.return_data_buffer.len());
    push!(interp, size);
    Return::Continue
}

pub fn returndatacopy<SPEC: Spec>(interp: &mut Interpreter) -> Return {
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(SPEC::enabled(BYZANTIUM));
    pop!(interp, memory_offset, offset, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    gas_or_fail!(interp, gas::verylowcopy_cost(len as u64));
    let data_offset = as_usize_saturated!(offset);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > interp.return_data_buffer.len() {
        return Return::OutOfOffset;
    }
    if len != 0 {
        let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
        memory_resize!(interp, memory_offset, len);
        interp.memory.set(
            memory_offset,
            &interp.return_data_buffer[data_offset..data_end],
        );
    }
    Return::Continue
}

pub fn gas(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, U256::from(interp.gas.remaining()));
    interp.add_next_gas_block(interp.program_counter() - 1)
}
