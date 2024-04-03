mod call_helpers;

pub use call_helpers::{
    calc_call_gas, get_memory_input_and_out_ranges, resize_memory_and_return_range,
};
use revm_primitives::{keccak256, BerlinSpec};

use crate::{
    gas::{self, cost_per_word, BASE, EOF_CREATE_GAS, KECCAK256WORD},
    interpreter::{Interpreter, InterpreterAction},
    primitives::{Address, Bytes, Eof, Spec, SpecId::*, B256, U256},
    CallInputs, CallScheme, CreateInputs, CreateScheme, EOFCreateInput, Host, InstructionResult,
    LoadAccountResult, TransferValue, MAX_INITCODE_SIZE,
};
use core::{cmp::max, ops::Range};
use std::boxed::Box;

pub fn resize_memory(
    interpreter: &mut Interpreter,
    offset: U256,
    len: U256,
) -> Option<Range<usize>> {
    let len = as_usize_or_fail_ret!(interpreter, len, None);
    if len != 0 {
        let offset = as_usize_or_fail_ret!(interpreter, offset, None);
        resize_memory!(interpreter, offset, len, None);
        // range is checked in resize_memory! macro and it is bounded by usize.
        Some(offset..offset + len)
    } else {
        //unrealistic value so we are sure it is not used
        Some(usize::MAX..usize::MAX)
    }
}

pub fn eofcreate<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    error_on_disabled_eof!(interpreter);
    gas!(interpreter, EOF_CREATE_GAS);
    let initcontainer_index = unsafe { *interpreter.instruction_pointer };
    pop!(interpreter, value, salt, data_offset, data_size);

    let Some(sub_container) = interpreter
        .eof()
        .expect("EOF is set")
        .body
        .container_section
        .get(initcontainer_index as usize)
        .cloned()
    else {
        // TODO(EOF) handle error
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    // resize memory and get return range.
    let Some(return_range) = resize_memory(interpreter, data_offset, data_size) else {
        return;
    };

    let eof = Eof::decode(sub_container.clone()).expect("Subcontainer is verified");

    if !eof.body.is_data_filled {
        // should be always false as it is verified by eof verification.
        panic!("Panic if data section is not full");
    }

    // deduct gas for hash that is needed to calculate address.
    gas_or_fail!(
        interpreter,
        cost_per_word::<KECCAK256WORD>(sub_container.len() as u64)
    );

    let created_address = interpreter
        .contract
        .caller
        .create2(salt.to_be_bytes(), keccak256(sub_container));

    // Send container for execution container is preverified.
    interpreter.next_action = InterpreterAction::EOFCreate {
        inputs: Box::new(EOFCreateInput::new(
            interpreter.contract.target_address,
            created_address,
            value,
            eof,
            interpreter.gas().remaining(),
            return_range,
        )),
    };

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

pub fn txcreate<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    error_on_disabled_eof!(interpreter);
    gas!(interpreter, EOF_CREATE_GAS);
    pop!(
        interpreter,
        tx_initcode_hash,
        value,
        salt,
        data_offset,
        data_size
    );
    // TODO(EOF) check when memory resize should be done
    let Some(return_range) = resize_memory(interpreter, data_offset, data_size) else {
        return;
    };

    let tx_initcode_hash = B256::from(tx_initcode_hash);

    // TODO(EOF) get initcode from TxEnv.
    let initcode = Bytes::new();

    // deduct gas for validation
    gas_or_fail!(interpreter, cost_per_word::<BASE>(initcode.len() as u64));

    // TODO check if data container is full
    let Ok(eof) = Eof::decode(initcode.clone()) else {
        push!(interpreter, U256::ZERO);
        return;
    };

    // Data section should be full, push zero to stack and return if not.
    if !eof.body.is_data_filled {
        push!(interpreter, U256::ZERO);
        return;
    }

    // TODO(EOF) validate initcode, we should do this only once and cache result.

    // deduct gas for hash.
    gas_or_fail!(
        interpreter,
        cost_per_word::<KECCAK256WORD>(initcode.len() as u64)
    );

    // Create new address. Gas for it is already deducted.
    let created_address = interpreter
        .contract
        .caller
        .create2(salt.to_be_bytes(), tx_initcode_hash);

    let gas_limit = interpreter.gas().remaining();
    // spend all gas. It will be reimbursed after frame returns.
    gas!(interpreter, gas_limit);

    interpreter.next_action = InterpreterAction::EOFCreate {
        inputs: Box::new(EOFCreateInput::new(
            interpreter.contract.target_address,
            created_address,
            value,
            eof,
            gas_limit,
            return_range,
        )),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn return_contract<H: Host + ?Sized>(interpreter: &mut Interpreter, _host: &mut H) {
    error_on_disabled_eof!(interpreter);
}

pub fn extcall_input(interpreter: &mut Interpreter) -> Option<Bytes> {
    pop_ret!(interpreter, input_offset, input_size, None);

    let return_memory_offset =
        resize_memory_and_return_range(interpreter, input_offset, input_size)?;

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

    if load_result.is_cold {
        gas!(interpreter, gas::COLD_ACCOUNT_ACCESS_COST, None);
    }

    // TODO(EOF) is_empty should only be checked on delegatecall
    let call_cost =
        gas::call_cost::<BerlinSpec>(transfers_value, load_result.is_cold, load_result.is_empty);
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

    // TODO check remaining gas more then N

    gas!(interpreter, gas_limit, None);
    Some(gas_limit)
}

pub fn extcall<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
    pop_address!(interpreter, target_address);

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
            value: TransferValue::Value(value),
            scheme: CallScheme::Call,
            is_static: interpreter.is_static,
            is_eof: true,
            return_memory_offset: 0..0,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn extdcall<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
    pop_address!(interpreter, target_address);

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
            value: TransferValue::ApparentValue(interpreter.contract.call_value),
            // TODO(EOF) should be EofDelegateCall?
            scheme: CallScheme::DelegateCall,
            is_static: interpreter.is_static,
            is_eof: true,
            return_memory_offset: 0..0,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn extscall<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
    pop_address!(interpreter, target_address);

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
            value: TransferValue::Value(U256::ZERO),
            scheme: CallScheme::Call,
            is_static: interpreter.is_static,
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
    panic_on_eof!(interpreter);
    error_on_static_call!(interpreter);

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
        gas_or_fail!(interpreter, gas::create2_cost(len));
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
    panic_on_eof!(interpreter);
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
    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
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
            value: TransferValue::Value(value),
            scheme: CallScheme::Call,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call_code<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
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

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
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
            value: TransferValue::Value(value),
            scheme: CallScheme::CallCode,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn delegate_call<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
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
        calc_call_gas::<H, SPEC>(interpreter, is_cold, false, false, local_gas_limit)
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
            value: TransferValue::ApparentValue(interpreter.contract.call_value),
            scheme: CallScheme::DelegateCall,
            is_static: interpreter.is_static,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn static_call<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
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
        calc_call_gas::<H, SPEC>(interpreter, is_cold, false, false, local_gas_limit)
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
            value: TransferValue::Value(U256::ZERO),
            scheme: CallScheme::StaticCall,
            is_static: true,
            is_eof: false,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}
