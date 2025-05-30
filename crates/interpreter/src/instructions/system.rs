use crate::{
    gas,
    interpreter::Interpreter,
    interpreter_types::{
        InputsTr, InterpreterTypes, LegacyBytecode, MemoryTr, ReturnData, RuntimeFlag, StackTr,
    },
    CallInput, InstructionResult,
};
use core::ptr;
use primitives::{B256, KECCAK_EMPTY, U256};

use crate::InstructionContext;

pub fn keccak256<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    popn_top!([offset], top, context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, top);
    gas_or_fail!(context.interpreter, gas::keccak256_cost(len));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(context.interpreter, offset);
        resize_memory!(context.interpreter, from, len);
        primitives::keccak256(context.interpreter.memory.slice_len(from, len).as_ref())
    };
    *top = hash.into();
}

pub fn address<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        context
            .interpreter
            .input
            .target_address()
            .into_word()
            .into()
    );
}

pub fn caller<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        context
            .interpreter
            .input
            .caller_address()
            .into_word()
            .into()
    );
}

pub fn codesize<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.bytecode_len())
    );
}

pub fn codecopy<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    popn!([memory_offset, code_offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    let Some(memory_offset) = memory_resize(context.interpreter, memory_offset, len) else {
        return;
    };
    let code_offset = as_usize_saturated!(code_offset);

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        context.interpreter.bytecode.bytecode_slice(),
    );
}

pub fn calldataload<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::VERYLOW);
    //pop_top!(interpreter, offset_ptr);
    popn_top!([], offset_ptr, context.interpreter);
    let mut word = B256::ZERO;
    let offset = as_usize_saturated!(offset_ptr);
    let input = context.interpreter.input.input();
    let input_len = input.len();
    if offset < input_len {
        let count = 32.min(input_len - offset);

        // SAFETY: `count` is bounded by the calldata length.
        // This is `word[..count].copy_from_slice(input[offset..offset + count])`, written using
        // raw pointers as apparently the compiler cannot optimize the slice version, and using
        // `get_unchecked` twice is uglier.
        match context.interpreter.input.input() {
            CallInput::Bytes(bytes) => {
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr().add(offset), word.as_mut_ptr(), count)
                };
            }
            CallInput::SharedBuffer(range) => {
                let input_slice = context.interpreter.memory.global_slice(range.clone());
                unsafe {
                    ptr::copy_nonoverlapping(
                        input_slice.as_ptr().add(offset),
                        word.as_mut_ptr(),
                        count,
                    )
                };
            }
        }
    }
    *offset_ptr = word.into();
}

pub fn calldatasize<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.interpreter.input.input().len())
    );
}

pub fn callvalue<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.interpreter.input.call_value());
}

pub fn calldatacopy<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    popn!([memory_offset, data_offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    let Some(memory_offset) = memory_resize(context.interpreter, memory_offset, len) else {
        return;
    };

    let data_offset = as_usize_saturated!(data_offset);
    match context.interpreter.input.input() {
        CallInput::Bytes(bytes) => {
            context
                .interpreter
                .memory
                .set_data(memory_offset, data_offset, len, bytes.as_ref());
        }
        CallInput::SharedBuffer(range) => {
            context.interpreter.memory.set_data_from_global(
                memory_offset,
                data_offset,
                len,
                range.clone(),
            );
        }
    }
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatasize<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, BYZANTIUM);
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.interpreter.return_data.buffer().len())
    );
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatacopy<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, BYZANTIUM);
    popn!([memory_offset, offset, len], context.interpreter);

    let len = as_usize_or_fail!(context.interpreter, len);
    let data_offset = as_usize_saturated!(offset);

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    // This behavior is changed in EOF.
    let data_end = data_offset.saturating_add(len);
    if data_end > context.interpreter.return_data.buffer().len()
        && !context.interpreter.runtime_flag.is_eof()
    {
        context.interpreter.halt(InstructionResult::OutOfOffset);
        return;
    }

    let Some(memory_offset) = memory_resize(context.interpreter, memory_offset, len) else {
        return;
    };

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        data_offset,
        len,
        context.interpreter.return_data.buffer(),
    );
}

/// Part of EOF `<https://eips.ethereum.org/EIPS/eip-7069>`.
pub fn returndataload<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    require_eof!(context.interpreter);
    gas!(context.interpreter, gas::VERYLOW);
    popn_top!([], offset, context.interpreter);
    let offset_usize = as_usize_saturated!(offset);

    let mut output = [0u8; 32];
    if let Some(available) = context
        .interpreter
        .return_data
        .buffer()
        .len()
        .checked_sub(offset_usize)
    {
        let copy_len = available.min(32);
        output[..copy_len].copy_from_slice(
            &context.interpreter.return_data.buffer()[offset_usize..offset_usize + copy_len],
        );
    }

    *offset = B256::from(output).into();
}

pub fn gas<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.interpreter.gas.remaining())
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{instruction_table, interpreter_types::LoopControl};
    use bytecode::{opcode::RETURNDATACOPY, opcode::RETURNDATALOAD, Bytecode};
    use primitives::{bytes, Bytes};

    #[test]
    fn returndataload() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[
            RETURNDATALOAD,
            RETURNDATALOAD,
            RETURNDATALOAD,
            RETURNDATALOAD,
        ]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        interpreter.runtime_flag.is_eof = true;

        let _ = interpreter.stack.push(U256::from(0));
        interpreter.return_data.set_buffer(bytes!(
            "000000000000000400000000000000030000000000000002000000000000000100"
        ));

        interpreter.step_dummy(&table);
        assert_eq!(
            interpreter.stack.data(),
            &vec![U256::from_limbs([0x01, 0x02, 0x03, 0x04])]
        );

        let _ = interpreter.stack.pop();
        let _ = interpreter.stack.push(U256::from(1));

        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            interpreter.stack.data(),
            &vec![U256::from_limbs([0x0100, 0x0200, 0x0300, 0x0400])]
        );

        let _ = interpreter.stack.pop();
        let _ = interpreter.stack.push(U256::from(32));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            interpreter.stack.data(),
            &vec![U256::from_limbs([0x00, 0x00, 0x00, 0x00])]
        );

        // Offset right at the boundary of the return data buffer size
        let _ = interpreter.stack.pop();
        let _ = interpreter
            .stack
            .push(U256::from(interpreter.return_data.buffer().len()));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            interpreter.stack.data(),
            &vec![U256::from_limbs([0x00, 0x00, 0x00, 0x00])]
        );
    }

    #[test]
    fn returndatacopy() {
        let bytecode = Bytecode::new_raw(Bytes::from(&[
            RETURNDATACOPY,
            RETURNDATACOPY,
            RETURNDATACOPY,
            RETURNDATACOPY,
            RETURNDATACOPY,
            RETURNDATACOPY,
        ]));
        let mut interpreter = Interpreter::default().with_bytecode(bytecode);

        let table = instruction_table();
        interpreter.runtime_flag.is_eof = true;

        interpreter.return_data.set_buffer(bytes!(
            "000000000000000400000000000000030000000000000002000000000000000100"
        ));
        interpreter.memory.resize(256);

        // Copying within bounds
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter.stack.push(U256::from(0));
        let _ = interpreter.stack.push(U256::from(0));

        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            *interpreter.memory.slice(0..32),
            interpreter.return_data.buffer()[0..32]
        );

        // Copying with partial out-of-bounds (should zero pad)
        let _ = interpreter.stack.push(U256::from(64));
        let _ = interpreter.stack.push(U256::from(16));
        let _ = interpreter.stack.push(U256::from(64));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            *interpreter.memory.slice(64..80),
            interpreter.return_data.buffer()[16..32]
        );
        assert_eq!(*interpreter.memory.slice(80..128), [0u8; 48]);

        // Completely out-of-bounds (should be all zeros)
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter.stack.push(U256::from(96));
        let _ = interpreter.stack.push(U256::from(128));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(*interpreter.memory.slice(128..160), [0u8; 32]);

        // Large offset
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter.stack.push(U256::MAX);
        let _ = interpreter.stack.push(U256::from(0));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(*interpreter.memory.slice(0..32), [0u8; 32]);

        // Offset just before the boundary of the return data buffer size
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter
            .stack
            .push(U256::from(interpreter.return_data.buffer().len() - 32));
        let _ = interpreter.stack.push(U256::from(0));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(
            *interpreter.memory.slice(0..32),
            interpreter.return_data.buffer()[interpreter.return_data.buffer().len() - 32..]
        );

        // Offset right at the boundary of the return data buffer size
        let _ = interpreter.stack.push(U256::from(32));
        let _ = interpreter
            .stack
            .push(U256::from(interpreter.return_data.buffer().len()));
        let _ = interpreter.stack.push(U256::from(0));
        interpreter.step_dummy(&table);
        assert!(interpreter.bytecode.is_not_end());
        assert_eq!(*interpreter.memory.slice(0..32), [0u8; 32]);
    }
}
