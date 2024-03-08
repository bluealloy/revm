mod call_helpers;

pub use call_helpers::{
    calc_call_gas, get_memory_input_and_out_ranges, resize_memory_and_return_range,
};
use revm_primitives::keccak256;

use crate::{
    gas::{self, cost_per_word, BASE, EOF_CREATE_GAS, KECCAK256WORD},
    interpreter::{Interpreter, InterpreterAction},
    primitives::{Address, Bytes, Eof, Spec, SpecId::*, B256, U256},
    CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme, EOFCreateInput, Host,
    InstructionResult, Transfer, MAX_INITCODE_SIZE,
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
        shared_memory_resize!(interpreter, offset, len, None);
        // range is checked in shared_memory_resize! macro and it is bounded by usize.
        Some(offset..offset + len)
    } else {
        //unrealistic value so we are sure it is not used
        Some(usize::MAX..usize::MAX)
    }
}

pub fn eofcreate<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
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
            interpreter.contract.address,
            created_address,
            value,
            eof,
            interpreter.gas().remaining(),
            return_range,
        )),
    };

    interpreter.instruction_pointer = unsafe { interpreter.instruction_pointer.offset(1) };
}

pub fn txcreate<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
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
            interpreter.contract.address,
            created_address,
            value,
            eof,
            gas_limit,
            return_range,
        )),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn return_contract<H: Host>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
}

pub fn extcall<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
    panic_on_eof!(interpreter);
    pop_address!(interpreter, to);
    pop!(interpreter, input_offset, input_size, value);
    if interpreter.is_static && value != U256::ZERO {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        return;
    }

    let Some(return_memory_offset) =
        resize_memory_and_return_range(interpreter, input_offset, input_size)
    else {
        return;
    };

    // TODO(EOF) check if destination is EOF.
    let Some(load_result) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    // TODO(EOF) is EOF!
    if load_result.is_cold {
        //gas!(interpreter, gas::COLD_ACCOUNT_ACCESS);
        return;
    }

    let is_new = !load_result.is_not_existing;
    let call_cost =
        gas::call_cost::<SPEC>(value != U256::ZERO, is_new, load_result.is_cold, true, true);
    gas!(interpreter, call_cost);

    // 7. Calculate the gas available to callee as caller’s
    // remaining gas reduced by max(ceil(gas/64), MIN_RETAINED_GAS) (MIN_RETAINED_GAS is 5000).
    let gas_reduce = max(interpreter.gas.remaining() / 64, 5000);
    let gas_limit = interpreter.gas().remaining().saturating_sub(gas_reduce);

    if gas_limit < 2300 {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        // TODO(EOF) error;
        // interpreter.instruction_result = InstructionResult::CallGasTooLow;
        return;
    }
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            contract: to,
            transfer: Transfer {
                source: interpreter.contract.address,
                target: to,
                value,
            },
            input,
            gas_limit,
            context: CallContext {
                address: to,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::Call,
            },
            is_static: interpreter.is_static,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn extdcall<H: Host>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
}

pub fn extscall<H: Host>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_disabled_eof!(interpreter);
}

pub fn create<const IS_CREATE2: bool, H: Host, SPEC: Spec>(
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
        shared_memory_resize!(interpreter, code_offset, len);
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
            caller: interpreter.contract.address,
            scheme,
            value,
            init_code: code,
            gas_limit,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    if interpreter.is_static && value != U256::ZERO {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        return;
    }

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(load_result) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        load_result,
        value != U256::ZERO,
        local_gas_limit,
        true,
        true,
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
            contract: to,
            transfer: Transfer {
                source: interpreter.contract.address,
                target: to,
                value,
            },
            input,
            gas_limit,
            context: CallContext {
                address: to,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::Call,
            },
            is_static: interpreter.is_static,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call_code<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(load_result) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        load_result,
        value != U256::ZERO,
        local_gas_limit,
        true,
        false,
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
            contract: to,
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value,
            },
            input,
            gas_limit,
            context: CallContext {
                address: interpreter.contract.address,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::CallCode,
            },
            is_static: interpreter.is_static,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn delegate_call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    check!(interpreter, HOMESTEAD);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(load_result) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        load_result,
        false,
        local_gas_limit,
        false,
        false,
    ) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            contract: to,
            // This is dummy send for StaticCall and DelegateCall,
            // it should do nothing and not touch anything.
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value: U256::ZERO,
            },
            input,
            gas_limit,
            context: CallContext {
                address: interpreter.contract.address,
                caller: interpreter.contract.caller,
                code_address: to,
                apparent_value: interpreter.contract.value,
                scheme: CallScheme::DelegateCall,
            },
            is_static: interpreter.is_static,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn static_call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    check!(interpreter, BYZANTIUM);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let value = U256::ZERO;
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(load_result) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let Some(gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        load_result,
        false,
        local_gas_limit,
        false,
        true,
    ) else {
        return;
    };
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = InterpreterAction::Call {
        inputs: Box::new(CallInputs {
            contract: to,
            // This is dummy send for StaticCall and DelegateCall,
            // it should do nothing and not touch anything.
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value: U256::ZERO,
            },
            input,
            gas_limit,
            context: CallContext {
                address: to,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::StaticCall,
            },
            is_static: true,
            return_memory_offset,
        }),
    };
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}
