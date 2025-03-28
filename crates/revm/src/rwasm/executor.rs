use crate::{
    interpreter::{
        interpreter_action::{SystemInterruptionInputs, SystemInterruptionOutcome},
        Gas,
        InstructionResult,
        Interpreter,
        InterpreterAction,
        InterpreterResult,
    },
    primitives::{hex, Address, Bytecode, Bytes, EVMError, Spec, U256},
    Context,
    Database,
};
use core::{ops::Deref, str::from_utf8};
use fluentbase_runtime::{
    instruction::{exec::SyscallExec, resume::SyscallResume},
    RuntimeContext,
};
use fluentbase_sdk::{
    codec::CompactABI,
    is_self_gas_management_contract,
    keccak256,
    BlockContextV1,
    BytecodeOrHash,
    ContractContextV1,
    ExitCode,
    SharedContextInput,
    SharedContextInputV1,
    SyscallInvocationParams,
    TxContextV1,
    B256,
    FUEL_DENOM_RATE,
    STATE_DEPLOY,
    STATE_MAIN,
    SYSCALL_ID_SYNC_EVM_GAS,
};
use revm_interpreter::{return_ok, return_revert};

pub(crate) fn execute_rwasm_frame<SPEC: Spec, EXT, DB: Database>(
    interpreter: &mut Interpreter,
    rwasm_bytecode: Bytes,
    context: &mut Context<EXT, DB>,
    is_create: bool,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    let bytecode_address = interpreter
        .contract
        .bytecode_address
        .unwrap_or(interpreter.contract.target_address);

    // encode input with all related context info
    let context_input = SharedContextInput::V1(SharedContextInputV1 {
        block: BlockContextV1::from(context.evm.env.deref()),
        tx: TxContextV1::from(context.evm.env.deref()),
        contract: ContractContextV1 {
            address: interpreter.contract.target_address,
            bytecode_address,
            caller: interpreter.contract.caller,
            is_static: interpreter.is_static,
            value: interpreter.contract.call_value,
        },
    });
    let mut context_input = context_input
        .encode()
        .expect("revm: unable to encode shared context input")
        .to_vec();
    context_input.extend_from_slice(interpreter.contract.input.as_ref());

    // calculate bytecode hash
    let rwasm_code_hash = interpreter
        .contract
        .hash
        .filter(|v| v != &B256::ZERO)
        .unwrap_or_else(|| keccak256(&rwasm_bytecode));
    debug_assert_eq!(rwasm_code_hash, keccak256(&rwasm_bytecode));
    let rwasm_bytecode = match &interpreter.contract.bytecode {
        Bytecode::Rwasm(bytecode) => bytecode.clone(),
        _ => unreachable!("revm: unexpected bytecode type"),
    };
    let bytecode_hash = BytecodeOrHash::Bytecode(rwasm_bytecode, Some(rwasm_code_hash));

    // fuel limit we denominate later to gas
    let fuel_limit = interpreter
        .gas
        .remaining()
        .checked_mul(FUEL_DENOM_RATE)
        .unwrap_or(u64::MAX);

    let is_gas_free = interpreter
        .contract
        .eip7702_address
        .filter(|eip7702_address| is_self_gas_management_contract(eip7702_address))
        .is_some();

    // execute function
    let mut runtime_context = RuntimeContext::root(fuel_limit);
    if is_gas_free {
        runtime_context = runtime_context.without_fuel();
    }
    let (fuel_consumed, fuel_refunded, exit_code) = SyscallExec::fn_impl(
        &mut runtime_context,
        bytecode_hash,
        &context_input,
        fuel_limit,
        if is_create { STATE_DEPLOY } else { STATE_MAIN },
    );
    if is_gas_free {
        debug_assert!(fuel_consumed == 0 && fuel_refunded == 0);
    }

    // make sure we have enough gas to charge from the call
    if !interpreter.gas.record_denominated_cost(fuel_consumed) {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas: interpreter.gas,
            },
        });
    }
    interpreter.gas.record_denominated_refund(fuel_refunded);

    // extract return data from the execution context
    let return_data: Bytes = runtime_context.into_return_data().into();

    Ok(process_exec_result(
        interpreter.contract.target_address,
        bytecode_address,
        interpreter.contract.eip7702_address,
        interpreter.contract.caller,
        interpreter.contract.call_value,
        exit_code,
        interpreter.gas,
        return_data,
        is_create,
        interpreter.is_static,
    ))
}

pub fn execute_rwasm_resume(outcome: SystemInterruptionOutcome) -> InterpreterAction {
    let SystemInterruptionOutcome {
        inputs,
        result,
        is_frame,
        ..
    } = outcome;

    // println!(
    //     "revm: resume execution: result={:?} gas={:?}",
    //     result.result, result.gas
    // );
    let fuel_consumed = result
        .gas
        .spent()
        .checked_mul(FUEL_DENOM_RATE)
        .unwrap_or(u64::MAX);
    let fuel_refunded = result
        .gas
        .refunded()
        .checked_mul(FUEL_DENOM_RATE as i64)
        .unwrap_or(i64::MAX);

    // we can safely convert the result into i32 here,
    // and we shouldn't worry about negative numbers
    // since the constraints is applied only for resulting exit codes
    let exit_code: ExitCode = match result.result {
        return_ok!() => ExitCode::Ok,
        return_revert!() => ExitCode::Panic,
        // a special case for frame execution where we always return `Err` as a failed call/create
        _ if is_frame => ExitCode::Err,
        InstructionResult::OutOfGas => ExitCode::OutOfFuel,
        _ => unreachable!("revm: not supported result code: {:?}", result.result),
    };

    // we count a contract as gas-free if it's a special system precompiled contract
    // that has self-gas management rules, for example, EVM/SVM runtimes
    let is_gas_free = inputs
        .eip7702_address
        .filter(|eip7702_address| is_self_gas_management_contract(eip7702_address))
        .is_some();

    // gas adjustment is needed
    // to synchronize gas/fuel between root and self-gas management runtimes,
    // this interruption can be made by EVM/SVM runtimes only
    let is_gas_adjustment = inputs.syscall_params.code_hash == SYSCALL_ID_SYNC_EVM_GAS;

    let mut runtime_context = RuntimeContext::root(0);
    if is_gas_free {
        runtime_context = runtime_context.without_fuel();
    }
    let (fuel_consumed, fuel_refunded, exit_code) = SyscallResume::fn_impl(
        &mut runtime_context,
        inputs.call_id,
        result.output.into(),
        exit_code.into_i32(),
        fuel_consumed,
        fuel_refunded,
        inputs.syscall_params.fuel16_ptr,
    );
    let return_data: Bytes = runtime_context.into_return_data().into();

    if is_gas_free {
        debug_assert!(fuel_consumed == 0 && fuel_refunded == 0);
    }

    // if we're free from paying gas,
    // then just take the previous gas value and don't charge anything
    let mut gas = if is_gas_free && !is_gas_adjustment {
        inputs.gas
    } else {
        result.gas
    };

    // make sure we have enough gas to charge from the call
    if !gas.record_denominated_cost(fuel_consumed) {
        return InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        };
    }
    // accumulate refunds (can be forwarded from an interrupted call)
    gas.record_denominated_refund(fuel_refunded);

    process_exec_result(
        inputs.target_address,
        inputs.bytecode_address,
        inputs.eip7702_address,
        inputs.caller,
        inputs.call_value,
        exit_code,
        gas,
        return_data,
        inputs.is_create,
        inputs.is_static,
    )
}

fn process_exec_result(
    target_address: Address,
    bytecode_address: Address,
    eip7702_address: Option<Address>,
    caller: Address,
    call_value: U256,
    exit_code: i32,
    gas: Gas,
    return_data: Bytes,
    is_create: bool,
    is_static: bool,
) -> InterpreterAction {
    // if we have success or failed exit code
    if exit_code <= 0 {
        return process_halt(exit_code, return_data.clone(), is_create, gas);
    }

    // otherwise, exit code is a "call_id" that identifies saved context
    let call_id = exit_code as u32;

    // try to parse execution params, if it's not possible then return an error
    let Ok(params) = CompactABI::<SyscallInvocationParams>::decode(&return_data, 0) else {
        unreachable!("revm: can't decode invocation params");
    };

    let is_gas_free = eip7702_address
        .filter(|eip7702_address| is_self_gas_management_contract(eip7702_address))
        .is_some();

    // if there is no enough gas for execution, then fail fast
    if !is_gas_free && params.fuel_limit / FUEL_DENOM_RATE > gas.remaining() {
        return InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        };
    }

    InterpreterAction::InterruptedCall {
        inputs: Box::new(SystemInterruptionInputs {
            target_address,
            bytecode_address,
            eip7702_address,
            caller,
            call_value,
            call_id,
            is_create,
            syscall_params: params,
            gas,
            is_static,
        }),
    }
}

fn process_halt(
    exit_code: i32,
    return_data: Bytes,
    is_create: bool,
    gas: Gas,
) -> InterpreterAction {
    let trace_output = |mut output: &[u8]| {
        if output.starts_with(&[0x08, 0xc3, 0x79, 0xa0]) {
            output = &output[68..];
        }
        println!(
            "output: 0x{} ({})",
            hex::encode(&output),
            from_utf8(output)
                .unwrap_or("can't decode utf-8")
                .trim_end_matches("\0")
        );
    };
    let exit_code = ExitCode::from(exit_code);
    if exit_code == ExitCode::Panic {
        trace_output(return_data.as_ref());
    }
    let result = match exit_code {
        ExitCode::Ok => {
            if is_create {
                InstructionResult::ReturnContract
            } else if return_data.is_empty() {
                InstructionResult::Stop
            } else {
                InstructionResult::Return
            }
        }
        ExitCode::Panic => InstructionResult::Revert,
        ExitCode::Err => InstructionResult::UnknownError,
        // rwasm failure codes
        ExitCode::RootCallOnly => InstructionResult::RootCallOnly,
        ExitCode::MalformedBuiltinParams => InstructionResult::MalformedBuiltinParams,
        ExitCode::CallDepthOverflow => InstructionResult::CallDepthOverflow,
        ExitCode::NonNegativeExitCode => InstructionResult::NonNegativeExitCode,
        ExitCode::UnknownError => InstructionResult::UnknownError,
        ExitCode::InputOutputOutOfBounds => InstructionResult::InputOutputOutOfBounds,
        ExitCode::UnreachableCodeReached => InstructionResult::UnreachableCodeReached,
        ExitCode::MemoryOutOfBounds => InstructionResult::MemoryOutOfBounds,
        ExitCode::TableOutOfBounds => InstructionResult::TableOutOfBounds,
        ExitCode::IndirectCallToNull => InstructionResult::IndirectCallToNull,
        ExitCode::IntegerDivisionByZero => InstructionResult::IntegerDivisionByZero,
        ExitCode::IntegerOverflow => InstructionResult::IntegerOverflow,
        ExitCode::BadConversionToInteger => InstructionResult::BadConversionToInteger,
        ExitCode::StackOverflow => InstructionResult::StackOverflow,
        ExitCode::BadSignature => InstructionResult::BadSignature,
        ExitCode::OutOfFuel => InstructionResult::OutOfFuel,
        ExitCode::GrowthOperationLimited => InstructionResult::GrowthOperationLimited,
        ExitCode::UnresolvedFunction => InstructionResult::UnresolvedFunction,
    };
    InterpreterAction::Return {
        result: InterpreterResult {
            result,
            output: return_data,
            gas,
        },
    }
}
