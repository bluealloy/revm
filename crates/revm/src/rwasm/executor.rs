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
    Frame,
};
use core::{mem, ops::Deref, str::from_utf8};
use fluentbase_runtime::{
    instruction::{
        exit::SyscallExit, input_size::SyscallInputSize, read::SyscallRead, resume::SyscallResume, write::SyscallWrite
    },
    RuntimeContext,
};
use fluentbase_rwasm::RwasmError;
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
    PRECOMPILE_EVM_RUNTIME,
    STATE_DEPLOY,
    STATE_MAIN,
};
use revm_interpreter::{
    opcode,
    opcode::InstructionTables,
    return_error,
    return_ok,
    return_revert,
    SharedMemory,
    EMPTY_SHARED_MEMORY,
};
use wasmtime::*;

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

    let (fuel_consumed, fuel_refunded, exit_code, return_data) = match &bytecode_address {
        _ => {
            // First the wasm module needs to be compiled. This is done with a global
            // "compilation environment" within an `Engine`. Note that engines can be
            // further configured through `Config` if desired instead of using the
            // default like this is here.
            let runtime_context = RuntimeContext::root(fuel_limit)
                .with_input(context_input);

            println!("Compiling module...");
            let wasm_bytecode = include_bytes!("../../../../../examples/identity/lib.wasm");
            let engine = Engine::default();
            let module = Module::new(&engine, wasm_bytecode).unwrap();

            // After a module is compiled we create a `Store` which will contain
            // instantiated modules and other items like host functions. A Store
            // contains an arbitrary piece of host information, and we use `MyState`
            // here.
            println!("Initializing...");
            let mut store = Store::new(&engine, runtime_context);

            // Our wasm module we'll be instantiating requires one imported function.
            // the function takes no parameters and returns no results. We create a host
            // implementation of that function here, and the `caller` parameter here is
            // used to get access to our original `MyState` value.
            println!("Creating callback...");

            let mut linker = Linker::new(&engine);
            linker
                .func_wrap(
                    "fluentbase_v1preview",
                    "_write",
                    |mut caller: Caller<'_, RuntimeContext>, offset: u32, length: u32| {
                        let memory = match caller.get_export("memory") {
                            Some(Extern::Memory(memory)) => memory,
                            _ => panic!("failed to find host memory"),
                        };
                        let data: Vec<u8> = memory
                            .data(&caller)
                            .get(offset as usize..)
                            .and_then(|arr| arr.get(..length as usize))
                            .unwrap()
                            .into();
                        let ctx = caller.data_mut();
                        SyscallWrite::fn_impl(ctx, &data);
                    },
                )
                .unwrap();

            linker
                .func_wrap(
                    "fluentbase_v1preview",
                    "_input_size",
                    |caller: Caller<'_, RuntimeContext>| -> anyhow::Result<u32> {
                        let size = SyscallInputSize::fn_impl(caller.data());
                        println!("_input_size syscall was executed with size={}", size);
                        Ok(size)
                    }
                )
                .unwrap();

            linker
                .func_wrap(
                    "fluentbase_v1preview",
                    "_read",
                    |mut caller: Caller<'_, RuntimeContext>,
                     target_ptr: u32,
                     offset: u32,
                     length: u32| {
                        // memory.write(caller.data_mut())

                        let buffer = SyscallRead::fn_impl(caller.data(), offset, length).unwrap();
                        let memory = match caller.get_export("memory") {
                            Some(Extern::Memory(memory)) => memory,
                            _ => panic!("failed to find host memory"),
                        };
                        let _ = memory.write(caller, target_ptr as usize, &buffer);
                    },
                )
                .unwrap();

            linker
                .func_wrap(
                    "fluentbase_v1preview",
                    "_exit",
                    |mut caller: Caller<'_, RuntimeContext>, exit_code: i32| -> anyhow::Result<()> {
                        let exit_code = SyscallExit::fn_impl(caller.data_mut(), exit_code).unwrap_err();
                        println!("_exit syscall was executed with exit code {}", exit_code);
                        Err(anyhow::Error::new(exit_code))


                    },
                )
                .unwrap();

            let instance = linker.instantiate(&mut store, &module).unwrap();

            // Next we poke around a bit to extract the `run` function from the module.
            println!("Extracting export...");
            let main = instance
                .get_typed_func::<(), ()>(&mut store, "main")
                .unwrap();
            println!("Calling export...");
            match main.call(&mut store, ()) {
                Ok(_) => {},
                Err(exit_code) => {
                    println!("hey here is error code {:?}", exit_code);
                    println!("{:?}", store.data().output());
                },
            }
            println!("_____________");
            (1, 1, 0, store.data().output().clone().into())
        }
        &PRECOMPILE_EVM_RUNTIME => {
            let mut runtime_context = RuntimeContext::root(fuel_limit);
            runtime_context = runtime_context.without_fuel();
            let native_sdk = RuntimeContextWrapper::new(runtime_context);
            let (fuel_consumed, fuel_refunded, exit_code) = native_sdk.exec(
                bytecode_hash,
                &context_input,
                fuel_limit,
                if is_create { STATE_DEPLOY } else { STATE_MAIN },
            );
            (
                fuel_consumed,
                fuel_refunded,
                exit_code,
                native_sdk.return_data(),
            )
        }
    };

    // make sure we have enough gas to charge from the call
    let mut gas = interpreter.gas;
    if !gas.record_denominated_cost(fuel_consumed) {
        return Ok(InterpreterAction::Return {
            result: InterpreterResult {
                result: InstructionResult::OutOfGas,
                output: Bytes::default(),
                gas,
            },
        });
    }
    gas.record_denominated_refund(fuel_refunded);

    Ok(process_exec_result(
        interpreter.contract.target_address,
        bytecode_address,
        interpreter.contract.eip7702_address,
        interpreter.contract.caller,
        interpreter.contract.call_value,
        exit_code,
        gas,
        return_data,
        is_create,
        interpreter.is_static,
    ))
}

pub fn execute_rwasm_resume(outcome: SystemInterruptionOutcome) -> InterpreterAction {
    let SystemInterruptionOutcome { inputs, result, .. } = outcome;

    println!(
        "revm: resume execution: result={:?} gas={:?}",
        result.result, result.gas
    );
    let fuel_consumed = result.gas.spent() * FUEL_DENOM_RATE;
    let fuel_refunded = result.gas.refunded() * FUEL_DENOM_RATE as i64;

    let exit_code = result.result as i32;

    let mut runtime_context = RuntimeContext::root(0);
    let is_gas_free = inputs
        .eip7702_address
        .and_then(|eip7702_address| Some(is_self_gas_management_contract(&eip7702_address)))
        .unwrap_or(false);
    if is_gas_free {
        runtime_context = runtime_context.without_fuel();
    }
    let (fuel_consumed, fuel_refunded, exit_code) = SyscallResume::fn_impl(
        &mut runtime_context,
        inputs.call_id,
        result.output.into(),
        exit_code,
        fuel_consumed,
        fuel_refunded,
        inputs.syscall_params.fuel16_ptr,
    );
    let return_data: Bytes = runtime_context.into_return_data().into();

    // if we're free from paying gas,
    // then just take the previous gas value and don't charge anything
    let mut gas = if is_gas_free { inputs.gas } else { result.gas };

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
        let exit_code = ExitCode::from(exit_code);
        if exit_code == ExitCode::Panic {
            let mut output = return_data.as_ref();
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
    if params.fuel_limit / FUEL_DENOM_RATE > gas.remaining() {
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
