use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{InputsTr, LegacyBytecode, MemoryTr, ReturnData, RuntimeFlag, StackTr},
    CallInput, InstructionContextTr, InstructionResult,
};
use core::ptr;
use primitives::{B256, KECCAK_EMPTY, U256};

/// Implements the KECCAK256 instruction.
///
/// Computes Keccak-256 hash of memory data.
#[inline]
pub fn keccak256<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn_top!([offset], top, context);
    let len = as_usize_or_fail!(context, top);
    gas_or_fail!(context, gas::keccak256_cost(len));
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(context, offset);
        resize_memory!(context, from, len);
        primitives::keccak256(context.memory().slice_len(from, len).as_ref())
    };
    *top = hash.into();
    InstructionReturn::cont()
}

/// Implements the ADDRESS instruction.
///
/// Pushes the current contract's address onto the stack.
#[inline]
pub fn address<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.input().target_address().into_word().into());
    InstructionReturn::cont()
}

/// Implements the CALLER instruction.
///
/// Pushes the caller's address onto the stack.
#[inline]
pub fn caller<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.input().caller_address().into_word().into());
    InstructionReturn::cont()
}

/// Implements the CODESIZE instruction.
///
/// Pushes the size of running contract's bytecode onto the stack.
#[inline]
pub fn codesize<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.bytecode().bytecode_len()));
    InstructionReturn::cont()
}

/// Implements the CODECOPY instruction.
///
/// Copies running contract's bytecode to memory.
#[inline]
pub fn codecopy<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn!([memory_offset, code_offset, len], context);
    let len = as_usize_or_fail!(context, len);
    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
    };
    let code_offset = as_usize_saturated!(code_offset);

    // Note: This can't panic because we resized memory to fit.
    let data = fuck_lt!(context.bytecode().bytecode_slice());
    context
        .memory()
        .set_data(memory_offset, code_offset, len, data);
    InstructionReturn::cont()
}

/// Implements the CALLDATALOAD instruction.
///
/// Loads 32 bytes of input data from the specified offset.
#[inline]
pub fn calldataload<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([], offset_ptr, context);
    let mut word = B256::ZERO;
    let offset = as_usize_saturated!(offset_ptr);
    let input_len = context.input().input().len();
    if offset < input_len {
        let count = 32.min(input_len - offset);

        // SAFETY: `count` is bounded by the calldata length.
        // This is `word[..count].copy_from_slice(input[offset..offset + count])`, written using
        // raw pointers as apparently the compiler cannot optimize the slice version, and using
        // `get_unchecked` twice is uglier.
        match fuck_lt_mut!(context).input().input() {
            CallInput::Bytes(bytes) => {
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr().add(offset), word.as_mut_ptr(), count)
                };
            }
            CallInput::SharedBuffer(range) => {
                let memory = context.memory();
                let input_slice = memory.global_slice(range.clone());
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
pub fn calldatasize<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.input().input().len()));
    InstructionReturn::cont()
}

/// Implements the CALLVALUE instruction.
///
/// Pushes the value sent with the current call onto the stack.
#[inline]
pub fn callvalue<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, context.input().call_value());
    InstructionReturn::cont()
}

/// Implements the CALLDATACOPY instruction.
///
/// Copies input data to memory.
#[inline]
pub fn calldatacopy<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn!([memory_offset, data_offset, len], context);
    let len = as_usize_or_fail!(context, len);
    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
    };

    let data_offset = as_usize_saturated!(data_offset);
    match fuck_lt_mut!(context).input().input() {
        CallInput::Bytes(bytes) => {
            context
                .memory()
                .set_data(memory_offset, data_offset, len, bytes.as_ref());
        }
        CallInput::SharedBuffer(range) => {
            context
                .memory()
                .set_data_from_global(memory_offset, data_offset, len, range.clone());
        }
    }
    InstructionReturn::cont()
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
#[inline]
pub fn returndatasize<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, BYZANTIUM);
    gas!(context, gas::BASE);
    push!(context, U256::from(context.return_data().buffer().len()));
    InstructionReturn::cont()
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
#[inline]
pub fn returndatacopy<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, BYZANTIUM);
    popn!([memory_offset, offset, len], context);

    let len = as_usize_or_fail!(context, len);
    let data_offset = as_usize_saturated!(offset);

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    let data_end = data_offset.saturating_add(len);
    if data_end > context.return_data().buffer().len() {
        return context.halt(InstructionResult::OutOfOffset);
    }

    let Some(memory_offset) = memory_resize(context, memory_offset, len) else {
        return InstructionReturn::cont();
    };

    // Note: This can't panic because we resized memory to fit.
    let data = fuck_lt!(context.return_data().buffer());
    context
        .memory()
        .set_data(memory_offset, data_offset, len, data);
    InstructionReturn::cont()
}

/// Implements the GAS instruction.
///
/// Pushes the amount of remaining gas onto the stack.
#[inline]
pub fn gas<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(context, U256::from(context.remaining_gas()));
    InstructionReturn::cont()
}

/// Common logic for copying data from a source buffer to the EVM's memory.
///
/// Handles memory expansion and gas calculation for data copy operations.
#[inline]
pub fn memory_resize<C: InstructionContextTr>(
    context: &mut C,
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
