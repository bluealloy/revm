use crate::{
    interpreter::{
        interpreter_action::{SystemInterruptionInputs, SystemInterruptionOutcome},
        Gas,
        InstructionResult,
        Interpreter,
        InterpreterAction,
        InterpreterResult,
    },
    primitives::{Address, Bytecode, Bytes, EVMError, Spec, U256},
    Context,
    Database,
    Frame,
};
use core::{mem, ops::Deref};
use fluentbase_runtime::RuntimeContext;
use fluentbase_sdk::{
    codec::CompactABI,
    is_self_gas_management_contract,
    keccak256,
    runtime::RuntimeContextWrapper,
    BlockContextV1,
    BytecodeOrHash,
    ContractContextV1,
    ExitCode,
    NativeAPI,
    SharedContextInput,
    SharedContextInputV1,
    SyscallInvocationParams,
    TxContextV1,
    B256,
    FUEL_DENOM_RATE,
    STATE_DEPLOY,
    STATE_MAIN,
};
use revm_interpreter::{opcode, opcode::InstructionTables, SharedMemory, EMPTY_SHARED_MEMORY};

pub(crate) fn execute_rwasm_frame<SPEC: Spec, EXT, DB: Database>(
    interpreter: &mut Interpreter,
    rwasm_bytecode: Bytes,
    context: &mut Context<EXT, DB>,
    is_create: bool,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    // encode input with all related context info
    let bytecode_address = interpreter
        .contract
        .bytecode_address
        .unwrap_or(interpreter.contract.target_address);
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
    let fuel_limit = interpreter.gas.remaining() * FUEL_DENOM_RATE;

    // execute function
    let mut runtime_context = RuntimeContext::root(fuel_limit);
    if is_self_gas_management_contract(&bytecode_address) {
        runtime_context = runtime_context.without_fuel();
    }
    let native_sdk = RuntimeContextWrapper::new(runtime_context);
    let (fuel_consumed, exit_code) = native_sdk.exec(
        bytecode_hash,
        &context_input,
        fuel_limit,
        if is_create { STATE_DEPLOY } else { STATE_MAIN },
    );

    // make sure we have enough gas to charge from the call
    let mut gas = interpreter.gas;
    if !gas.record_denominated_cost(fuel_consumed as u64) {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        });
    }

    // extract return data from the execution context
    let return_data = native_sdk.return_data();

    Ok(process_exec_result(
        interpreter.contract.target_address,
        interpreter
            .contract
            .bytecode_address
            .unwrap_or(interpreter.contract.target_address),
        interpreter.contract.caller,
        interpreter.contract.call_value,
        exit_code,
        gas,
        return_data,
        is_create,
        interpreter.is_static,
    ))
}

fn process_exec_result(
    target_address: Address,
    bytecode_address: Address,
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
        let exit_code = ExitCode::from(exit_code);
        println!("revm: exit code: {} ({})", exit_code.into_i32(), exit_code);
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
            ExitCode::BadSignature => InstructionResult::Return,
            ExitCode::OutOfFuel => InstructionResult::OutOfGas,
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
            ExitCode::GrowthOperationLimited => InstructionResult::GrowthOperationLimited,
            ExitCode::UnresolvedFunction => InstructionResult::UnresolvedFunction,
        };
        return InterpreterAction::Return {
            result: InterpreterResult {
                result,
                output: return_data,
                gas,
            },
        };
    }

    // otherwise, exit code is a "call_id" that identifies saved context
    let call_id = exit_code as u32;

    // try to parse execution params, if it's not possible then return an error
    let Ok(params) = CompactABI::<SyscallInvocationParams>::decode(&return_data, 0) else {
        unreachable!("revm: can't decode invocation params");
    };

    // if there is no enough gas for execution, then fail fast
    if params.gas_limit > gas.remaining() {
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

pub fn execute_evm_resume<SPEC: Spec, EXT, DB: Database>(
    interrupted_outcome: SystemInterruptionOutcome,
    frame: &mut Frame,
    shared_memory: &mut SharedMemory,
    instruction_tables: &InstructionTables<'_, Context<EXT, DB>>,
    context: &mut Context<EXT, DB>,
) -> InterpreterAction {
    let interpreter = frame.interpreter_mut();

    debug_assert!(
        interpreter.instruction_pointer > interpreter.bytecode.as_ptr(),
        "revm: instruction pointer underflow"
    );
    let prev_opcode = unsafe { *interpreter.instruction_pointer.offset(-1) };

    let InterpreterResult {
        result,
        output,
        gas,
    } = interrupted_outcome.result;

    // if execution failed, then we must terminate execution of the contract
    if !result.is_ok() {
        interpreter.gas = gas;
        return InterpreterAction::Return {
            result: InterpreterResult::new(result, output, gas),
        };
    }

    interpreter.gas = gas;

    match prev_opcode {
        opcode::BALANCE | opcode::SELFBALANCE | opcode::EXTCODESIZE | opcode::CODESIZE => {
            assert_eq!(output.len(), 32);
            let balance = U256::from_le_slice(output.as_ref());
            if let Err(result) = interpreter.stack.push(balance) {
                interpreter.instruction_result = result;
            }
        }
        opcode::EXTCODEHASH => {
            assert_eq!(output.len(), 32);
            let code_hash = B256::from_slice(output.as_ref());
            if let Err(result) = interpreter.stack.push(code_hash.into()) {
                interpreter.instruction_result = result;
            }
        }
        _ => unreachable!(
            "revm: not resumable opcode ({})",
            interpreter.current_opcode()
        ),
    }

    if interpreter.instruction_result == InstructionResult::CallOrCreate {
        interpreter.instruction_result = InstructionResult::Continue;
    }

    let memory = mem::replace(shared_memory, EMPTY_SHARED_MEMORY);
    let next_action = match instruction_tables {
        InstructionTables::Plain(table) => interpreter.run(memory, table, context),
        InstructionTables::Boxed(table) => interpreter.run(memory, table, context),
    };
    // Take the shared memory back.
    *shared_memory = interpreter.take_memory();

    next_action
}

pub fn execute_rwasm_resume(
    system_interruption_outcome: SystemInterruptionOutcome,
) -> InterpreterAction {
    let fuel_consumed = system_interruption_outcome.gas_consumed.spent() * FUEL_DENOM_RATE;
    // TODO(dmitry123): "we need to pass refund as well"
    if system_interruption_outcome.gas_consumed.refunded() > 0 {
        unreachable!("we don't support refunds yet")
    }

    let SystemInterruptionOutcome {
        call_id,
        target_address,
        bytecode_address,
        caller,
        call_value,
        is_create,
        is_static,
        result,
        exit_code,
        ..
    } = system_interruption_outcome;

    let mut runtime_context = RuntimeContext::root(0);
    if is_self_gas_management_contract(&bytecode_address) {
        runtime_context = runtime_context.without_fuel();
    }
    let native_sdk = RuntimeContextWrapper::new(runtime_context);
    let (fuel_consumed, exit_code) = native_sdk.resume(
        call_id,
        result.output.as_ref(),
        exit_code,
        fuel_consumed as u32,
    );

    // make sure we have enough gas to charge from the call
    let mut gas = result.gas;
    if !gas.record_denominated_cost(fuel_consumed as u64) {
        return InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        };
    }

    // extract return data from the execution context
    let return_data = native_sdk.return_data();

    process_exec_result(
        target_address,
        bytecode_address,
        caller,
        call_value,
        exit_code,
        gas,
        return_data,
        is_create,
        is_static,
    )
}
