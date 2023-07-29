use super::prelude::*;

pub(super) fn keccak256(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    pop!(interpreter, from, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::keccak256_cost(len as u64));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, from);
        memory_resize!(interpreter, from, len);
        crate::primitives::keccak256(interpreter.memory.get_slice(from, len))
    };

    push_b256!(interpreter, hash);
}

pub(super) fn address(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.address));
}

pub(super) fn caller(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.caller));
}

pub(super) fn codesize(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.bytecode.len()));
}

pub(super) fn codecopy(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    pop!(interpreter, memory_offset, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
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

pub(super) fn calldataload(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_saturated!(index);

    let load = if index < interpreter.contract.input.len() {
        let n = 32.min(interpreter.contract.input.len() - index);
        let mut bytes = [0u8; 32];
        bytes[..n].copy_from_slice(&interpreter.contract.input[index..index + n]);
        U256::from_be_bytes(bytes)
    } else {
        U256::ZERO
    };

    push!(interpreter, load);
}

pub(super) fn calldatasize(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.input.len()));
}

pub(super) fn callvalue(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, interpreter.contract.value);
}

pub(super) fn calldatacopy(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    pop!(interpreter, memory_offset, data_offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, data_offset, len, &interpreter.contract.input);
}

// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub(super) fn returndatasize(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    gas!(interpreter, gas::BASE);
    check!(interpreter, SpecId::enabled(spec, BYZANTIUM));
    push!(
        interpreter,
        U256::from(interpreter.return_data_buffer.len())
    );
}

// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub(super) fn returndatacopy(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, BYZANTIUM));
    pop!(interpreter, memory_offset, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    let data_offset = as_usize_saturated!(offset);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > interpreter.return_data_buffer.len() {
        interpreter.instruction_result = InstructionResult::OutOfOffset;
        return;
    }
    if len != 0 {
        let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
        memory_resize!(interpreter, memory_offset, len);
        interpreter.memory.set(
            memory_offset,
            &interpreter.return_data_buffer[data_offset..data_end],
        );
    }
}

pub(super) fn gas(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.gas.remaining()));
}
