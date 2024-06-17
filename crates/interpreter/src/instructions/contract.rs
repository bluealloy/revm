mod call_helpers;

pub use call_helpers::{calc_call_gas, get_memory_input_and_out_ranges, resize_memory};
use revm_primitives::{keccak256, BerlinSpec};

use crate::{
    gas::{self, cost_per_word, EOF_CREATE_GAS, KECCAK256WORD},
    interpreter::Interpreter,
    primitives::{Address, Bytes, Eof, Spec, SpecId::*, U256},
    CallInputs, CallScheme, CallValue, CreateInputs, CreateScheme, EOFCreateInputs, Host,
    InstructionResult, InterpreterAction, InterpreterResult, LoadAccountResult, MAX_INITCODE_SIZE,
};
use core::cmp::max;
use std::boxed::Box;

/// EOF Create instruction
pub fn eofcreate<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_eof!(interpreter);
    gas!(interpreter, EOF_CREATE_GAS);
    let initcontainer_index = unsafe { *interpreter.instruction_pointer };
    pop!(interpreter, value, salt, data_offset, data_size);

    let sub_container = interpreter
        .eof()
        .expect("EOF is set")
        .body
        .container_section
        .get(initcontainer_index as usize)
        .cloned()
        .expect("EOF is checked");

    // resize memory and get return range.
    let Some(input_range) = resize_memory(interpreter, data_offset, data_size) else {
        return;
    };

    let input = if !input_range.is_empty() {
        interpreter
            .shared_memory
            .slice_range(input_range)
            .to_vec()
            .into()
    } else {
        Bytes::new()
    };

    let eof = Eof::decode(sub_container.clone()).expect("Subcontainer is verified");

    if !eof.body.is_data_filled {
        // should be always false as it is verified by eof verification.
        panic!("Panic if data section is not full");
    }

    // deduct gas for hash that is needed to calculate address.
    gas_or_fail!(
        interpreter,
        cost_per_word(sub_container.len() as u64, KECCAK256WORD)
    );

    let created_address = interpreter
        .contract
        .target_address
        .create2(salt.to_be_bytes(), keccak256(sub_container));

    let gas_limit = interpreter.gas().remaining_63_of_64_parts();
    gas!(interpreter, gas_limit);
    // Send container for execution container is preverified.
    interpreter.instruction_result = InstructionResult::CallOrCreate;
    interpreter.next_action = InterpreterAction::EOFCreate {
        inputs: Box::new(EOFCreateInputs::new(
            interpreter.contract.target_address,
            created_address,
            value,
            eof,
            gas_limit,
            input,
        )),
    };

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

pub fn return_contract<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    require_init_eof!(interpreter);
    let deploy_container_index = unsafe { *interpreter.instruction_pointer };
    pop!(interpreter, aux_data_offset, aux_data_size);
    let aux_data_size = as_usize_or_fail!(interpreter, aux_data_size);
    // important: offset must be ignored if len is zeros
    let container = interpreter
        .eof()
        .expect("EOF is set")
        .body
        .container_section
        .get(deploy_container_index as usize)
        .expect("EOF is checked");

    // convert to EOF so we can check data section size.
    let new_eof = Eof::decode(container.clone()).expect("Container is verified");

    let aux_slice = if aux_data_size != 0 {
        let aux_data_offset = as_usize_or_fail!(interpreter, aux_data_offset);
        resize_memory!(interpreter, aux_data_offset, aux_data_size);

        interpreter
            .shared_memory
            .slice(aux_data_offset, aux_data_size)
    } else {
        &[]
    };

    let new_data_size = new_eof.body.data_section.len() + aux_slice.len();
    if new_data_size > 0xFFFF {
        // aux data is too big
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    if new_data_size < new_eof.header.data_size as usize {
        // aux data is too small
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }

    // append data bytes
    let output = [new_eof.raw(), aux_slice].concat().into();

    let result = InstructionResult::ReturnContract;
    interpreter.instruction_result = result;
    interpreter.next_action = crate::InterpreterAction::Return {
        result: InterpreterResult {
            output,
            gas: interpreter.gas,
            result,
        },
    };
}

pub fn extcall_input(interpreter: &mut Interpreter) -> Option<Bytes> {
    pop_ret!(interpreter, input_offset, input_size, None);

    let return_memory_offset = resize_memory(interpreter, input_offset, input_size)?;

    if return_memory_offset.is_empty() {
        return Some(Bytes::new());
    }

    Some(Bytes::copy_from_slice(
        interpreter
            .shared_memory
            .slice_range(return_memory_offset.clone()),
    ))
}

pub fn extcall_gas_calc<H: Host + ?Sized>(
    interpreter: &mut Interpreter,
    host: &mut H,
    target: Address,
    transfers_value: bool,
) -> Option<u64> {
    let Some(load_result) = host.load_account(target) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return None;
    };

    // TODO(EOF) is_empty should only be checked on delegatecall
    let call_cost = gas::call_cost(
        BerlinSpec::SPEC_ID,
        transfers_value,
        load_result.is_cold,
        load_result.is_empty,
    );
    gas!(interpreter, call_cost, None);

    // 7. Calculate the gas available to callee as caller’s
    // remaining gas reduced by max(ceil(gas/64), MIN_RETAINED_GAS) (MIN_RETAINED_GAS is 5000).
    let gas_reduce = max(interpreter.gas.remaining() / 64, 5000);
    let gas_limit = interpreter.gas().remaining().saturating_sub(gas_reduce);

    if gas_limit < 2300 {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        // TODO(EOF) error;
        // interpreter.instruction_result = InstructionResult::CallGasTooLow;
        return None;
    }

    gas!(interpreter, gas_limit, None);
    Some(gas_limit)
}

pub fn extcall<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    require_eof!(interpreter);
    pop_address!(interpreter, target_address);

    // TODO check if target is left paddded with zeroes.

    // input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    pop!(interpreter, value);
    let has_transfer = value != U256::ZERO;

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, has_transfer) else {
        return;
    };
    // TODO Check if static and value 0

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address,
            caller: interpreter.contract.target_address,
            bytecode_address: target_address,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: interpreter.is_static,
            is_eof: true,
            return_memory_offset: 0..0,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn extdelegatecall<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    require_eof!(interpreter);
    pop_address!(interpreter, target_address);

    // TODO check if target is left paddded with zeroes.

    // input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, false) else {
        return;
    };
    // TODO Check if static and value 0

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address,
            caller: interpreter.contract.target_address,
            bytecode_address: target_address,
            value: CallValue::Apparent(interpreter.contract.call_value),
            // TODO(EOF) should be EofDelegateCall?
            scheme: CallScheme::DelegateCall,
            is_static: interpreter.is_static,
            is_eof: true,
            return_memory_offset: 0..0,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn extstaticcall<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    require_eof!(interpreter);
    pop_address!(interpreter, target_address);

    // TODO check if target is left paddded with zeroes.

    // input call
    let Some(input) = extcall_input(interpreter) else {
        return;
    };

    let Some(gas_limit) = extcall_gas_calc(interpreter, host, target_address, false) else {
        return;
    };

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address,
            caller: interpreter.contract.target_address,
            bytecode_address: target_address,
            value: CallValue::Transfer(U256::ZERO),
            scheme: CallScheme::Call,
            is_static: true,
            is_eof: true,
            return_memory_offset: 0..0,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn create<const IS_CREATE2: bool, H: Host + ?Sized, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut H,
) {
    require_non_staticcall!(interpreter);

    // EIP-1014: Skinny CREATE2
    if IS_CREATE2 {
        check!(interpreter, PETERSBURG);
    }

    pop!(interpreter, value, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len);

    let mut code = Bytes::new();
    if len != 0 {
        // EIP-3860: Limit and meter initcode
        if SPEC::enabled(SHANGHAI) {
            // Limit is set as double of max contract bytecode size
            let max_initcode_size = host
                .env()
                .cfg
                .limit_contract_code_size
                .map(|limit| limit.saturating_mul(2))
                .unwrap_or(MAX_INITCODE_SIZE);
            if len > max_initcode_size {
                interpreter.instruction_result = InstructionResult::CreateInitCodeSizeLimit;
                return;
            }
            gas!(interpreter, gas::initcode_cost(len as u64));
        }

        let code_offset = as_usize_or_fail!(interpreter, code_offset);
        resize_memory!(interpreter, code_offset, len);
        code = Bytes::copy_from_slice(interpreter.shared_memory.slice(code_offset, len));
    }

    // EIP-1014: Skinny CREATE2
    let scheme = if IS_CREATE2 {
        pop!(interpreter, salt);
        // SAFETY: len is reasonable in size as gas for it is already deducted.
        gas_or_fail!(interpreter, gas::create2_cost(len.try_into().unwrap()));
        CreateScheme::Create2 { salt }
    } else {
        gas!(interpreter, gas::CREATE);
        CreateScheme::Create
    };

    let mut gas_limit = interpreter.gas().remaining();

    // EIP-150: Gas cost changes for IO-heavy operations
    if SPEC::enabled(TANGERINE) {
        // take remaining gas and deduce l64 part of it.
        gas_limit -= gas_limit / 64
    }
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Create {
        inputs: Box::new(CreateInputs {
            caller: interpreter.contract.target_address,
            scheme,
            value,
            init_code: code,
            gas_limit,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    let has_transfer = value != U256::ZERO;
    if interpreter.is_static && has_transfer {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        return;
    }

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(LoadAccountResult { is_cold, is_empty }) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    let Some(mut gas_limit) = calc_call_gas::<SPEC>(
        interpreter,
        is_cold,
        has_transfer,
        is_empty,
        local_gas_limit,
    ) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // add call stipend if there is value to be transferred.
    if has_transfer {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address: to,
            caller: interpreter.contract.target_address,
            bytecode_address: to,
            value: CallValue::Transfer(value),
            scheme: CallScheme::Call,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call_code<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(LoadAccountResult { is_cold, .. }) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(mut gas_limit) = calc_call_gas::<SPEC>(
        interpreter,
        is_cold,
        value != U256::ZERO,
        false,
        local_gas_limit,
    ) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // add call stipend if there is value to be transferred.
    if value != U256::ZERO {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address: interpreter.contract.target_address,
            caller: interpreter.contract.target_address,
            bytecode_address: to,
            value: CallValue::Transfer(value),
            scheme: CallScheme::CallCode,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn delegate_call<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, HOMESTEAD);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(LoadAccountResult { is_cold, .. }) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    let Some(gas_limit) =
        calc_call_gas::<SPEC>(interpreter, is_cold, false, false, local_gas_limit)
    else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address: interpreter.contract.target_address,
            caller: interpreter.contract.caller,
            bytecode_address: to,
            value: CallValue::Apparent(interpreter.contract.call_value),
            scheme: CallScheme::DelegateCall,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn static_call<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, BYZANTIUM);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(LoadAccountResult { is_cold, .. }) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(gas_limit) =
        calc_call_gas::<SPEC>(interpreter, is_cold, false, false, local_gas_limit)
    else {
        return;
    };
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            input,
            gas_limit,
            target_address: to,
            caller: interpreter.contract.target_address,
            bytecode_address: to,
            value: CallValue::Transfer(U256::ZERO),
            scheme: CallScheme::StaticCall,
            is_static: true,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}
