use crate::{
    rwasm::{execute_rwasm_interruption, SystemInterruptionInputs, SystemInterruptionOutcome},
    ContextTrDbError,
    EthFrame,
    EvmTr,
};
use bytecode::Bytecode;
use context_interface::{result::FromStringError, Block, Cfg, ContextTr, Transaction};
use core::{mem::take, ops::Deref};
use fluentbase_genesis::is_self_gas_management_contract;
use fluentbase_runtime::{
    instruction::{exec::SyscallExec, resume::SyscallResume},
    RuntimeContext,
};
use fluentbase_sdk::{
    codec::CompactABI,
    Address,
    BlockContextV1,
    BytecodeOrHash,
    Bytes,
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
    U256,
};
use interpreter::{
    interpreter::EthInterpreter,
    interpreter_types::{InputsTr, LoopControl, RuntimeFlag},
    return_ok,
    return_revert,
    CallInput,
    FrameInput,
    Gas,
    InstructionResult,
    InterpreterAction,
    InterpreterResult,
};

pub fn execute_rwasm_frame<
    EVM: EvmTr,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
>(
    frame: &mut EthFrame<EVM, ERROR, EthInterpreter>,
    evm: &mut EVM,
) -> Result<InterpreterAction, ERROR> {
    let interpreter = &mut frame.interpreter;
    let is_create: bool = matches!(frame.input, FrameInput::Create(..));
    let is_static: bool = interpreter.runtime_flag.is_static();
    let bytecode_address = interpreter
        .input
        .bytecode_address()
        .cloned()
        .unwrap_or_else(|| interpreter.input.target_address());
    let context = evm.ctx();

    // encode input with all related context info
    let context_input = SharedContextInput::V1(SharedContextInputV1 {
        block: BlockContextV1 {
            chain_id: context.cfg().chain_id(),
            coinbase: context.block().beneficiary(),
            timestamp: context.block().timestamp(),
            number: context.block().number(),
            difficulty: context.block().difficulty(),
            prev_randao: context.block().prevrandao().unwrap(),
            gas_limit: context.block().gas_limit(),
            base_fee: U256::from(context.block().basefee()),
        },
        tx: TxContextV1 {
            gas_limit: context.tx().gas_limit(),
            nonce: context.tx().nonce(),
            gas_price: U256::from(context.tx().gas_price()),
            gas_priority_fee: context
                .tx()
                .max_priority_fee_per_gas()
                .map(|v| U256::from(v)),
            origin: context.tx().caller(),
            value: context.tx().value(),
        },
        contract: ContractContextV1 {
            address: interpreter.input.target_address(),
            bytecode_address,
            caller: context.tx().caller(),
            is_static: interpreter.runtime_flag.is_static(),
            value: context.tx().value(),
            gas_limit: interpreter.control.gas().remaining(),
        },
    });
    let mut context_input = context_input
        .encode()
        .expect("revm: unable to encode shared context input")
        .to_vec();
    let input = interpreter.input.input().clone();
    let bytes = match input {
        CallInput::Bytes(vec) => vec,
        _ => unreachable!("not bytes found in call input "), /* TODO(khasan): this shared memory
                                                              * thing can be returned here? */
    };
    context_input.extend_from_slice(&bytes);

    let rwasm_code_hash = interpreter.bytecode.hash().unwrap();

    let rwasm_bytecode = match &interpreter.bytecode.clone() {
        Bytecode::Rwasm(bytecode) => bytecode.clone(),
        _ => unreachable!("revm: unexpected bytecode type"),
    };
    let bytecode_hash = BytecodeOrHash::Bytecode(rwasm_bytecode, Some(rwasm_code_hash));

    // fuel limit we denominate later to gas
    let fuel_limit = interpreter
        .control
        .gas
        .remaining()
        .checked_mul(FUEL_DENOM_RATE)
        .unwrap_or(u64::MAX);

    let is_gas_free = interpreter
        .input
        .rwasm_proxy_address
        .or_else(|| Some(bytecode_address))
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

    // make sure we have enough gas to charge from the call
    if !interpreter
        .control
        .gas
        .record_denominated_cost(fuel_consumed)
    {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas: interpreter.control.gas,
            },
        });
    }
    interpreter
        .control
        .gas
        .record_denominated_refund(fuel_refunded);

    // extract return data from the execution context
    let return_data: Bytes;
    return_data = runtime_context.into_return_data();

    let gas = interpreter.control.gas;
    process_exec_result(
        frame,
        evm,
        exit_code,
        gas,
        return_data,
        is_create,
        is_static,
        is_gas_free,
    )
}

pub fn execute_rwasm_resume<
    EVM: EvmTr,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
>(
    frame: &mut EthFrame<EVM, ERROR, EthInterpreter>,
    evm: &mut EVM,
    outcome: SystemInterruptionOutcome,
) -> Result<InterpreterAction, ERROR> {
    let SystemInterruptionOutcome {
        inputs,
        result,
        is_frame,
        ..
    } = outcome;

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
        // InstructionResult::MalformedBuiltinParams => ExitCode::MalformedBuiltinParams,
        // InstructionResult::PrecompileError => ExitCode::PrecompileError,
        // TODO(dmitry123): "don't panic here, for tests only"
        _ => {
            unreachable!("revm: not supported result code: {:?}", result.result)
        }
    };

    let mut runtime_context = RuntimeContext::root(0);
    if inputs.is_gas_free {
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

    let return_data: Bytes;
    return_data = runtime_context.into_return_data();

    // if we're free from paying gas,
    // then just take the previous gas value and don't charge anything
    let mut gas = if inputs.is_gas_free {
        inputs.gas
    } else {
        result.gas
    };

    // make sure we have enough gas to charge from the call
    if !gas.record_denominated_cost(fuel_consumed) {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        });
    }
    // accumulate refunds (can be forwarded from an interrupted call)
    gas.record_denominated_refund(fuel_refunded);

    process_exec_result::<EVM, ERROR>(
        frame,
        evm,
        exit_code,
        gas,
        return_data,
        inputs.is_create,
        inputs.is_static,
        inputs.is_gas_free,
    )
}

fn process_exec_result<
    EVM: EvmTr,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
>(
    frame: &mut EthFrame<EVM, ERROR, EthInterpreter>,
    evm: &mut EVM,
    exit_code: i32,
    gas: Gas,
    return_data: Bytes,
    is_create: bool,
    is_static: bool,
    is_gas_free: bool,
) -> Result<InterpreterAction, ERROR> {
    // if we have success or failed exit code
    if exit_code <= 0 {
        return Ok(process_halt(exit_code, return_data.clone(), is_create, gas));
    }

    // otherwise, exit code is a "call_id" that identifies saved context
    let call_id = exit_code as u32;

    // try to parse execution params, if it's not possible then return an error
    let Ok(params) = CompactABI::<SyscallInvocationParams>::decode(&return_data, 0) else {
        unreachable!("revm: can't decode invocation params");
    };

    // if there is no enough gas for execution, then fail fast
    if !is_gas_free && params.fuel_limit / FUEL_DENOM_RATE > gas.remaining() {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        });
    }

    let inputs = SystemInterruptionInputs {
        call_id,
        is_create,
        syscall_params: params,
        gas,
        is_static,
        is_gas_free,
    };

    execute_rwasm_interruption::<EVM, ERROR>(frame, evm, inputs)
}

fn process_halt(
    exit_code: i32,
    return_data: Bytes,
    is_create: bool,
    gas: Gas,
) -> InterpreterAction {
    #[cfg(feature = "debug-print")]
    let trace_output = |mut output: &[u8]| {
        use core::str::from_utf8;
        use fluentbase_sdk::hex;
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
        #[cfg(feature = "debug-print")]
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
        ExitCode::PrecompileError => InstructionResult::PrecompileError,
    };
    InterpreterAction::Return {
        result: InterpreterResult {
            result,
            output: return_data,
            gas,
        },
    }
}

pub fn run_rwasm_loop<EVM: EvmTr, ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError>(
    frame: &mut EthFrame<EVM, ERROR, EthInterpreter>,
    evm: &mut EVM,
) -> Result<InterpreterAction, ERROR> {
    loop {
        let next_action = if let Some(interrupted_outcome) = frame.take_interrupted_outcome() {
            execute_rwasm_resume(frame, evm, interrupted_outcome)
        } else {
            execute_rwasm_frame(frame, evm)
        }?;
        let result = match next_action {
            InterpreterAction::Return { result } => result,
            _ => return Ok(next_action),
        };
        if !frame.is_interrupted_call() {
            return Ok(InterpreterAction::Return { result });
        }
    }
}
