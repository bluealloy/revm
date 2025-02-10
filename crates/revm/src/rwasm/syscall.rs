use crate::{
    interpreter::{
        gas,
        gas::{sload_cost, sstore_cost},
        interpreter_action::SystemInterruptionInputs,
        CallInputs,
        CallOutcome,
        CallScheme,
        CallValue,
        InstructionResult,
        InterpreterResult,
    },
    primitives::{Address, Bytes, EVMError, Log, LogData, Spec, B256, TANGERINE, U256},
    Context,
    Database,
    FrameOrResult,
    FrameResult,
};
use core::cmp::min;
use fluentbase_sdk::{
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

pub(crate) fn execute_rwasm_interruption<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: &mut Box<SystemInterruptionInputs>,
) -> Result<FrameOrResult, EVMError<DB::Error>> {
    macro_rules! return_result {
        ($output:expr) => {{
            let result =
                InterpreterResult::new(InstructionResult::Return, $output.into(), inputs.gas);
            return Ok(FrameOrResult::Result(FrameResult::Call(CallOutcome::new(
                result,
                Default::default(),
            ))));
        }};
    }
    macro_rules! return_error {
        ($error:ident) => {{
            let result =
                InterpreterResult::new(InstructionResult::$error, Default::default(), inputs.gas);
            return Ok(FrameOrResult::Result(FrameResult::Call(CallOutcome::new(
                result,
                Default::default(),
            ))));
        }};
    }
    macro_rules! return_frame {
        ($frame:expr) => {
            return Ok($frame);
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
            if !inputs.gas.record_cost($value) {
                return_error!(OutOfGas);
            }
        };
    }

    match inputs.code_hash {
        SYSCALL_ID_STORAGE_READ => {
            assert_return!(
                inputs.input.len() == 32 && inputs.state == STATE_MAIN,
                Revert
            );
            let slot = U256::from_le_slice(&inputs.input[0..32]);
            // execute sload
            let value = context.evm.sload(inputs.target_address, slot)?;
            charge_gas!(sload_cost(SPEC::SPEC_ID, value.is_cold));
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output)
        }

        SYSCALL_ID_STORAGE_WRITE => {
            assert_return!(
                inputs.input.len() == 32 + 32 && inputs.state == STATE_MAIN,
                Revert
            );
            // don't allow for static context
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            let slot = U256::from_le_slice(&inputs.input[0..32]);
            let new_value = U256::from_le_slice(&inputs.input[32..64]);
            // execute sstore
            let value = context.evm.sstore(inputs.target_address, slot, new_value)?;
            if let Some(gas_cost) = sstore_cost(
                SPEC::SPEC_ID,
                &value.data,
                inputs.local_gas_limit,
                value.is_cold,
            ) {
                charge_gas!(gas_cost);
            } else {
                return_error!(OutOfGas);
            }
            return_result!(Bytes::default())
        }

        SYSCALL_ID_CALL => {
            assert_return!(
                inputs.input.len() >= 20 + 32 && inputs.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.input[0..20]);
            let value = U256::from_le_slice(&inputs.input[20..52]);
            let contract_input = inputs.input.slice(52..);
            // for static calls with value greater than 0 - revert
            let has_transfer = !value.is_zero();
            if inputs.is_static && has_transfer {
                return_error!(CallNotAllowedInsideStatic);
            }
            let Ok(account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, has_transfer, account_load));
            let mut gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    inputs.gas.remaining_63_of_64_parts(),
                    inputs.local_gas_limit,
                )
            } else {
                inputs.local_gas_limit
            };
            charge_gas!(gas_limit);
            if has_transfer {
                gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
            }
            // create call inputs
            let call_inputs = Box::new(CallInputs {
                input: contract_input,
                gas_limit,
                target_address,
                caller: inputs.target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::Call,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            context.evm.make_call_frame(&call_inputs)
        }

        SYSCALL_ID_STATIC_CALL => {
            assert_return!(
                inputs.input.len() >= 20 && inputs.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.input[0..20]);
            let contract_input = inputs.input.slice(20..);
            let Ok(mut account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, false, account_load));
            let gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    inputs.gas.remaining_63_of_64_parts(),
                    inputs.local_gas_limit,
                )
            } else {
                inputs.local_gas_limit
            };
            charge_gas!(gas_limit);
            // create call inputs
            let inputs = Box::new(CallInputs {
                input: contract_input,
                gas_limit,
                target_address,
                caller: inputs.target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(U256::ZERO),
                scheme: CallScheme::StaticCall,
                is_static: true,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            let frame = context.evm.make_call_frame(&inputs)?;
            return_frame!(frame);
        }

        SYSCALL_ID_CALL_CODE => {
            assert_return!(
                inputs.input.len() >= 20 + 32 && inputs.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.input[0..20]);
            let value = U256::from_le_slice(&inputs.input[20..52]);
            let contract_input = inputs.input.slice(52..);

            let Ok(mut account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(
                SPEC::SPEC_ID,
                !value.is_zero(),
                account_load
            ));
            let mut gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    inputs.gas.remaining_63_of_64_parts(),
                    inputs.local_gas_limit,
                )
            } else {
                inputs.local_gas_limit
            };
            charge_gas!(gas_limit);
            // add call stipend if there is a value to be transferred
            if !value.is_zero() {
                gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
            }
            // create call inputs
            let inputs = Box::new(CallInputs {
                input: contract_input,
                gas_limit,
                target_address: inputs.target_address,
                caller: inputs.target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::CallCode,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            let frame = context.evm.make_call_frame(&inputs)?;
            return_frame!(frame);
        }

        SYSCALL_ID_DELEGATE_CALL => {
            assert_return!(
                inputs.input.len() >= 20 && inputs.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.input[0..20]);
            let contract_input = inputs.input.slice(20..);

            let Ok(mut account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, false, account_load));
            let gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    inputs.gas.remaining_63_of_64_parts(),
                    inputs.local_gas_limit,
                )
            } else {
                inputs.local_gas_limit
            };
            charge_gas!(gas_limit);
            // create call inputs
            let inputs = Box::new(CallInputs {
                input: contract_input,
                gas_limit,
                target_address: inputs.target_address,
                caller: inputs.caller,
                bytecode_address: target_address,
                value: CallValue::Apparent(inputs.call_value),
                scheme: CallScheme::DelegateCall,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            let frame = context.evm.make_call_frame(&inputs)?;
            return_frame!(frame);
        }

        SYSCALL_ID_CREATE | SYSCALL_ID_CREATE2 => {
            unreachable!("revm: unsupported system interruption")
        }

        SYSCALL_ID_EMIT_LOG => {
            assert_return!(
                inputs.input.len() >= 1 && inputs.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // read topics from input
            let topics_len = inputs.input[0] as usize;
            assert_return!(topics_len <= 4, Revert);
            let mut topics = Vec::with_capacity(topics_len);
            assert_return!(
                inputs.input.len() >= 1 + topics_len * B256::len_bytes(),
                Revert
            );
            for i in 0..topics_len {
                let offset = 1 + i * B256::len_bytes();
                let topic = &inputs.input.as_ref()[offset..(offset + B256::len_bytes())];
                topics.push(B256::from_slice(topic));
            }
            // all remaining bytes are data
            let data = inputs.input.slice((1 + topics_len * B256::len_bytes())..);
            // make sure we have enough gas to cover this operation
            let Some(gas_cost) = gas::log_cost(topics_len as u8, data.len() as u64) else {
                return_error!(OutOfGas);
            };
            charge_gas!(gas_cost);
            // write new log into the journal
            context.evm.journaled_state.log(Log {
                address: inputs.target_address,
                // it's safe to go unchecked here because we do topics check upper
                data: LogData::new_unchecked(topics, data),
            });
            return_result!(Bytes::default());
        }

        SYSCALL_ID_DESTROY_ACCOUNT => {
            assert_return!(
                inputs.input.len() == 20 && inputs.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // destroy an account
            let target = Address::from_slice(&inputs.input[0..20]);
            let result = context.evm.selfdestruct(inputs.target_address, target)?;
            // charge gas cost
            charge_gas!(gas::selfdestruct_cost(SPEC::SPEC_ID, result));
            // return value as bytes with success exit code
            return_result!(Bytes::default());
        }

        SYSCALL_ID_BALANCE => {
            assert_return!(
                inputs.input.len() == 20 && inputs.state == STATE_MAIN,
                Revert
            );
            let address = Address::from_slice(&inputs.input[0..20]);
            let value = context.evm.balance(address)?;
            // make sure we have enough gas for this op
            charge_gas!(if inputs.target_address == address {
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

        // TODO(dmitry123): "rethink these system calls"
        SYSCALL_ID_WRITE_PREIMAGE
        | SYSCALL_ID_PREIMAGE_COPY
        | SYSCALL_ID_PREIMAGE_SIZE
        | SYSCALL_ID_EXT_STORAGE_READ => {
            return_error!(Revert);
        }

        SYSCALL_ID_TRANSIENT_READ => {
            assert_return!(
                inputs.input.len() == 32 && inputs.state == STATE_MAIN,
                Revert
            );
            // read value from storage
            let slot = U256::from_le_slice(&inputs.input[0..32].as_ref());
            let value = context.evm.tload(inputs.target_address, slot);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            // return value
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_TRANSIENT_WRITE => {
            assert_return!(
                inputs.input.len() == 64 && inputs.state == STATE_MAIN,
                Revert
            );
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // read input
            let slot = U256::from_le_slice(&inputs.input[0..32]);
            let value = U256::from_le_slice(&inputs.input[32..64]);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            context.evm.tstore(inputs.target_address, slot, value);
            // empty result
            return_result!(Bytes::default());
        }

        _ => return_error!(Revert),
    }
}
