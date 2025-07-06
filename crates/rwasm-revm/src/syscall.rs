use crate::{
    frame::{ContextTrDbError, RwasmFrame},
    types::{SystemInterruptionInputs, SystemInterruptionOutcome},
};
use core::cmp::min;
use fluentbase_genesis::is_system_precompile;
use fluentbase_sdk::{
    byteorder::{ByteOrder, LittleEndian, ReadBytesExt},
    bytes::Buf,
    calc_create4_address,
    keccak256,
    Address,
    Bytes,
    Log,
    LogData,
    B256,
    FUEL_DENOM_RATE,
    STATE_MAIN,
    SVM_ELF_MAGIC_BYTES,
    SVM_MAX_CODE_SIZE,
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
    SYSCALL_ID_METADATA_COPY,
    SYSCALL_ID_METADATA_CREATE,
    SYSCALL_ID_METADATA_SIZE,
    SYSCALL_ID_METADATA_WRITE,
    SYSCALL_ID_SELF_BALANCE,
    SYSCALL_ID_STATIC_CALL,
    SYSCALL_ID_STORAGE_READ,
    SYSCALL_ID_STORAGE_WRITE,
    SYSCALL_ID_TRANSIENT_READ,
    SYSCALL_ID_TRANSIENT_WRITE,
    U256,
    WASM_MAGIC_BYTES,
    WASM_MAX_CODE_SIZE,
};
use revm::{
    bytecode::{ownable_account::OwnableAccountBytecode, Bytecode},
    context::{result::FromStringError, Cfg, ContextTr, CreateScheme, JournalTr},
    handler::EvmTr,
    interpreter::{
        gas,
        gas::{sload_cost, sstore_cost, sstore_refund, warm_cold_cost, CALL_STIPEND},
        interpreter::EthInterpreter,
        interpreter_types::InputsTr,
        CallInput,
        CallInputs,
        CallScheme,
        CallValue,
        CreateInputs,
        FrameInput,
        Gas,
        InstructionResult,
        InterpreterAction,
        InterpreterResult,
    },
    primitives::{
        hardfork::{SpecId, BERLIN, ISTANBUL, TANGERINE},
        MAX_INITCODE_SIZE,
    },
};
use std::{boxed::Box, vec::Vec};

pub(crate) fn execute_rwasm_interruption<
    EVM: EvmTr,
    ERROR: From<ContextTrDbError<EVM::Context>> + FromStringError,
>(
    frame: &mut RwasmFrame<EVM, ERROR, EthInterpreter>,
    evm: &mut EVM,
    inputs: SystemInterruptionInputs,
) -> Result<InterpreterAction, ERROR> {
    let mut local_gas = Gas::new(inputs.gas.remaining());
    let spec_id: SpecId = evm.ctx().cfg().spec().into();
    let journal = evm.ctx().journal();
    let current_target_address = frame.interpreter.input.target_address();
    let account_owner_address = frame.interpreter.input.account_owner_address();

    macro_rules! return_result {
        ($result:expr, $error:ident) => {{
            let result =
                InterpreterResult::new(InstructionResult::$error, $result.into(), local_gas);
            frame.insert_interrupted_outcome(SystemInterruptionOutcome {
                inputs: Box::new(inputs),
                // we have to clone result, because we don't know do we need to resume or not
                result: result.clone(),
                is_frame: false,
            });
            return Ok(InterpreterAction::Return { result });
        }};
        ($error:ident) => {{
            let result =
                InterpreterResult::new(InstructionResult::$error, Default::default(), local_gas);
            frame.insert_interrupted_outcome(SystemInterruptionOutcome {
                inputs: Box::new(inputs),
                // we have to clone result, because we don't know do we need to resume or not
                result: result.clone(),
                is_frame: false,
            });
            return Ok(InterpreterAction::Return { result });
        }};
    }
    macro_rules! return_frame {
        ($action:expr) => {{
            frame.insert_interrupted_outcome(SystemInterruptionOutcome {
                inputs: Box::new(inputs),
                result: InterpreterResult::new(
                    InstructionResult::Continue,
                    Bytes::default(),
                    local_gas,
                ),
                is_frame: true,
            });
            return Ok($action);
        }};
    }
    macro_rules! assert_return {
        ($cond:expr, $error:ident) => {
            if !($cond) {
                return_result!($error);
            }
        };
    }
    macro_rules! charge_gas {
        ($value:expr) => {{
            if !local_gas.record_cost($value) {
                return_result!(OutOfGas);
            }
        }};
    }

    match inputs.syscall_params.code_hash {
        SYSCALL_ID_STORAGE_READ => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_STORAGE_READ: slot={}", slot);
            // execute sload
            let value = journal.sload(current_target_address, slot)?;
            charge_gas!(sload_cost(spec_id, value.is_cold));
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output, Return)
        }

        SYSCALL_ID_STORAGE_WRITE => {
            assert_return!(
                inputs.syscall_params.input.len() == 32 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // don't allow for static context
            assert_return!(!inputs.is_static, StateChangeDuringStaticCall);
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            let new_value = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_STORAGE_WRITE: slot={slot}, new_value={new_value}");
            // execute sstore
            let value = journal.sstore(current_target_address, slot, new_value)?;
            if local_gas.remaining() <= CALL_STIPEND {
                return_result!(ReentrancySentryOOG);
            }
            let gas_cost = sstore_cost(spec_id.clone(), &value.data, value.is_cold);
            charge_gas!(gas_cost);
            local_gas.record_refund(sstore_refund(spec_id, &value.data));
            return_result!(Bytes::default(), Return)
        }

        SYSCALL_ID_CALL => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[20..52]);
            let contract_input = inputs.syscall_params.input.slice(52..);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_CALL: callee_address={target_address}, value={value}",);
            // for static calls with value greater than 0 - revert
            let has_transfer = !value.is_zero();
            if inputs.is_static && has_transfer {
                return_result!(CallNotAllowedInsideStatic);
            }
            let Ok(mut account_load) = journal.load_account_delegated(target_address) else {
                return_result!(FatalExternalError);
            };
            // In EVM, there exists an issue with precompiled contracts.
            // These contracts are preloaded and initially empty.
            // However, a precompiled contract can also be explicitly added
            // inside the genesis file, which affects its state and the gas
            // price for the CALL opcode.
            //
            // Using the CALL opcode to invoke a precompiled contract typically
            // has no practical use, as the contract is stateless.
            // Despite this, there are unit tests that require this condition
            // to be supported.
            //
            // While addressing this, improves compatibility with the EVM,
            // it also breaks several unit tests.
            // Nevertheless, the added compatibility is deemed to outweigh these issues.
            if is_system_precompile(&target_address) {
                account_load.is_empty = true;
            }
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(spec_id, has_transfer, account_load));
            let mut gas_limit = min(
                local_gas.remaining_63_of_64_parts(),
                inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE,
            );
            charge_gas!(gas_limit);
            if has_transfer {
                gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
            }
            // create call inputs
            let call_inputs = Box::new(CallInputs {
                input: CallInput::Bytes(contract_input),
                gas_limit,
                target_address,
                caller: current_target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::Call,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            return_frame!(InterpreterAction::NewFrame(FrameInput::Call(call_inputs)));
        }

        SYSCALL_ID_STATIC_CALL => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let contract_input = inputs.syscall_params.input.slice(20..);
            let Ok(mut account_load) = journal.load_account_delegated(target_address) else {
                return_result!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(spec_id.clone(), false, account_load));
            let gas_limit = if spec_id.is_enabled_in(TANGERINE) {
                min(
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE,
                )
            } else {
                inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE
            };
            charge_gas!(gas_limit);

            #[cfg(feature = "debug-print")]
            println!("SYSCALL_ID_STATIC_CALL: target_address={target_address}");
            // create call inputs
            let call_inputs = Box::new(CallInputs {
                input: CallInput::Bytes(contract_input),
                gas_limit,
                target_address,
                caller: current_target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(U256::ZERO),
                scheme: CallScheme::StaticCall,
                is_static: true,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            return_frame!(InterpreterAction::NewFrame(FrameInput::Call(call_inputs)));
        }

        SYSCALL_ID_CALL_CODE => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20 + 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[20..52]);
            let contract_input = inputs.syscall_params.input.slice(52..);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_CALL_CODE: target_address={target_address}, value={value}");
            let Ok(mut account_load) = journal.load_account_delegated(target_address) else {
                return_result!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(spec_id, !value.is_zero(), account_load));
            let mut gas_limit = if spec_id.is_enabled_in(TANGERINE) {
                min(
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE,
                )
            } else {
                inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE
            };
            charge_gas!(gas_limit);
            // add call stipend if there is a value to be transferred
            if !value.is_zero() {
                gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
            }
            // create call inputs
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_CALL_CODE_inputs: target_address={}, caller={}, bytecode_address={}, gas={:?}", target_address, target_address, target_address, gas_limit);
            let call_inputs = Box::new(CallInputs {
                input: CallInput::Bytes(contract_input),
                gas_limit,
                target_address: current_target_address,
                caller: current_target_address,
                bytecode_address: target_address,
                value: CallValue::Transfer(value),
                scheme: CallScheme::CallCode,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            return_frame!(InterpreterAction::NewFrame(FrameInput::Call(call_inputs)));
        }

        SYSCALL_ID_DELEGATE_CALL => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let target_address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let contract_input = inputs.syscall_params.input.slice(20..);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_DELEGATE_CALL: target_address={target_address}");
            let Ok(mut account_load) = journal.load_account_delegated(target_address) else {
                return_result!(FatalExternalError);
            };
            // set is_empty to false as we are not creating this account.
            account_load.is_empty = false;
            // EIP-150: gas cost changes for IO-heavy operations
            charge_gas!(gas::call_cost(spec_id, false, account_load));
            let gas_limit = if spec_id.is_enabled_in(TANGERINE) {
                min(
                    local_gas.remaining_63_of_64_parts(),
                    inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE,
                )
            } else {
                inputs.syscall_params.fuel_limit / FUEL_DENOM_RATE
            };
            charge_gas!(gas_limit);
            // create call inputs
            let call_inputs = Box::new(CallInputs {
                input: CallInput::Bytes(contract_input),
                gas_limit,
                target_address: current_target_address,
                caller: frame.interpreter.input.caller_address(),
                bytecode_address: target_address,
                value: CallValue::Apparent(frame.interpreter.input.call_value()),
                scheme: CallScheme::DelegateCall,
                is_static: inputs.is_static,
                is_eof: false,
                return_memory_offset: Default::default(),
            });
            return_frame!(InterpreterAction::NewFrame(FrameInput::Call(call_inputs)));
        }

        SYSCALL_ID_CREATE | SYSCALL_ID_CREATE2 => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, StateChangeDuringStaticCall);
            // make sure we have enough bytes inside input params
            let is_create2 = inputs.syscall_params.code_hash == SYSCALL_ID_CREATE2;
            let (scheme, value, init_code) = if is_create2 {
                assert_return!(
                    inputs.syscall_params.input.len() >= 32 + 32,
                    MalformedBuiltinParams
                );
                let value = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
                let salt = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
                let init_code = inputs.syscall_params.input.slice(64..);
                (CreateScheme::Create2 { salt }, value, init_code)
            } else {
                assert_return!(
                    inputs.syscall_params.input.len() >= 32,
                    MalformedBuiltinParams
                );
                let value = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
                let init_code = inputs.syscall_params.input.slice(32..);
                (CreateScheme::Create, value, init_code)
            };
            #[cfg(feature = "debug-print")]
            println!(
                "SYSCALL_CREATE/CREATE2: scheme={scheme:?} value={value} init_code_len={}",
                init_code.len()
            );
            // make sure we don't exceed max possible init code
            // TODO(khasan): take into consideration evm.ctx().cfg().max_init_code
            let max_initcode_size = if init_code.len() >= 4 && init_code[0..4] == WASM_MAGIC_BYTES {
                WASM_MAX_CODE_SIZE
            } else if init_code.len() >= 4 && init_code[0..4] == SVM_ELF_MAGIC_BYTES {
                SVM_MAX_CODE_SIZE
            } else {
                MAX_INITCODE_SIZE
            };
            assert_return!(
                init_code.len() <= max_initcode_size,
                CreateContractSizeLimit
            );
            if !init_code.is_empty() {
                charge_gas!(gas::initcode_cost(init_code.len()));
            }
            if is_create2 {
                let Some(gas) = gas::create2_cost(init_code.len().try_into().unwrap()) else {
                    return_result!(OutOfGas);
                };
                charge_gas!(gas);
            } else {
                charge_gas!(gas::CREATE);
            };
            let mut gas_limit = local_gas.remaining();
            gas_limit -= gas_limit / 64;
            charge_gas!(gas_limit);
            // create inputs
            let create_inputs = Box::new(CreateInputs {
                caller: current_target_address,
                scheme,
                value,
                init_code,
                gas_limit,
            });
            return_frame!(InterpreterAction::NewFrame(FrameInput::Create(
                create_inputs
            )));
        }

        SYSCALL_ID_EMIT_LOG => {
            assert_return!(
                inputs.syscall_params.input.len() >= 1 && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, StateChangeDuringStaticCall);
            // read topics from input
            let topics_len = inputs.syscall_params.input[0] as usize;
            assert_return!(topics_len <= 4, MalformedBuiltinParams);
            let mut topics = Vec::with_capacity(topics_len);
            assert_return!(
                inputs.syscall_params.input.len() >= 1 + topics_len * B256::len_bytes(),
                MalformedBuiltinParams
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
                return_result!(OutOfGas);
            };
            charge_gas!(gas_cost);
            // write new log into the journal
            journal.log(Log {
                address: current_target_address,
                // it's safe to go unchecked here because we do topics check upper
                data: LogData::new_unchecked(topics, data),
            });
            return_result!(Bytes::new(), Return);
        }

        SYSCALL_ID_DESTROY_ACCOUNT => {
            assert_return!(
                inputs.syscall_params.input.len() == 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // not allowed for static calls
            assert_return!(!inputs.is_static, StateChangeDuringStaticCall);
            // destroy an account
            let target = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let result = journal.selfdestruct(current_target_address, target)?;
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_DESTROY_ACCOUNT: target={target} result={result:?}",);
            // charge gas cost
            charge_gas!(gas::selfdestruct_cost(spec_id, result));
            // return value as bytes with success exit code
            return_result!(Bytes::new(), Return);
        }

        SYSCALL_ID_BALANCE => {
            assert_return!(
                inputs.syscall_params.input.len() == 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let value = journal
                .load_account(address)
                .map(|acc| acc.map(|a| a.info.balance))?;
            // make sure we have enough gas for this op
            charge_gas!(if spec_id.is_enabled_in(BERLIN) {
                warm_cold_cost(value.is_cold)
            } else if spec_id.is_enabled_in(ISTANBUL) {
                700
            } else if spec_id.is_enabled_in(TANGERINE) {
                400
            } else {
                20
            });
            // write the result
            let output: [u8; 32] = value.data.to_le_bytes();
            return_result!(output, Return);
        }

        SYSCALL_ID_SELF_BALANCE => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            let value = journal
                .load_account(current_target_address)
                .map(|acc| acc.map(|a| a.info.balance))?;
            charge_gas!(gas::LOW);
            let output: [u8; 32] = value.data.to_le_bytes();
            return_result!(output, Return)
        }

        SYSCALL_ID_CODE_SIZE => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20,
                MalformedBuiltinParams
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_CODE_SIZE: address={address}");
            let code = journal.code(address)?;
            charge_gas!(if spec_id.is_enabled_in(BERLIN) {
                warm_cold_cost(code.is_cold)
            } else if spec_id.is_enabled_in(TANGERINE) {
                700
            } else {
                20
            });
            let mut code_len = code.data.len() as u32;
            // we store system precompile bytecode in the state trie,
            // according to evm requirements, we should return empty code
            if is_system_precompile(&address) {
                code_len = 0;
            }
            let code_size = U256::from(code_len);
            let output = code_size.to_le_bytes::<32>();
            return_result!(output, Return);
        }

        SYSCALL_ID_CODE_HASH => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20,
                MalformedBuiltinParams
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let code_hash = journal.code_hash(address)?;
            #[cfg(feature = "debug-print")]
            println!(
                "SYSCALL_CODE_HASH: address={address} code_hash={}",
                code_hash.data,
            );
            charge_gas!(if spec_id.is_enabled_in(BERLIN) {
                warm_cold_cost(code_hash.is_cold)
            } else if spec_id.is_enabled_in(TANGERINE) {
                700
            } else {
                400
            });
            let mut code_hash = code_hash.data;
            // we store system precompile bytecode in the state trie,
            // according to evm requirements, we should return empty code
            if is_system_precompile(&address) {
                code_hash = B256::ZERO;
            }
            return_result!(code_hash, Return);
        }

        SYSCALL_ID_CODE_COPY => {
            assert_return!(
                inputs.syscall_params.state == STATE_MAIN
                    && inputs.syscall_params.input.len() == 20 + 8 * 2,
                MalformedBuiltinParams
            );
            let address = Address::from_slice(&inputs.syscall_params.input[0..20]);
            let mut reader = inputs.syscall_params.input[20..].reader();
            let _code_offset = reader.read_u64::<LittleEndian>().unwrap();
            let code_length = reader.read_u64::<LittleEndian>().unwrap();
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_CODE_COPY: address={address} code_offset={_code_offset} code_length={code_length}");
            let code = journal.code(address)?;
            let Some(gas_cost) = gas::extcodecopy_cost(spec_id, code_length as usize, code.is_cold)
            else {
                return_result!(OutOfGas);
            };
            charge_gas!(gas_cost);
            if code_length == 0 {
                return_result!(Bytes::new(), Return);
            }
            let mut bytecode = code.data;
            // we store system precompile bytecode in the state trie,
            // according to evm requirements, we should return empty code
            if is_system_precompile(&address) {
                bytecode = Bytes::new();
            }
            // TODO(dmitry123): "add offset/length checks"
            return_result!(bytecode, Return);
        }

        SYSCALL_ID_METADATA_SIZE => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // syscall is allowed only for accounts that are owned by somebody
            let Some(account_owner_address) = account_owner_address else {
                return_result!(MalformedBuiltinParams);
            };
            // read an account from its address
            let address = Address::from_slice(&inputs.syscall_params.input[..20]);
            let Ok(mut account) = journal.load_account_code(address) else {
                return_result!(FatalExternalError);
            };
            // to make sure this account is ownable and owner by the same runtime, that allows
            // a runtime to modify any account it owns
            let Some(ownable_account_bytecode) = (match account.info.code.as_mut() {
                Some(Bytecode::OwnableAccount(ownable_account_bytecode)) => {
                    // if an account is not the same - it's not a malformed building param, runtime might not know it's account
                    if ownable_account_bytecode.owner_address == account_owner_address {
                        Some(ownable_account_bytecode)
                    } else {
                        None
                    }
                }
                _ => None,
            }) else {
                let output = Bytes::from([
                    // metadata length is 0 in this case
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    // pass info about an account (is_account_ownable, is_cold, is_empty)
                    0x00u8,
                    account.is_cold as u8,
                    account.is_empty() as u8,
                ]);
                return_result!(output, Return);
            };
            // execute a syscall
            assert_return!(
                inputs.syscall_params.input.len() == 20,
                MalformedBuiltinParams
            );
            let mut output = [0u8; 4 + 3];
            LittleEndian::write_u32(&mut output, ownable_account_bytecode.metadata.len() as u32);
            #[cfg(feature = "debug-print")]
            println!(
                "SYSCALL_METADATA_SIZE: address={address} metadata_size={}",
                ownable_account_bytecode.metadata.len() as u32
            );
            output[4] = 0x01u8; // the account belongs to the same runtime
            output[5] = account.is_cold as u8;
            output[6] = account.is_empty() as u8;
            return_result!(output, Return)
        }
        SYSCALL_ID_METADATA_CREATE => {
            assert_return!(
                inputs.syscall_params.input.len() >= 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // syscall is allowed only for accounts that are owned by somebody
            let Some(account_owner_address) = account_owner_address else {
                return_result!(MalformedBuiltinParams);
            };
            // read an account from its address
            let salt = U256::from_be_slice(&inputs.syscall_params.input[..32]);
            let metadata = inputs.syscall_params.input.slice(32..);
            let derived_metadata_address =
                calc_create4_address(&account_owner_address, &salt, |v| keccak256(v));
            let Ok(account) = journal.load_account_code(derived_metadata_address) else {
                return_result!(FatalExternalError);
            };
            #[cfg(feature = "debug-print")]
            println!(
                "SYSCALL_METADATA_CREATE: address={derived_metadata_address} salt={salt} length={}",
                metadata.len(),
            );
            // make sure there is no account create collision
            if !account.is_empty() {
                return_result!(CreateCollision);
            }
            // create new derived ownable account
            journal.set_code(
                derived_metadata_address,
                Bytecode::OwnableAccount(OwnableAccountBytecode::new(
                    account_owner_address,
                    metadata.clone(),
                )),
            );
            return_result!(Bytes::new(), Return)
        }

        SYSCALL_ID_METADATA_WRITE | SYSCALL_ID_METADATA_COPY => {
            assert_return!(
                inputs.syscall_params.input.len() >= 20
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // syscall is allowed only for accounts that are owned by somebody
            let Some(account_owner_address) = account_owner_address else {
                return_result!(MalformedBuiltinParams);
            };
            // read an account from its address
            let address = Address::from_slice(&inputs.syscall_params.input[..20]);
            let Ok(mut account) = journal.load_account_code(address) else {
                return_result!(FatalExternalError);
            };
            // to make sure this account is ownable and owner by the same runtime, that allows
            // a runtime to modify any account it owns
            let ownable_account_bytecode = match account.info.code.as_mut() {
                Some(Bytecode::OwnableAccount(ownable_account_bytecode))
                    if ownable_account_bytecode.owner_address == account_owner_address =>
                {
                    ownable_account_bytecode
                }
                _ => {
                    return_result!(Bytes::new(), MalformedBuiltinParams)
                }
            };
            // execute a syscall
            match inputs.syscall_params.code_hash {
                SYSCALL_ID_METADATA_WRITE => {
                    assert_return!(
                        inputs.syscall_params.input.len() >= 20 + 4,
                        MalformedBuiltinParams
                    );
                    let offset =
                        LittleEndian::read_u32(&inputs.syscall_params.input[20..24]) as usize;
                    let length = inputs.syscall_params.input[24..].len();
                    #[cfg(feature = "debug-print")]
                    println!(
                        "SYSCALL_METADATA_WRITE: address={address} offset={}, length={}",
                        offset, length,
                    );
                    // TODO(dmitry123): "figure out a way how to optimize it"
                    let mut metadata = ownable_account_bytecode.metadata.to_vec();
                    metadata.resize(offset + length, 0);
                    metadata[offset..(offset + length)]
                        .copy_from_slice(&inputs.syscall_params.input[24..]);
                    ownable_account_bytecode.metadata = metadata.into();
                    // code hash might change, rewrite it
                    account.info.code_hash = account.info.code.as_ref().unwrap().hash_slow();
                    journal.touch_account(address);
                    return_result!(Bytes::new(), Return)
                }
                SYSCALL_ID_METADATA_COPY => {
                    assert_return!(
                        inputs.syscall_params.input.len() == 28,
                        MalformedBuiltinParams
                    );
                    let offset = LittleEndian::read_u32(&inputs.syscall_params.input[20..24]);
                    let length = LittleEndian::read_u32(&inputs.syscall_params.input[24..28]);
                    #[cfg(feature = "debug-print")]
                    println!(
                        "SYSCALL_METADATA_COPY: address={address} offset={}, length={}, metadata_length={}",
                        offset, length, ownable_account_bytecode.metadata.len(),
                    );
                    // take min
                    let length = length.min(ownable_account_bytecode.metadata.len() as u32);
                    let metadata = ownable_account_bytecode
                        .metadata
                        .slice(offset as usize..(offset + length) as usize);
                    return_result!(metadata, Return)
                }
                _ => unreachable!(),
            }
        }

        SYSCALL_ID_TRANSIENT_READ => {
            assert_return!(
                inputs.syscall_params.input.len() == 32
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            // read value from storage
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32].as_ref());
            let value = journal.tload(current_target_address, slot);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_TRANSIENT_READ: slot={slot} value={value}");
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            // return value
            let output: [u8; 32] = value.to_le_bytes();
            return_result!(output, Return);
        }

        SYSCALL_ID_TRANSIENT_WRITE => {
            assert_return!(
                inputs.syscall_params.input.len() == 64
                    && inputs.syscall_params.state == STATE_MAIN,
                MalformedBuiltinParams
            );
            assert_return!(!inputs.is_static, StateChangeDuringStaticCall);
            // read input
            let slot = U256::from_le_slice(&inputs.syscall_params.input[0..32]);
            let value = U256::from_le_slice(&inputs.syscall_params.input[32..64]);
            #[cfg(feature = "debug-print")]
            println!("SYSCALL_TRANSIENT_WRITE: slot={slot} value={value}");
            // charge gas
            charge_gas!(gas::WARM_STORAGE_READ_COST);
            journal.tstore(current_target_address, slot, value);
            // empty result
            return_result!(Bytes::new(), Return);
        }

        _ => return_result!(MalformedBuiltinParams),
    }
}
