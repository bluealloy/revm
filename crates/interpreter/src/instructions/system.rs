use crate::{
    gas,
    primitives::{Spec, B256, KECCAK_EMPTY, U256},
    Host, InstructionResult, Interpreter,
};
use core::ptr;

pub fn keccak256<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    pop_top!(interpreter, offset, len_ptr);
    let len = as_usize_or_fail!(interpreter, len_ptr);
    gas_or_fail!(interpreter, gas::keccak256_cost(len as u64));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, from, len);
        crate::primitives::keccak256(interpreter.shared_memory.slice(from, len))
    };
    *len_ptr = hash.into();
}

pub fn address<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, interpreter.contract.address.into_word());
}

pub fn caller<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, interpreter.contract.caller.into_word());
}

pub fn codesize<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.bytecode.len()));
}

pub fn codecopy<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    pop!(interpreter, memory_offset, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let code_offset = as_usize_saturated!(code_offset);
    resize_memory!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter.shared_memory.set_data(
        memory_offset,
        code_offset,
        len,
        interpreter.contract.bytecode.original_bytecode_slice(),
    );
}

pub fn calldataload<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, offset_ptr);
    let mut word = B256::ZERO;
    let offset = as_usize_saturated!(offset_ptr);
    if offset < interpreter.contract.input.len() {
        let count = 32.min(interpreter.contract.input.len() - offset);
        // SAFETY: count is bounded by the calldata length.
        // This is `word[..count].copy_from_slice(input[offset..offset + count])`, written using
        // raw pointers as apparently the compiler cannot optimize the slice version, and using
        // `get_unchecked` twice is uglier.
        debug_assert!(count <= 32 && offset + count <= interpreter.contract.input.len());
        unsafe {
            ptr::copy_nonoverlapping(
                interpreter.contract.input.as_ptr().add(offset),
                word.as_mut_ptr(),
                count,
            )
        };
    }
    *offset_ptr = word.into();
}

pub fn calldatasize<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.input.len()));
}

pub fn callvalue<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, interpreter.contract.value);
}

pub fn calldatacopy<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    pop!(interpreter, memory_offset, data_offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let data_offset = as_usize_saturated!(data_offset);
    resize_memory!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter.shared_memory.set_data(
        memory_offset,
        data_offset,
        len,
        &interpreter.contract.input,
    );
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatasize<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, BYZANTIUM);
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(interpreter.return_data_buffer.len())
    );
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatacopy<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, BYZANTIUM);
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
        resize_memory!(interpreter, memory_offset, len);
        interpreter.shared_memory.set(
            memory_offset,
            &interpreter.return_data_buffer[data_offset..data_end],
        );
    }
}

pub fn gas<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.gas.remaining()));
}
