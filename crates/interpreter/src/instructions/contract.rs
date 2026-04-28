mod call_helpers;

pub use call_helpers::{
    get_memory_input_and_out_ranges, load_acc_and_calc_gas, load_account_delegated,
    load_account_delegated_handle_error, resize_memory,
};

use crate::{
    instructions::utility::IntoAddress,
    interpreter_action::FrameInput,
    interpreter_types::{
        InputsTr, InterpreterTypes as ITy, LoopControl, MemoryTr, RuntimeFlag, StackTr,
    },
    CallInput, CallInputs, CallScheme, CallValue, CreateInputs, Host,
    InstructionExecResult as Result, InstructionResult, InterpreterAction,
};
use context_interface::CreateScheme;
use primitives::{hardfork::SpecId, Bytes, U256};
use std::boxed::Box;

use crate::InstructionContext as Ictx;

/// Implements the CREATE/CREATE2 instruction.
///
/// Creates a new contract with provided bytecode.
pub fn create<const IS_CREATE2: bool, IT: ITy, H: Host + ?Sized>(
    context: Ictx<'_, H, IT>,
) -> Result {
    // Static call check is before gas charging (unlike execution-specs where it's
    // inside generic_create). This is safe because CREATE in a static context is
    // always an error regardless of gas accounting.
    require_non_staticcall!(context.interpreter);

    // EIP-1014: Skinny CREATE2
    if IS_CREATE2 {
        check!(context.interpreter, PETERSBURG);
    }

    popn!([value, code_offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);

    let mut code = Bytes::new();
    if len != 0 {
        // EIP-3860: Limit and meter initcode
        if context
            .interpreter
            .runtime_flag
            .spec_id()
            .is_enabled_in(SpecId::SHANGHAI)
        {
            // Limit is set as double of max contract bytecode size
            if len > context.host.max_initcode_size() {
                return Err(InstructionResult::CreateInitCodeSizeLimit);
            }
            gas!(
                context.interpreter,
                context.host.gas_params().initcode_cost(len)
            );
        }

        let code_offset = as_usize_or_fail!(context.interpreter, code_offset);
        context
            .interpreter
            .resize_memory(context.host.gas_params(), code_offset, len)?;

        code = Bytes::copy_from_slice(
            context
                .interpreter
                .memory
                .slice_len(code_offset, len)
                .as_ref(),
        );
    }

    // EIP-1014: Skinny CREATE2
    let scheme = if IS_CREATE2 {
        popn!([salt], context.interpreter);
        // SAFETY: `len` is reasonable in size as gas for it is already deducted.
        gas!(
            context.interpreter,
            context.host.gas_params().create2_cost(len)
        );
        CreateScheme::Create2 { salt }
    } else {
        gas!(context.interpreter, context.host.gas_params().create_cost());
        CreateScheme::Create
    };

    // State gas for account creation + contract metadata (EIP-8037)
    if context.host.is_amsterdam_eip8037_enabled() {
        state_gas!(
            context.interpreter,
            context.host.gas_params().create_state_gas()
        );
    }

    let mut gas_limit = context.interpreter.gas.remaining();

    // EIP-150: Gas cost changes for IO-heavy operations
    if context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(SpecId::TANGERINE)
    {
        // Take remaining gas and deduce l64 part of it.
        gas_limit = context.host.gas_params().call_stipend_reduction(gas_limit);
    }
    gas!(context.interpreter, gas_limit);

    // Call host to interact with target contract
    let create_inputs = CreateInputs::new(
        context.interpreter.input.target_address(),
        scheme,
        value,
        code,
        gas_limit,
        context.interpreter.gas.reservoir(),
    );
    context
        .interpreter
        .bytecode
        .set_action(InterpreterAction::NewFrame(FrameInput::Create(Box::new(
            create_inputs,
        ))));
    Err(InstructionResult::Suspend)
}

/// Implements the CALL, CALLCODE, DELEGATECALL, and STATICCALL instructions.
pub fn call<const KIND: u8, IT: ITy, H: Host + ?Sized>(mut context: Ictx<'_, H, IT>) -> Result {
    use bytecode::opcode::{CALL, CALLCODE, DELEGATECALL, STATICCALL};

    if KIND == DELEGATECALL {
        check!(context.interpreter, HOMESTEAD);
    } else if KIND == STATICCALL {
        check!(context.interpreter, BYZANTIUM);
    }

    let (local_gas_limit, to, value) = if matches!(KIND, CALL | CALLCODE) {
        popn!([local_gas_limit, to, value], context.interpreter);
        (local_gas_limit, to, value)
    } else {
        popn!([local_gas_limit, to], context.interpreter);
        (local_gas_limit, to, U256::ZERO)
    };
    let to = to.into_address();
    // Max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);
    let has_transfer = !value.is_zero();

    if KIND == CALL && context.interpreter.runtime_flag.is_static() && has_transfer {
        return Err(InstructionResult::CallNotAllowedInsideStatic);
    }

    let (input, return_memory_offset) =
        get_memory_input_and_out_ranges(context.interpreter, context.host.gas_params())?;

    let is_call = KIND == CALL;
    let (gas_limit, bytecode, bytecode_hash) =
        load_acc_and_calc_gas(&mut context, to, has_transfer, is_call, local_gas_limit)?;

    let target_address = if matches!(KIND, CALLCODE | DELEGATECALL) {
        context.interpreter.input.target_address()
    } else {
        to
    };
    let caller = if KIND == DELEGATECALL {
        context.interpreter.input.caller_address()
    } else {
        context.interpreter.input.target_address()
    };
    let value = if KIND == DELEGATECALL {
        CallValue::Apparent(context.interpreter.input.call_value())
    } else {
        CallValue::Transfer(value)
    };
    let scheme = match KIND {
        CALL => CallScheme::Call,
        CALLCODE => CallScheme::CallCode,
        DELEGATECALL => CallScheme::DelegateCall,
        STATICCALL => CallScheme::StaticCall,
        _ => unreachable!(),
    };
    let is_static = context.interpreter.runtime_flag.is_static() || KIND == STATICCALL;

    // Call host to interact with target contract
    context
        .interpreter
        .bytecode
        .set_action(InterpreterAction::NewFrame(FrameInput::Call(Box::new(
            CallInputs {
                input: CallInput::SharedBuffer(input),
                gas_limit,
                target_address,
                caller,
                bytecode_address: to,
                known_bytecode: (bytecode_hash, bytecode),
                value,
                scheme,
                is_static,
                return_memory_offset,
                reservoir: context.interpreter.gas.reservoir(),
            },
        ))));
    Err(InstructionResult::Suspend)
}
