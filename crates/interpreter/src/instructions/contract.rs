mod call_helpers;

pub use call_helpers::{calc_call_gas, get_memory_input_and_out_ranges};

use crate::{
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::{Interpreter, InterpreterAction},
    primitives::{Address, Bytes, Log, LogData, Spec, SpecId::*, B256, U256},
    CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme, Host, InstructionResult,
    Transfer, MAX_INITCODE_SIZE,
};
use std::{boxed::Box, vec::Vec};
use core::cmp::min;
use revm_primitives::BLOCK_HASH_HISTORY;

pub fn create3<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn create4<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn return_contract<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn call2<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn delegate_call2<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

pub fn static_call2<H: Host>(interpreter: &mut Interpreter, host: &mut H) {}

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

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        host,
        to,
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

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        host,
        to,
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

    let Some(gas_limit) =
        calc_call_gas::<H, SPEC>(interpreter, host, to, false, local_gas_limit, false, false)
    else {
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

    let Some(gas_limit) =
        calc_call_gas::<H, SPEC>(interpreter, host, to, false, local_gas_limit, false, true)
    else {
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
