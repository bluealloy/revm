use crate::{
    interpreter::{
        gas,
        gas::sstore_cost,
        interpreter_action::SystemInterruptionInputs,
        CallInputs,
        CallOutcome,
        CallScheme,
        CallValue,
        CreateInputs,
        InstructionResult,
        InterpreterResult,
    },
    primitives::{
        bytes::Buf,
        wasm::{WASM_MAGIC_BYTES, WASM_MAX_CODE_SIZE},
        Address,
        Bytecode,
        Bytes,
        CreateScheme,
        EVMError,
        Log,
        LogData,
        Spec,
        SpecId,
        B256,
        BERLIN,
        ISTANBUL,
        MAX_INITCODE_SIZE,
        TANGERINE,
        U256,
    },
    Context,
    Database,
    FrameOrResult,
    FrameResult,
};
use core::cmp::min;
use fluentbase_sdk::{
    byteorder::{LittleEndian, ReadBytesExt},
    keccak256,
    STATE_MAIN,
    SYSCALL_ID_BALANCE,
    SYSCALL_ID_CALL,
    SYSCALL_ID_CALL_CODE,
    SYSCALL_ID_CODE_COPY,
    SYSCALL_ID_CODE_HASH,
    SYSCALL_ID_CODE_SIZE,
    SYSCALL_ID_CREATE,
    SYSCALL_ID_CREATE2,
    SYSCALL_ID_DELEGATE_CALL,
    SYSCALL_ID_DESTROY_ACCOUNT,
    SYSCALL_ID_EMIT_LOG,
    SYSCALL_ID_EXT_STORAGE_READ,
    SYSCALL_ID_PREIMAGE_COPY,
    SYSCALL_ID_PREIMAGE_SIZE,
    SYSCALL_ID_SELF_BALANCE,
    SYSCALL_ID_STATIC_CALL,
    SYSCALL_ID_STORAGE_READ,
    SYSCALL_ID_STORAGE_WRITE,
    SYSCALL_ID_TRANSIENT_READ,
    SYSCALL_ID_TRANSIENT_WRITE,
    SYSCALL_ID_WRITE_PREIMAGE,
};
use revm_interpreter::{
    gas::{sload_cost, sstore_refund, warm_cold_cost},
    Gas,
    Host,
};

pub(crate) fn execute_rwasm_interruption<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    inputs: &Box<SystemInterruptionInputs>,
) -> Result<(FrameOrResult, Gas), EVMError<DB::Error>> {
    let mut local_gas = Gas::new(inputs.gas.remaining());

    macro_rules! return_result {
        ($output:expr) => {{
            let result =
                InterpreterResult::new(InstructionResult::Return, $output.into(), local_gas);
            let result = FrameOrResult::Result(FrameResult::Call(CallOutcome::new(
                result,
                Default::default(),
            )));
            return Ok((result, Gas::new(0)));
        }};
    }
    macro_rules! return_error {
        ($error:ident) => {{
            let result =
                InterpreterResult::new(InstructionResult::$error, Default::default(), local_gas);
            let result = FrameOrResult::Result(FrameResult::Call(CallOutcome::new(
                result,
                Default::default(),
            )));
            return Ok((result, Gas::new(0)));
        }};
    }
    macro_rules! return_frame {
        ($frame:expr) => {
            return Ok(($frame, local_gas));
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
        ($value:expr) => {{
            if !local_gas.record_cost($value) {
                return_error!(OutOfGas);
            }
        }};
    }

    match inputs.syscall_params.code_hash {
        SYSCALL_ID_STORAGE_READ => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            // execute sload
            let value = context.evm.sload(inputs.target_address, slot)?;
            charge_gas!(sload_cost(SPEC::SPEC_ID, value.is_cold));
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output)
        }

        SYSCALL_ID_STORAGE_WRITE => {
            assert_return!(
                inputs.syscall_params.input.len() == 32 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            // don't allow for static context
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            let new_value = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
            // execute sstore
            let value = context.evm.sstore(inputs.target_address, slot, new_value)?;
            if let Some(gas_cost) = sstore_cost(
                SPEC::SPEC_ID,
                &value.data,
                local_gas.remaining(),
                value.is_cold,
            ) {
                charge_gas!(gas_cost);
            } else {
                return_error!(OutOfGas);
            }
            local_gas.record_refund(sstore_refund(SPEC::SPEC_ID, &value.data));
            return_result!(Bytes::default())
        }

        SYSCALL_ID_CALL => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[20..52]);
            let contract_input = inputs.syscall_params.input.slice(52..);
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
            let mut gas_limit = min(
                local_gas.remaining_63_of_64_parts(),
                inputs.syscall_params.gas_limit,
            );
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
            let frame = context.evm.make_call_frame(&call_inputs)?;
            return_frame!(frame);
        }

        SYSCALL_ID_STATIC_CALL => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let contract_input = inputs.syscall_params.input.slice(20..);
            let Ok(mut account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, false, account_load));
            let gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.gas_limit,
                )
            } else {
                inputs.syscall_params.gas_limit
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
                inputs.syscall_params.input.len() >= 20 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[20..52]);
            let contract_input = inputs.syscall_params.input.slice(52..);

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
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.gas_limit,
                )
            } else {
                inputs.syscall_params.gas_limit
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
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let contract_input = inputs.syscall_params.input.slice(20..);

            let Ok(mut account_load) = context.evm.load_account_delegated(target_address) else {
                return_error!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(SPEC::SPEC_ID, false, account_load));
            let gas_limit = if SPEC::enabled(TANGERINE) {
                min(
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.gas_limit,
                )
            } else {
                inputs.syscall_params.gas_limit
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
            assert_return!(inputs.syscall_params.state == STATE_MAIN, Revert);
            // not allowed for static calls
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // make sure we have enough bytes inside input params
            let is_create2 = inputs.syscall_params.code_hash == SYSCALL_ID_CREATE2;
            let (scheme, value, init_code) = if is_create2 {
                assert_return!(inputs.syscall_params.input.len() >= 32 + 32, Revert);
                let value = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
                let salt = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
                let init_code = inputs.syscall_params.input.slice(64..);
                (CreateScheme::Create2 { salt }, value, init_code)
            } else {
                assert_return!(inputs.syscall_params.input.len() >= 32, Revert);
                let value = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
                let init_code = inputs.syscall_params.input.slice(32..);
                (CreateScheme::Create, value, init_code)
            };
            // make sure we don't exceed max possible init code
            let max_initcode_size = context
                .evm
                .env
                .cfg
                .limit_contract_code_size
                .map(|limit| limit.saturating_mul(2))
                .unwrap_or_else(|| {
                    if init_code.len() >= 4 && init_code[0..4] == WASM_MAGIC_BYTES {
                        WASM_MAX_CODE_SIZE
                    } else {
                        MAX_INITCODE_SIZE
                    }
                });
            assert_return!(
                init_code.len() <= max_initcode_size,
                CreateContractSizeLimit
            );
            if !init_code.is_empty() {
                charge_gas!(gas::initcode_cost(init_code.len() as u64));
            }
            if is_create2 {
                let Some(gas) = gas::create2_cost(init_code.len().try_into().unwrap()) else {
                    return_error!(OutOfGas);
                };
                charge_gas!(gas);
            } else {
                charge_gas!(gas::CREATE);
            };
            let mut gas_limit = local_gas.remaining();
            gas_limit -= gas_limit / 64;
            charge_gas!(gas_limit);
            // create inputs
            let inputs = Box::new(CreateInputs {
                caller: inputs.target_address,
                scheme,
                value,
                init_code,
                gas_limit,
            });
            let frame = context.evm.make_create_frame(SpecId::CANCUN, &inputs)?;
            return_frame!(frame);
        }

        SYSCALL_ID_EMIT_LOG => {
            assert_return!(
                inputs.syscall_params.input.len() >= 1 && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // read topics from input
            let topics_len = inputs.syscall_params.input[0] as usize;
            assert_return!(topics_len <= 4, Revert);
            let mut topics = Vec::with_capacity(topics_len);
            assert_return!(
                inputs.syscall_params.input.len() >= 1 + topics_len * B256::len_bytes(),
                Revert
            );
            for i in 0..topics_len {
                let offset = 1 + i * B256::len_bytes();
                let topic =
                    &inputs.syscall_params.input.as_ref()[offset..(offset + B256::len_bytes())];
                topics.push(B256::from_slice(topic));
            }
            // all remaining bytes are data
            let data = inputs
                .syscall_params
                .input
                .slice((1 + topics_len * B256::len_bytes())..);
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
                inputs.syscall_params.input.len() == 20
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // destroy an account
            let target = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let result = context.evm.selfdestruct(inputs.target_address, target)?;
            // charge gas cost
            charge_gas!(gas::selfdestruct_cost(SPEC::SPEC_ID, result));
            // return value as bytes with success exit code
            return_result!(Bytes::default());
        }

        SYSCALL_ID_BALANCE => {
            assert_return!(
                inputs.syscall_params.input.len() == 20
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = context.evm.balance(address)?;
            // make sure we have enough gas for this op
            charge_gas!(if SPEC::enabled(BERLIN) {
                warm_cold_cost(value.is_cold)
            } else if SPEC::enabled(ISTANBUL) {
                700
            } else if SPEC::enabled(TANGERINE) {
                400
            } else {
                20
            });
            // write the result
            let output: [u8; 32] = value.data.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_SELF_BALANCE => {
            assert_return!(inputs.syscall_params.state == STATE_MAIN, Revert);
            let value = context.evm.balance(inputs.target_address)?;
            charge_gas!(gas::LOW);
            let output: [u8; 32] = value.data.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_CODE_SIZE => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20,
                Revert
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let Some(code) = context.code(address) else {
                return_error!(FatalExternalError);
            };
            charge_gas!(if SPEC::enabled(BERLIN) {
                warm_cold_cost(code.is_cold)
            } else if SPEC::enabled(TANGERINE) {
                700
            } else {
                20
            });
            let code_size = U256::from(code.data.len() as u32);
            let output = code_size.to_le_bytes::<32>();
            return_result!(output);
        }

        SYSCALL_ID_CODE_HASH => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20,
                Revert
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let Some(code) = context.code_hash(address) else {
                return_error!(FatalExternalError);
            };
            charge_gas!(if SPEC::enabled(BERLIN) {
                warm_cold_cost(code.is_cold)
            } else if SPEC::enabled(TANGERINE) {
                700
            } else {
                400
            });
            let code_hash = code.data;
            return_result!(code_hash);
        }

        SYSCALL_ID_CODE_COPY => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20 + 8 * 2,
                Revert
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let mut reader = inputs.syscall_params.input[20..].reader();
            let code_offset = reader.read_u64::<LittleEndian>().unwrap();
            let code_length = reader.read_u64::<LittleEndian>().unwrap();
            let Some(code) = context.code(address) else {
                return_error!(FatalExternalError);
            };
            let Some(gas_cost) = gas::extcodecopy_cost(SPEC::SPEC_ID, code_length, code.is_cold)
            else {
                return_error!(OutOfGas);
            };
            charge_gas!(gas_cost);
            if code_length == 0 {
                return_result!(Bytes::default());
            }
            let code_offset = min(code_offset, code.len() as u64);
            let code = code
                .data
                .slice(code_offset as usize..(code_offset + code_length) as usize);
            return_result!(code);
        }

        // TODO(dmitry123): "rethink these system calls"
        SYSCALL_ID_WRITE_PREIMAGE => {
            assert_return!(inputs.syscall_params.state == STATE_MAIN, Revert);
            let preimage_hash = keccak256(inputs.syscall_params.input.as_ref());
            let address = Address::from_slice(&preimage_hash[12..]);
            let Ok(account_load) = context.evm.load_account_delegated(address) else {
                return_error!(FatalExternalError);
            };
            if account_load.is_empty {
                context.evm.journaled_state.set_code_with_hash(
                    address,
                    Bytecode::new_legacy(inputs.syscall_params.input.clone()),
                    preimage_hash,
                );
            }
            return_result!(preimage_hash);
        }
        SYSCALL_ID_PREIMAGE_SIZE => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let preimage_hash = B256::from_slice(&inputs.syscall_params.input[0..32]);
            let address = Address::from_slice(&preimage_hash[12..]);
            let Ok(account_load) = context.evm.load_account_delegated(address) else {
                return_error!(FatalExternalError);
            };
            let preimage_size = if !account_load.is_empty {
                let Some(code) = context.code(address) else {
                    return_error!(FatalExternalError);
                };
                code.data.len() as u32
            } else {
                0
            };
            return_result!(preimage_size.to_le_bytes());
        }
        SYSCALL_ID_PREIMAGE_COPY => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            let preimage_hash = B256::from_slice(&inputs.syscall_params.input[0..32]);
            let address = Address::from_slice(&preimage_hash[12..]);
            let Ok(account_load) = context.evm.code(address) else {
                return_error!(FatalExternalError);
            };
            return_result!(account_load.data);
        }

        SYSCALL_ID_EXT_STORAGE_READ => {
            return_error!(Revert);
        }

        SYSCALL_ID_TRANSIENT_READ => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            // read value from storage
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32].as_ref());
            let value = context.evm.tload(inputs.target_address, slot);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            // return value
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output);
        }

        SYSCALL_ID_TRANSIENT_WRITE => {
            assert_return!(
                inputs.syscall_params.input.len() == 64
                    && inputs.syscall_params.state == STATE_MAIN,
                Revert
            );
            assert_return!(!inputs.is_static, CallNotAllowedInsideStatic);
            // read input
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            context.evm.tstore(inputs.target_address, slot, value);
            // empty result
            return_result!(Bytes::default());
        }

        _ => return_error!(Revert),
    }
}
