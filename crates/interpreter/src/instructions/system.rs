use crate::{
    gas,
    interpreter::Interpreter,
    primitives::{keccak256, Spec, SpecId::*, B256, KECCAK_EMPTY, U256},
    Host, InstructionResult,
};
use core::cmp::min;

pub fn sha3(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, from, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);
    gas_or_fail!(interpreter, gas::sha3_cost(len as u64));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, from, InstructionResult::OutOfGas);
        memory_resize!(interpreter, from, len);
        keccak256(interpreter.memory.get_slice(from, len))
    };

    push_b256!(interpreter, hash);
}

pub fn address(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.address));
}

pub fn caller(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.caller));
}

pub fn codesize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.bytecode.len()));
}

pub fn codecopy(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, memory_offset, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset, InstructionResult::OutOfGas);
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        interpreter.contract.bytecode.original_bytecode_slice(),
    );
}

pub fn calldataload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_saturated!(index);

    let load = if index < interpreter.contract.input.len() {
        let have_bytes = min(interpreter.contract.input.len() - index, 32);
        let mut bytes = [0u8; 32];
        bytes[..have_bytes].copy_from_slice(&interpreter.contract.input[index..index + have_bytes]);
        B256(bytes)
    } else {
        B256::zero()
    };

    push_b256!(interpreter, load);
}

pub fn calldatasize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.input.len()));
}

pub fn callvalue(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, interpreter.contract.value);
}

pub fn calldatacopy(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, memory_offset, data_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset, InstructionResult::OutOfGas);
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, data_offset, len, &interpreter.contract.input);
}

pub fn returndatasize<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(interpreter, SPEC::enabled(BYZANTIUM));
    push!(
        interpreter,
        U256::from(interpreter.return_data_buffer.len())
    );
}

pub fn returndatacopy<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(interpreter, SPEC::enabled(BYZANTIUM));
    pop!(interpreter, memory_offset, offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    let data_offset = as_usize_saturated!(offset);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > interpreter.return_data_buffer.len() {
        interpreter.instruction_result = InstructionResult::OutOfOffset;
        return;
    }
    if len != 0 {
        let memory_offset =
            as_usize_or_fail!(interpreter, memory_offset, InstructionResult::OutOfGas);
        memory_resize!(interpreter, memory_offset, len);
        interpreter.memory.set(
            memory_offset,
            &interpreter.return_data_buffer[data_offset..data_end],
        );
    }
}

pub fn gas(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, U256::from(interpreter.gas.remaining()));
    if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        interpreter.instruction_result = ret;
    }
}
