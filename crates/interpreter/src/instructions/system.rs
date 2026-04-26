use crate::{
    interpreter::{resize_memory, Interpreter},
    interpreter_types::{
        InputsTr, InterpreterTypes as ITy, LegacyBytecode, MemoryTr, ReturnData, RuntimeFlag,
        StackTr,
    },
    CallInput, InstructionExecResult as Result, InstructionResult,
};
use context_interface::{cfg::GasParams, Host};
use core::ptr;
use primitives::{B256, KECCAK_EMPTY, U256};

use crate::InstructionContext as Ictx;

/// Implements the KECCAK256 instruction.
///
/// Computes Keccak-256 hash of memory data.
pub fn keccak256<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([offset], top, context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, top);
    gas!(
        context.interpreter,
        context.host.gas_params().keccak256_cost(len)
    );
    let hash = if len == 0 {
        KECCAK_EMPTY
    } else {
        let from = as_usize_or_fail!(context.interpreter, offset);
        resize_memory(
            &mut context.interpreter.gas,
            &mut context.interpreter.memory,
            context.host.gas_params(),
            from,
            len,
        )?;
        primitives::keccak256(context.interpreter.memory.slice_len(from, len).as_ref())
    };
    *top = hash.into();
    Ok(())
}

/// Implements the ADDRESS instruction.
///
/// Pushes the current contract's address onto the stack.
pub fn address<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        context
            .interpreter
            .input
            .target_address()
            .into_word()
            .into()
    );
    Ok(())
}

/// Implements the CALLER instruction.
///
/// Pushes the caller's address onto the stack.
pub fn caller<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        context
            .interpreter
            .input
            .caller_address()
            .into_word()
            .into()
    );
    Ok(())
}

/// Implements the CODESIZE instruction.
///
/// Pushes the size of running contract's bytecode onto the stack.
pub fn codesize<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.bytecode_len())
    );
    Ok(())
}

/// Implements the CODECOPY instruction.
///
/// Copies running contract's bytecode to memory.
pub fn codecopy<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn!([memory_offset, code_offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    let Some(memory_offset) = copy_cost_and_memory_resize(
        context.interpreter,
        context.host.gas_params(),
        memory_offset,
        len,
    )?
    else {
        return Ok(());
    };
    let code_offset = as_usize_saturated!(code_offset);

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        context.interpreter.bytecode.bytecode_slice(),
    );
    Ok(())
}

/// Implements the CALLDATALOAD instruction.
///
/// Loads 32 bytes of input data from the specified offset.
pub fn calldataload<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([], offset_ptr, context.interpreter);
    let mut word = B256::ZERO;
    let offset = as_usize_saturated!(*offset_ptr);
    let input = context.interpreter.input.input();
    let input_len = input.len();
    if offset < input_len {
        let count = 32.min(input_len - offset);
        let input = &*input.as_bytes_memory(&context.interpreter.memory);
        // SAFETY: `count` is bounded by the calldata length.
        // This is `word[..count].copy_from_slice(input[offset..offset + count])`, written using
        // raw pointers as apparently the compiler cannot optimize the slice version, and using
        // `get_unchecked` twice is uglier.
        unsafe { ptr::copy_nonoverlapping(input.as_ptr().add(offset), word.as_mut_ptr(), count) };
    }
    *offset_ptr = word.into();
    Ok(())
}

/// Implements the CALLDATASIZE instruction.
///
/// Pushes the size of input data onto the stack.
pub fn calldatasize<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        U256::from(context.interpreter.input.input().len())
    );
    Ok(())
}

/// Implements the CALLVALUE instruction.
///
/// Pushes the value sent with the current call onto the stack.
pub fn callvalue<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(context.interpreter, context.interpreter.input.call_value());
    Ok(())
}

/// Implements the CALLDATACOPY instruction.
///
/// Copies input data to memory.
pub fn calldatacopy<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn!([memory_offset, data_offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    let Some(memory_offset) = copy_cost_and_memory_resize(
        context.interpreter,
        context.host.gas_params(),
        memory_offset,
        len,
    )?
    else {
        return Ok(());
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
    Ok(())
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatasize<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, BYZANTIUM);
    push!(
        context.interpreter,
        U256::from(context.interpreter.return_data.buffer().len())
    );
    Ok(())
}

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
pub fn returndatacopy<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, BYZANTIUM);
    popn!([memory_offset, offset, len], context.interpreter);

    let len = as_usize_or_fail!(context.interpreter, len);
    let data_offset = as_usize_saturated!(offset);

    // Old legacy behavior is to panic if data_end is out of scope of return buffer.
    let data_end = data_offset.saturating_add(len);
    if data_end > context.interpreter.return_data.buffer().len() {
        return Err(InstructionResult::OutOfOffset);
    }

    let Some(memory_offset) = copy_cost_and_memory_resize(
        context.interpreter,
        context.host.gas_params(),
        memory_offset,
        len,
    )?
    else {
        return Ok(());
    };

    // Note: This can't panic because we resized memory to fit.
    context.interpreter.memory.set_data(
        memory_offset,
        data_offset,
        len,
        context.interpreter.return_data.buffer(),
    );
    Ok(())
}

/// Implements the GAS instruction.
///
/// Pushes the amount of remaining gas onto the stack.
/// Returns `gas_left` only (excluding the state gas reservoir) per EIP-8037.
/// On mainnet (no state gas), this is equivalent to returning `remaining`.
pub fn gas<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    let gas = &context.interpreter.gas;
    push!(context.interpreter, U256::from(gas.remaining()));
    Ok(())
}

/// Common logic for copying data from a source buffer to the EVM's memory.
///
/// Handles memory expansion and gas calculation for data copy operations.
pub fn copy_cost_and_memory_resize(
    interpreter: &mut Interpreter<impl ITy>,
    gas_params: &GasParams,
    memory_offset: U256,
    len: usize,
) -> Result<Option<usize>, InstructionResult> {
    // Safe to cast usize to u64
    gas!(interpreter, gas_params.copy_cost(len));
    if len == 0 {
        return Ok(None);
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    interpreter.resize_memory(gas_params, memory_offset, len)?;

    Ok(Some(memory_offset))
}
