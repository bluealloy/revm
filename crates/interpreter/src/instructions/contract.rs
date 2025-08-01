mod call_helpers;
pub use call_helpers::{calc_call_gas, get_memory_input_and_out_ranges, resize_memory};

use crate::{
    gas,
    instructions::{utility::IntoAddress, InstructionReturn},
    interpreter_action::FrameInput,
    interpreter_types::{InputsTr, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, Host, InstructionContextTr,
    InstructionResult, InterpreterAction,
};
use context_interface::CreateScheme;
use primitives::{hardfork::SpecId, Address, Bytes, B256, U256};
use std::boxed::Box;

/// Implements the CREATE/CREATE2 instruction.
///
/// Creates a new contract with provided bytecode.
#[inline]
pub fn create<const IS_CREATE2: bool, C: InstructionContextTr>(
    context: &mut C,
) -> InstructionReturn {
    require_non_staticcall!(context);

    // EIP-1014: Skinny CREATE2
    if IS_CREATE2 {
        check!(context, PETERSBURG);
    }

    popn!([value, code_offset, len], context);
    let len = as_usize_or_fail!(context, len);

    let mut code = Bytes::new();
    if len != 0 {
        // EIP-3860: Limit and meter initcode
        if context
            .runtime_flag()
            .spec_id()
            .is_enabled_in(SpecId::SHANGHAI)
        {
            // Limit is set as double of max contract bytecode size
            if len > context.host().max_initcode_size() {
                return context.halt(InstructionResult::CreateInitCodeSizeLimit);
            }
            gas!(context, gas::initcode_cost(len));
        }

        let code_offset = as_usize_or_fail!(context, code_offset);
        resize_memory!(context, code_offset, len);
        code = Bytes::copy_from_slice(context.memory().slice_len(code_offset, len).as_ref());
    }

    // EIP-1014: Skinny CREATE2
    let scheme = if IS_CREATE2 {
        popn!([salt], context);
        // SAFETY: `len` is reasonable in size as gas for it is already deducted.
        gas_or_fail!(context, gas::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        gas!(context, gas::CREATE);
        CreateScheme::Create
    };

    let mut gas_limit = context.remaining_gas();

    // EIP-150: Gas cost changes for IO-heavy operations
    if context
        .runtime_flag()
        .spec_id()
        .is_enabled_in(SpecId::TANGERINE)
    {
        // Take remaining gas and deduce l64 part of it.
        gas_limit -= gas_limit / 64
    }
    gas!(context, gas_limit);

    // Call host to interact with target contract
    let action = InterpreterAction::NewFrame(FrameInput::Create(Box::new(CreateInputs {
        caller: context.input().target_address(),
        scheme,
        value,
        init_code: code,
        gas_limit,
    })));
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}

/// Implements the CALL instruction.
///
/// Message call with value transfer to another account.
#[inline]
pub fn call<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn!([local_gas_limit, to, value], context);
    let to = to.into_address();
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let has_transfer = !value.is_zero();
    if context.runtime_flag().is_static() && has_transfer {
        return context.halt(InstructionResult::CallNotAllowedInsideStatic);
    }

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(context) else {
        return InstructionReturn::halt();
    };

    let Some(account_load) = context.host().load_account_delegated(to) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    let Some(mut gas_limit) = calc_call_gas(context, account_load, has_transfer, local_gas_limit)
    else {
        return InstructionReturn::halt();
    };

    gas!(context, gas_limit);

    // Add call stipend if there is value to be transferred.
    if has_transfer {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    let target_address = context.input().target_address();
    let action = InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
        input: CallInput::SharedBuffer(input),
        gas_limit,
        target_address: to,
        caller: target_address,
        bytecode_address: to,
        value: CallValue::Transfer(value),
        scheme: CallScheme::Call,
        is_static: context.runtime_flag().is_static(),
        return_memory_offset,
    })));
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}

/// Implements the CALLCODE instruction.
///
/// Message call with alternative account's code.
#[inline]
pub fn call_code<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    popn!([local_gas_limit, to, value], context);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    //pop!(context, value);
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(context) else {
        return InstructionReturn::halt();
    };

    let Some(mut load) = context.host().load_account_delegated(to) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    // Set `is_empty` to false as we are not creating this account.
    load.is_empty = false;
    let Some(mut gas_limit) = calc_call_gas(context, load, !value.is_zero(), local_gas_limit)
    else {
        return InstructionReturn::halt();
    };

    gas!(context, gas_limit);

    // Add call stipend if there is value to be transferred.
    if !value.is_zero() {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    let target_address = context.input().target_address();
    let action = InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
        input: CallInput::SharedBuffer(input),
        gas_limit,
        target_address,
        caller: target_address,
        bytecode_address: to,
        value: CallValue::Transfer(value),
        scheme: CallScheme::CallCode,
        is_static: context.runtime_flag().is_static(),
        return_memory_offset,
    })));
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}

/// Implements the DELEGATECALL instruction.
///
/// Message call with alternative account's code but same sender and value.
#[inline]
pub fn delegate_call<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, HOMESTEAD);
    popn!([local_gas_limit, to], context);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(context) else {
        return InstructionReturn::halt();
    };

    let Some(mut load) = context.host().load_account_delegated(to) else {
        return context.halt(InstructionResult::FatalExternalError);
    };

    // Set is_empty to false as we are not creating this account.
    load.is_empty = false;
    let Some(gas_limit) = calc_call_gas(context, load, false, local_gas_limit) else {
        return InstructionReturn::halt();
    };

    gas!(context, gas_limit);

    // Call host to interact with target contract
    let caller = context.input().caller_address();
    let call_value = context.input().call_value();
    let is_static = context.runtime_flag().is_static();
    let action = InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
        input: CallInput::SharedBuffer(input),
        gas_limit,
        target_address: context.input().target_address(),
        caller,
        bytecode_address: to,
        value: CallValue::Apparent(call_value),
        scheme: CallScheme::DelegateCall,
        is_static,
        return_memory_offset,
    })));
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}

/// Implements the STATICCALL instruction.
///
/// Static message call (cannot modify state).
#[inline]
pub fn static_call<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, BYZANTIUM);
    popn!([local_gas_limit, to], context);
    let to = Address::from_word(B256::from(to));
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(context) else {
        return InstructionReturn::halt();
    };

    let Some(mut load) = context.host().load_account_delegated(to) else {
        return context.halt(InstructionResult::FatalExternalError);
    };
    // Set `is_empty` to false as we are not creating this account.
    load.is_empty = false;
    let Some(gas_limit) = calc_call_gas(context, load, false, local_gas_limit) else {
        return InstructionReturn::halt();
    };
    gas!(context, gas_limit);

    // Call host to interact with target contract
    let action = InterpreterAction::NewFrame(FrameInput::Call(Box::new(CallInputs {
        input: CallInput::SharedBuffer(input),
        gas_limit,
        target_address: to,
        caller: context.input().target_address(),
        bytecode_address: to,
        value: CallValue::Transfer(U256::ZERO),
        scheme: CallScheme::StaticCall,
        is_static: true,
        return_memory_offset,
    })));
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}
