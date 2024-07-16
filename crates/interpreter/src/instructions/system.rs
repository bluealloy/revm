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
    push_b256!(interpreter, interpreter.contract.target_address.into_word());
}

pub fn caller<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, interpreter.contract.caller.into_word());
}

pub fn codesize<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
    assume!(!interpreter.contract.bytecode.is_eof());
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

    // Inform the optimizer that the bytecode cannot be EOF to remove a bounds check.
    assume!(!interpreter.contract.bytecode.is_eof());
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

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    // This behavior is changed in EOF.
    if data_end > interpreter.return_data_buffer.len() && !interpreter.is_eof {
        interpreter.instruction_result = InstructionResult::OutOfOffset;
        return;
    }

    // if len is zero memory is not resized.
    if len == 0 {
        return;
    }

    // resize memory
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    resize_memory!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter.shared_memory.set_data(
        memory_offset,
        data_offset,
        len,
        &interpreter.return_data_buffer,
    );
}

/// Part of EOF `<https://eips.ethereum.org/EIPS/eip-7069>`.
pub fn returndataload<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, offset);
    let offset_usize = as_usize_saturated!(offset);

    let mut output = [0u8; 32];
    if let Some(available) = interpreter
        .return_data_buffer
        .len()
        .checked_sub(offset_usize)
    {
        let copy_len = available.min(32);
        output[..copy_len].copy_from_slice(
            &interpreter.return_data_buffer[offset_usize..offset_usize + copy_len],
        );
    }

    *offset = B256::from(output).into();
}

pub fn gas<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.gas.remaining()));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        opcode::{make_instruction_table, RETURNDATACOPY, RETURNDATALOAD},
        primitives::{bytes, Bytecode, PragueSpec},
        DummyHost, Gas, InstructionResult,
    };

    #[test]
    fn returndataload() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::default();

        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(
            [
                RETURNDATALOAD,
                RETURNDATALOAD,
                RETURNDATALOAD,
                RETURNDATALOAD,
            ]
            .into(),
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
        let _ = interp.stack.push(U256::from(32));
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.stack.data(),
            &vec![U256::from_limbs([0x00, 0x00, 0x00, 0x00])]
        );

        // Offset right at the boundary of the return data buffer size
        let _ = interp.stack.pop();
        let _ = interp
            .stack
            .push(U256::from(interp.return_data_buffer.len()));
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.stack.data(),
            &vec![U256::from_limbs([0x00, 0x00, 0x00, 0x00])]
        );
    }

    #[test]
    fn returndatacopy() {
        let table = make_instruction_table::<_, PragueSpec>();
        let mut host = DummyHost::default();

        let mut interp = Interpreter::new_bytecode(Bytecode::LegacyRaw(
            [
                RETURNDATACOPY,
                RETURNDATACOPY,
                RETURNDATACOPY,
                RETURNDATACOPY,
                RETURNDATACOPY,
                RETURNDATACOPY,
            ]
            .into(),
        ));
        interp.is_eof = true;
        interp.gas = Gas::new(10000);

        interp.return_data_buffer =
            bytes!("000000000000000400000000000000030000000000000002000000000000000100");
        interp.shared_memory.resize(256);

        // Copying within bounds
        interp.stack.push(U256::from(32)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.shared_memory.slice(0, 32),
            &interp.return_data_buffer[0..32]
        );

        // Copying with partial out-of-bounds (should zero pad)
        interp.stack.push(U256::from(64)).unwrap();
        interp.stack.push(U256::from(16)).unwrap();
        interp.stack.push(U256::from(64)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.shared_memory.slice(64, 16),
            &interp.return_data_buffer[16..32]
        );
        assert_eq!(&interp.shared_memory.slice(80, 48), &[0u8; 48]);

        // Completely out-of-bounds (should be all zeros)
        interp.stack.push(U256::from(32)).unwrap();
        interp.stack.push(U256::from(96)).unwrap();
        interp.stack.push(U256::from(128)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(&interp.shared_memory.slice(128, 32), &[0u8; 32]);

        // Large offset
        interp.stack.push(U256::from(32)).unwrap();
        interp.stack.push(U256::MAX).unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(&interp.shared_memory.slice(0, 32), &[0u8; 32]);

        // Offset just before the boundary of the return data buffer size
        interp.stack.push(U256::from(32)).unwrap();
        interp
            .stack
            .push(U256::from(interp.return_data_buffer.len() - 32))
            .unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(
            interp.shared_memory.slice(0, 32),
            &interp.return_data_buffer[interp.return_data_buffer.len() - 32..]
        );

        // Offset right at the boundary of the return data buffer size
        interp.stack.push(U256::from(32)).unwrap();
        interp
            .stack
            .push(U256::from(interp.return_data_buffer.len()))
            .unwrap();
        interp.stack.push(U256::from(0)).unwrap();
        interp.step(&table, &mut host);
        assert_eq!(interp.instruction_result, InstructionResult::Continue);
        assert_eq!(&interp.shared_memory.slice(0, 32), &[0u8; 32]);
    }
}
