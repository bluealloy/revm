use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{
        InputsTrait, InterpreterTypes, LegacyBytecode, LoopControl, MemoryTrait, ReturnData,
        RuntimeFlag, StackTrait,
    },
    Host, InstructionResult,
};
use core::ptr;
use primitives::{B256, KECCAK_EMPTY, U256};

pub fn keccak256<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn_top!([offset], top, interpreter);
    let len = as_usize_or_fail!(interpreter, top);
    gas_or_fail!(interpreter, gas::keccak256_cost(len));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, from, len);
        primitives::keccak256(interpreter.memory.slice_len(from, len).as_ref())
    };
    *top = hash.into();
}

pub fn address<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        interpreter.input.target_address().into_word().into()
    );
}

pub fn caller<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        interpreter.input.caller_address().into_word().into()
    );
}

pub fn codesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.bytecode.bytecode_len()));
}

pub fn codecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([memory_offset, code_offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    let Some(memory_offset) = memory_resize(interpreter, memory_offset, len) else {
        return;
    };
    let code_offset = as_usize_saturated!(code_offset);

    // Note: This can't panic because we resized memory to fit.
    interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        interpreter.bytecode.bytecode_slice(),
    );
}

pub fn calldataload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::VERYLOW);
    //pop_top!(interpreter, offset_ptr);
    popn_top!([], offset_ptr, interpreter);
    let mut word = B256::ZERO;
    let offset = as_usize_saturated!(offset_ptr);
    let input = interpreter.input.input();
    let input_len = input.len();
    if offset < input_len {
        let count = 32.min(input_len - offset);
        // SAFETY: `count` is bounded by the calldata length.
        // This is `word[..count].copy_from_slice(input[offset..offset + count])`, written using
        // raw pointers as apparently the compiler cannot optimize the slice version, and using
        // `get_unchecked` twice is uglier.
        debug_assert!(count <= 32 && offset + count <= input_len);
        unsafe { ptr::copy_nonoverlapping(input.as_ptr().add(offset), word.as_mut_ptr(), count) };
    }
    *offset_ptr = word.into();
}

pub fn calldatasize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.input.input().len()));
}

pub fn callvalue<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, interpreter.input.call_value());
}

pub fn calldatacopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn!([memory_offset, data_offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    let Some(memory_offset) = memory_resize(interpreter, memory_offset, len) else {
        return;
    };

    let data_offset = as_usize_saturated!(data_offset);
    // Note: This can't panic because we resized memory to fit.
    interpreter
        .memory
        .set_data(memory_offset, data_offset, len, interpreter.input.input());
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatasize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    check!(interpreter, BYZANTIUM);
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(interpreter.return_data.buffer().len())
    );
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatacopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    check!(interpreter, BYZANTIUM);
    popn!([memory_offset, offset, len], interpreter);

    let len = as_usize_or_fail!(interpreter, len);
    let data_offset = as_usize_saturated!(offset);

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    // This behavior is changed in EOF.
    let data_end = data_offset.saturating_add(len);
    if data_end > interpreter.return_data.buffer().len() && !interpreter.runtime_flag.is_eof() {
        interpreter
            .control
            .set_instruction_result(InstructionResult::OutOfOffset);
        return;
    }

    let Some(memory_offset) = memory_resize(interpreter, memory_offset, len) else {
        return;
    };

    // Note: This can't panic because we resized memory to fit.
    interpreter.memory.set_data(
        memory_offset,
        data_offset,
        len,
        interpreter.return_data.buffer(),
    );
}

/// Part of EOF `<https://eips.ethereum.org/EIPS/eip-7069>`.
pub fn returndataload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    require_eof!(interpreter);
    gas!(interpreter, gas::VERYLOW);
    popn_top!([], offset, interpreter);
    let offset_usize = as_usize_saturated!(offset);

    let mut output = [0u8; 32];
    if let Some(available) = interpreter
        .return_data
        .buffer()
        .len()
        .checked_sub(offset_usize)
    {
        let copy_len = available.min(32);
        output[..copy_len].copy_from_slice(
            &interpreter.return_data.buffer()[offset_usize..offset_usize + copy_len],
        );
    }

    *offset = B256::from(output).into();
}

pub fn gas<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(interpreter.control.gas().remaining())
    );
}

// common logic for copying data from a source buffer to the EVM's memory
pub fn memory_resize(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    memory_offset: U256,
    len: usize,
) -> Option<usize> {
    // Safe to cast usize to u64
    gas_or_fail!(interpreter, gas::copy_cost_verylow(len), None);
    if len == 0 {
        return None;
    }
    let memory_offset = as_usize_or_fail_ret!(interpreter, memory_offset, None);
    resize_memory!(interpreter, memory_offset, len, None);

    Some(memory_offset)
}

// TODO : Tests
/*
#[cfg(test)]
mod test {
    use super::*;
    use crate::{table::make_instruction_table, DummyHost, Gas, InstructionResult};
    use bytecode::opcode::{RETURNDATACOPY, RETURNDATALOAD};
    use bytecode::Bytecode;
    use primitives::bytes;
    use specification::hardfork::{PragueSpec, SpecId};
    use context_interface::DefaultEthereumWiring;

    #[test]
    fn returndataload() {
        let table = make_instruction_table::<Interpreter, DummyHost<DefaultEthereumWiring>>();
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
        interp.spec_id = SpecId::PRAGUE;
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
        let table = make_instruction_table::<Interpreter, _>();
        let mut host = DummyHost::<DefaultEthereumWiring>::default();

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
        interp.spec_id = SpecId::PRAGUE;
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

*/
