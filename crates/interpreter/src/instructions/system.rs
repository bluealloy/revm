use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{
        InputsTr, InterpreterTypes, LegacyBytecode, MemoryTr, ReturnData, RuntimeFlag, StackTr,
    },
    CallInput, InstructionResult,
};
use core::ptr;
use primitives::{B256, KECCAK_EMPTY, U256};

use crate::InstructionContext;

/// Implements the KECCAK256 instruction.
///
/// Computes Keccak-256 hash of memory data.
#[inline]
pub fn keccak256<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    popn_top!([offset], top, context);
    let len = as_usize_or_fail!(context, top);
    gas_or_fail!(context, gas::keccak256_cost(len));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(context, offset);
        resize_memory!(context, from, len);
        primitives::keccak256(context.interpreter.memory.slice_len(from, len).as_ref())
    };
    *top = hash.into();
    InstructionReturn::cont()
}

/// Implements the ADDRESS instruction.
///
/// Pushes the current contract's address onto the stack.
#[inline]
pub fn address<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        context
            .interpreter
            .input
            .target_address()
            .into_word()
            .into()
    );
    InstructionReturn::cont()
}

/// Implements the CALLER instruction.
///
/// Pushes the caller's address onto the stack.
#[inline]
pub fn caller<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        context
            .interpreter
            .input
            .caller_address()
            .into_word()
            .into()
    );
    InstructionReturn::cont()
}

/// Implements the CODESIZE instruction.
///
/// Pushes the size of running contract's bytecode onto the stack.
#[inline]
pub fn codesize<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        U256::from(context.interpreter.bytecode.bytecode_len())
    );
    InstructionReturn::cont()
}

/// Implements the CODECOPY instruction.
///
/// Copies running contract's bytecode to memory.
#[inline]
pub fn codecopy<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    popn!([memory_offset, code_offset, len], context);
    let len = as_usize_or_fail!(context, len);
    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
    };
    let code_offset = as_usize_saturated!(code_offset);

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        context.interpreter.bytecode.bytecode_slice(),
    );
    InstructionReturn::cont()
}

/// Implements the CALLDATALOAD instruction.
///
/// Loads 32 bytes of input data from the specified offset.
#[inline]
pub fn calldataload<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([], offset_ptr, context);
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
    InstructionReturn::cont()
}

/// Implements the CALLDATASIZE instruction.
///
/// Pushes the size of input data onto the stack.
#[inline]
pub fn calldatasize<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.interpreter.input.input().len()));
    InstructionReturn::cont()
}

/// Implements the CALLVALUE instruction.
///
/// Pushes the value sent with the current call onto the stack.
#[inline]
pub fn callvalue<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.interpreter.input.call_value());
    InstructionReturn::cont()
}

/// Implements the CALLDATACOPY instruction.
///
/// Copies input data to memory.
#[inline]
pub fn calldatacopy<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    popn!([memory_offset, data_offset, len], context);
    let len = as_usize_or_fail!(context, len);
    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
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
    InstructionReturn::cont()
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
#[inline]
pub fn returndatasize<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, BYZANTIUM);
    gas!(context, gas::BASE);
    push!(
        context,
        U256::from(context.interpreter.return_data.buffer().len())
    );
    InstructionReturn::cont()
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
#[inline]
pub fn returndatacopy<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, BYZANTIUM);
    popn!([memory_offset, offset, len], context);

    let len = as_usize_or_fail!(context, len);
    let data_offset = as_usize_saturated!(offset);

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    let data_end = data_offset.saturating_add(len);
    if data_end > context.interpreter.return_data.buffer().len() {
        context.halt(InstructionResult::OutOfOffset);
        return InstructionReturn::halt();
    }

    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
    };

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        data_offset,
        len,
        context.interpreter.return_data.buffer(),
    );
    InstructionReturn::cont()
}

/// Implements the GAS instruction.
///
/// Pushes the amount of remaining gas onto the stack.
#[inline]
pub fn gas<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.interpreter.gas.remaining()));
    InstructionReturn::cont()
}

/// Common logic for copying data from a source buffer to the EVM's memory.
///
/// Handles memory expansion and gas calculation for data copy operations.
#[inline]
pub fn memory_resize<H: ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
    memory_offset: U256,
    len: usize,
) -> Option<usize> {
    // Safe to cast usize to u64
    gas_or_fail!(context, gas::copy_cost_verylow(len), None);
    if len == 0 {
        return None;
    }
    let memory_offset = as_usize_or_fail_ret!(context, memory_offset, None);
    resize_memory!(context, memory_offset, len, None);

    Some(memory_offset)
}
