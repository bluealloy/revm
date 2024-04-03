use crate::{
    gas,
    primitives::{Spec, B256, KECCAK_EMPTY, U256},
    Host, InstructionResult, Interpreter,
};

pub fn keccak256<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    pop!(interpreter, from, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::keccak256_cost(len as u64));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, from);
        resize_memory!(interpreter, from, len);
        crate::primitives::keccak256(interpreter.shared_memory.slice(from, len))
    };

    push_b256!(interpreter, hash);
}

pub fn address<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, interpreter.contract.target_address.into_word());
}

pub fn caller<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, interpreter.contract.caller.into_word());
}

pub fn codesize<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    panic_on_eof!(interpreter);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.bytecode.len()));
}

pub fn codecopy<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    panic_on_eof!(interpreter);
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
        &interpreter.contract.bytecode.original_bytes(),
    );
}

pub fn calldataload<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop!(interpreter, offset);
    let offset = as_usize_saturated!(offset);
    let load = if offset < interpreter.contract.input.len() {
        let have_bytes = 32.min(interpreter.contract.input.len() - offset);
        let mut bytes = [0u8; 32];
        bytes[..have_bytes]
            .copy_from_slice(&interpreter.contract.input[offset..offset + have_bytes]);
        B256::new(bytes)
    } else {
        B256::ZERO
    };

    push_b256!(interpreter, load);
}

pub fn calldatasize<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.contract.input.len()));
}

pub fn callvalue<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, interpreter.contract.call_value);
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
    let data_end = data_offset.saturating_add(len);
    if data_end > interpreter.return_data_buffer.len() {
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

/// Part of EOF https://eips.ethereum.org/EIPS/eip-7069
pub fn returndataload<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    error_on_disabled_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, offset);
    let offset_usize = as_usize_or_fail!(interpreter, offset);
    if offset_usize.saturating_add(32) > interpreter.return_data_buffer.len() {
        // TODO(EOF) proper error.
        interpreter.instruction_result = InstructionResult::OutOfOffset;
        return;
    }
    *offset =
        B256::from_slice(&interpreter.return_data_buffer[offset_usize..offset_usize + 32]).into();
}

pub fn gas<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    panic_on_eof!(interpreter);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.gas.remaining()));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        opcode::{make_instruction_table, RETURNDATALOAD},
        primitives::{bytes, Bytecode, PragueSpec, U256},
        DummyHost, Gas, Interpreter,
    };

    #[test]
    fn returndataload() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::default();

        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(
            [RETURNDATALOAD, RETURNDATALOAD, RETURNDATALOAD].into(),
        ));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);

        interp.stack.push(U256::from(0)).unwrap();
        interp.return_data_buffer =
            bytes!("000000000000000400000000000000030000000000000002000000000000000100");
        interp.step(&table, &mut host);
        assert_eq!(
            interp.stack.data(),
            &vec![U256::from_limbs([0x01, 0x02, 0x03, 0x04])]
        );

        let _ = interp.stack.pop();
        let _ = interp.stack.push(U256::from(1));

        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.stack.data(),
            &vec![U256::from_limbs([0x0100, 0x0200, 0x0300, 0x0400])]
        );

        let _ = interp.stack.pop();
        let _ = interp.stack.push(U256::from(2));
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::OutOfOffset);
    }
}
