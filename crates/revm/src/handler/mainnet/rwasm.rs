use crate::{
    frame::SystemInterruptionFrame,
    primitives::{Address, Bytes, EVMError, Log, LogData, Spec, B256, TANGERINE, U256},
    Context,
    Database,
    Frame,
};
use core::{cmp::min, ops::Deref};
use fluentbase_runtime::{types::FixedPreimageResolver, RuntimeContext};
use fluentbase_sdk::{
    codec::CompactABI,
    runtime::RuntimeContextWrapper,
    BlockContextV1,
    ContractContextV1,
    ExitCode,
    NativeAPI,
    SharedContextInput,
    SharedContextInputV1,
    SyscallInvocationParams,
    TxContextV1,
    STATE_MAIN,
    SYSCALL_ID_BALANCE,
    SYSCALL_ID_CALL,
    SYSCALL_ID_CALL_CODE,
    SYSCALL_ID_CREATE,
    SYSCALL_ID_CREATE2,
    SYSCALL_ID_DELEGATE_CALL,
    SYSCALL_ID_DESTROY_ACCOUNT,
    SYSCALL_ID_EMIT_LOG,
    SYSCALL_ID_EXT_STORAGE_READ,
    SYSCALL_ID_PREIMAGE_COPY,
    SYSCALL_ID_PREIMAGE_SIZE,
    SYSCALL_ID_STATIC_CALL,
    SYSCALL_ID_STORAGE_READ,
    SYSCALL_ID_STORAGE_WRITE,
    SYSCALL_ID_TRANSIENT_READ,
    SYSCALL_ID_TRANSIENT_WRITE,
    SYSCALL_ID_WRITE_PREIMAGE,
};
use revm_interpreter::{
    gas,
    gas::{sload_cost, sstore_cost},
    CallInputs,
    CallScheme,
    CallValue,
    Gas,
    InstructionResult,
    Interpreter,
    InterpreterAction,
    InterpreterResult,
};

pub(crate) fn execute_rwasm_frame<SPEC: Spec, EXT, DB: Database>(
    interpreter: &mut Interpreter,
    rwasm_bytecode: Bytes,
    context: &mut Context<EXT, DB>,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    // encode input with all related context info
    let context_input = SharedContextInput::V1(SharedContextInputV1 {
        block: BlockContextV1::from(context.evm.env.deref()),
        tx: TxContextV1::from(context.evm.env.deref()),
        contract: ContractContextV1 {
            address: interpreter.contract.target_address,
            bytecode_address: interpreter
                .contract
                .bytecode_address
                .unwrap_or(interpreter.contract.target_address),
            caller: interpreter.contract.caller,
            is_static: interpreter.is_static,
            value: interpreter.contract.call_value,
        },
    });
    let context_input = context_input
        .encode()
        .expect("revm: unable to encode shared context input");
    let code_hash = interpreter.contract.hash.clone().unwrap();

    // execute function
    let runtime_context = RuntimeContext::root(interpreter.gas.limit());
    let preimage_resolver = FixedPreimageResolver::new(rwasm_bytecode, code_hash);
    let native_sdk =
        RuntimeContextWrapper::new(runtime_context).with_preimage_resolver(&preimage_resolver);
    let (fuel_consumed, exit_code) = native_sdk.exec(
        &code_hash,
        context_input.as_ref(),
        interpreter.gas.limit(),
        STATE_MAIN,
    );

    // make sure we have enough gas to charge from the call
    let mut gas = interpreter.gas;
    if !gas.record_cost(fuel_consumed) {
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
        interpreter.contract.caller,
        exit_code,
        gas,
        return_data,
    ))
}

fn process_exec_result(
    caller: Address,
    exit_code: i32,
    gas: Gas,
    return_data: Bytes,
) -> InterpreterAction {
    // if we have success or failed exit code
    if exit_code <= 0 {
        let result = if exit_code == 0 {
            if return_data.is_empty() {
                InstructionResult::Stop
            } else {
                InstructionResult::Return
            }
        } else {
            InstructionResult::Revert
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

    InterpreterAction::InterruptRwasm {
        caller,
        call_id,
        code_hash: params.code_hash,
        input: params.input,
        gas_limit: params.fuel_limit,
        state: params.state,
    }
}

#[inline]
pub fn resume_rwasm_frame<SPEC: Spec, EXT, DB: Database>(
    _context: &mut Context<EXT, DB>,
    call_id: u32,
    result: InterpreterResult,
    caller: Address,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    let runtime_context = RuntimeContext::root(0);
    let native_sdk = RuntimeContextWrapper::new(runtime_context);
    let exit_code = if result.is_ok() {
        ExitCode::Ok.into_i32()
    } else {
        ExitCode::MalformedSyscallParams.into_i32()
    };
    let (fuel_consumed, exit_code) = native_sdk.resume(
        call_id,
        result.output.as_ref(),
        exit_code,
        result.gas.spent(),
    );

    // make sure we have enough gas to charge from the call
    let mut gas = result.gas;
    if !gas.record_cost(fuel_consumed) {
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

    Ok(process_exec_result(caller, exit_code, gas, return_data))
}

pub(crate) fn execute_system_interruption<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    params: &Box<SystemInterruptionFrame>,
) -> Result<InterpreterAction, EVMError<DB::Error>> {
    let mut gas = Gas::new(params.gas_limit);

    macro_rules! return_result {
        ($output:expr) => {
            return Ok(InterpreterAction::ResumeRwasm {
                call_id: params.call_id,
                result: InterpreterResult::new(InstructionResult::Return, $output.into(), gas),
                caller: params.caller,
            })
        };
    }
    macro_rules! return_error {
        ($error:ident) => {
            return Ok(InterpreterAction::ResumeRwasm {
                call_id: params.call_id,
                result: InterpreterResult::new(InstructionResult::$error, Default::default(), gas),
                caller: params.caller,
            })
        };
    }
    macro_rules! assert_return {
        ($cond:expr, $error:ident) => {
            if !($cond) {
                return_error!($error);
            }
        };
    }
    macro_rules! charge_gas {
        ($value:expr) => {
            if !gas.record_cost($value) {
                return_error!(OutOfGas);
            }
        };
    }

    match params.code_hash {
        SYSCALL_ID_STORAGE_READ => {
            assert_return!(
                params.input.len() == 32 && params.state == STATE_MAIN,
                Revert
            );
            let slot = U256::from_le_slice(&params.input[0..32]);
            // execute sload
            let value = context.evm.sload(params.caller, slot)?;
            charge_gas!(sload_cost(SPEC::SPEC_ID, value.is_cold));
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output)
        }

        SYSCALL_ID_STORAGE_WRITE => {
            assert_return!(
                params.input.len() == 32 + 32 && params.state == STATE_MAIN,
                Revert
            );
            // don't allow for static context
            assert_return!(!params.is_static, CallNotAllowedInsideStatic);
            let slot = U256::from_le_slice(&params.input[0..32]);
            let new_value = U256::from_le_slice(&params.input[32..64]);
            // execute sstore
            let value = context.evm.sstore(params.caller, slot, new_value)?;
            if let Some(gas_cost) =
                sstore_cost(SPEC::SPEC_ID, &value.data, params.gas_limit, value.is_cold)
            {
                charge_gas!(gas_cost);
            } else {
                return_error!(OutOfGas);
            }
            return_result!(Bytes::default())
        }

        SYSCALL_ID_CALL => {
            assert_return!(
                params.input.len() >= 20 + 32 && params.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&params.input[0..20]);
            let value = U256::from_le_slice(&params.input[20..52]);
            let contract_input = params.input.slice(52..);
            // for static calls with value greater than 0 - revert
            let has_transfer = !value.is_zero();
            if params.is_static && has_transfer {
                return_error!(CallNotAllowedInsideStatic);
            }
            let Ok(account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, has_transfer, account_load));
            let mut gas_limit = if SPEC::enabled(TANGERINE) {
                min(gas.remaining_63_of_64_parts(), params.gas_limit)
            } else {
                params.gas_limit
            };
            charge_gas!(gas_limit);
            if has_transfer {
                gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
            }
            // create call inputs
            let inputs = Box::new(CallInputs {
                input: contract_input,
                gas_limit,
                target_address,
                caller: params.caller,
                bytecode_address: target_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::Call,
                is_static: params.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            Ok(InterpreterAction::Call { inputs })
        }

        SYSCALL_ID_STATIC_CALL
        | SYSCALL_ID_CALL_CODE
        | SYSCALL_ID_DELEGATE_CALL
        | SYSCALL_ID_CREATE
        | SYSCALL_ID_CREATE2 => {
            unreachable!("revm: unsupported system interruption")
        }

        SYSCALL_ID_EMIT_LOG => {
            assert_return!(
                params.input.len() >= 1 && params.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!params.is_static, CallNotAllowedInsideStatic);
            // read topics from input
            let topics_len = params.input[0] as usize;
            assert_return!(topics_len <= 4, Revert);
            let mut topics = Vec::with_capacity(topics_len);
            assert_return!(
                params.input.len() >= 1 + topics_len * B256::len_bytes(),
                Revert
            );
            for i in 0..topics_len {
                let offset = 1 + i * B256::len_bytes();
                let topic = &params.input.as_ref()[offset..(offset + B256::len_bytes())];
                topics.push(B256::from_slice(topic));
            }
            // all remaining bytes are data
            let data = params.input.slice((1 + topics_len * B256::len_bytes())..);
            // make sure we have enough gas to cover this operation
            let Some(gas_cost) = gas::log_cost(topics_len as u8, data.len() as u64) else {
                return_error!(OutOfGas);
            };
            charge_gas!(gas_cost);
            // write new log into the journal
            context.evm.journaled_state.log(Log {
                address: params.caller,
                // it's safe to go unchecked here because we do topics check upper
                data: LogData::new_unchecked(topics, data),
            });
            return_result!(Bytes::default());
        }

        SYSCALL_ID_DESTROY_ACCOUNT => {
            assert_return!(
                params.input.len() == 20 && params.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!params.is_static, CallNotAllowedInsideStatic);
            // destroy an account
            let target = Address::from_slice(&params.input[0..20]);
            let result = context.evm.selfdestruct(params.caller, target)?;
            // charge gas cost
            charge_gas!(gas::selfdestruct_cost(SPEC::SPEC_ID, result));
            // return value as bytes with success exit code
            return_result!(Bytes::default());
        }

        SYSCALL_ID_BALANCE => {
            assert_return!(
                params.input.len() == 20 && params.state == STATE_MAIN,
                Revert
            );
            let address = Address::from_slice(&params.input[0..20]);
            let value = context.evm.balance(address)?;
            // make sure we have enough gas for this op
            charge_gas!(if params.caller == address {
                gas::LOW
            } else if value.is_cold {
                gas::COLD_ACCOUNT_ACCESS_COST
            } else {
                gas::WARM_STORAGE_READ_COST
            });
            // write the result
            let output: [u8; 32] = value.data.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_WRITE_PREIMAGE | SYSCALL_ID_PREIMAGE_COPY | SYSCALL_ID_PREIMAGE_SIZE => {
            unreachable!("revm: unsupported system interruption")
        }

        SYSCALL_ID_EXT_STORAGE_READ => {
            unreachable!("revm: unsupported system interruption")
        }

        SYSCALL_ID_TRANSIENT_READ => {
            assert_return!(
                params.input.len() == 32 && params.state == STATE_MAIN,
                Revert
            );
            // read value from storage
            let slot = U256::from_le_slice(&params.input[0..32].as_ref());
            let value = context.evm.tload(params.caller, slot);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            // return value
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_TRANSIENT_WRITE => {
            assert_return!(
                params.input.len() == 64 && params.state == STATE_MAIN,
                Revert
            );
            assert_return!(!params.is_static, CallNotAllowedInsideStatic);
            // read input
            let slot = U256::from_le_slice(&params.input[0..32]);
            let value = U256::from_le_slice(&params.input[32..64]);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            context.evm.tstore(params.caller, slot, value);
            // empty result
            return_result!(Bytes::default());
        }

        _ => unreachable!("revm: unsupported system interruption"),
    }
}
