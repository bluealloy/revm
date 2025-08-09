use super::{
    merkle_trie::{log_rlp_hash, state_merkle_trie_root},
    models::{SpecName, Test, TestSuite},
    utils::recover_address,
};
use fluentbase_genesis::GENESIS_CONTRACTS_BY_ADDRESS;
use fluentbase_revm::{RwasmBuilder, RwasmContext, RwasmEvm};
use fluentbase_sdk::{Address, PRECOMPILE_EVM_RUNTIME};
use hashbrown::HashSet;
use indicatif::{ProgressBar, ProgressDrawTarget};
use revm::primitives::eip4844::BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE;
use revm::{
    bytecode::{ownable_account::OwnableAccountBytecode, Bytecode},
    context::{
        result::ExecutionResult, transaction::AccessListItem, BlockEnv, CfgEnv, TransactTo,
        TransactionType::Eip1559, TxEnv,
    },
    context_interface::block::calc_excess_blob_gas,
    database::{CacheState, InMemoryDB, State, StateBuilder},
    handler::MainnetContext,
    primitives::{hardfork::SpecId, keccak256, Bytes, B256, U256},
    state::AccountInfo,
    ExecuteCommitEvm, MainBuilder, MainnetEvm,
};
use serde_json::json;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Error)]
#[error("Test {name} failed: {kind}")]
pub struct TestError {
    pub name: String,
    pub kind: TestErrorKind,
}

#[derive(Debug, Error)]
pub enum TestErrorKind {
    #[error("logs root mismatch: got {got}, expected {expected}")]
    LogsRootMismatch { got: B256, expected: B256 },
    #[error("state root mismatch: got {got}, expected {expected}")]
    StateRootMismatch { got: B256, expected: B256 },
    #[error("state root mismatch2: got {got}, expected {expected}")]
    StateRootMismatch2 { got: B256, expected: B256 },
    #[error("unknown private key: {0:?}")]
    UnknownPrivateKey(B256),
    #[error("unexpected exception: got {got_exception:?}, expected {expected_exception:?}")]
    UnexpectedException {
        expected_exception: Option<String>,
        got_exception: Option<String>,
    },
    #[error("unexpected output: got {got_output:?}, expected {expected_output:?}")]
    UnexpectedOutput {
        expected_output: Option<Bytes>,
        got_output: Option<Bytes>,
    },
    #[error(transparent)]
    SerdeDeserialize(#[from] serde_json::Error),
    #[error("thread panicked")]
    Panic,
}

pub fn find_all_json_tests(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension() == Some("json".as_ref()))
            .map(DirEntry::into_path)
            .collect()
    }
}

fn skip_test(path: &Path) -> bool {
    let path_str = path.to_str().expect("Path is not valid UTF-8");
    let name = path.file_name().unwrap().to_str().unwrap();

    matches!(
        name,
        // funky test with `bigint 0x00` value in json :) not possible to happen on mainnet and
        // require custom json parser. https://github.com/ethereum/tests/issues/971
        |"ValueOverflow.json"| "ValueOverflowParis.json"

        // precompiles having storage is not possible
        | "RevertPrecompiledTouch_storage.json"
        | "RevertPrecompiledTouch.json"

        // txbyte is of type 02 and we don't parse tx bytes for this test to fail.
        | "typeTwoBerlin.json"

        // Need to handle Test errors
        | "transactionIntinsicBug.json"

        // Test check if gas price overflows, we handle this correctly but does not match tests specific exception.
        | "HighGasPrice.json"
        | "CREATE_HighNonce.json"
        | "CREATE_HighNonceMinus1.json"
        | "CreateTransactionHighNonce.json"

        // Skip test where basefee/accesslist/difficulty is present but it shouldn't be supported in
        // London/Berlin/TheMerge. https://github.com/ethereum/tests/blob/5b7e1ab3ffaf026d99d20b17bb30f533a2c80c8b/GeneralStateTests/stExample/eip1559.json#L130
        // It is expected to not execute these tests.
        | "basefeeExample.json"
        | "eip1559.json"
        | "mergeTest.json"

        // These tests are passing, but they take a lot of time to execute so we are going to skip them.
        | "loopExp.json"
        | "Call50000_sha256.json"
        | "static_Call50000_sha256.json"
        | "loopMul.json"
        | "CALLBlake2f_MaxRounds.json"
    ) || path_str.contains("stEOF")
}

fn check_evm_execution<ERROR: Debug + ToString + Clone>(
    test: &Test,
    _spec_name: &SpecName,
    expected_output: Option<&Bytes>,
    test_name: &str,
    exec_result1: &Result<ExecutionResult, ERROR>,
    exec_result2: &Result<ExecutionResult, ERROR>,
    evm: &mut MainnetEvm<MainnetContext<State<InMemoryDB>>>,
    evm2: &mut RwasmEvm<RwasmContext<State<InMemoryDB>>>,
    print_json_outcome: bool,
    genesis_addresses: &HashSet<Address>,
) -> Result<(), TestError> {
    if !exec_result1.is_err() && exec_result2.is_err() {
        exec_result2.as_ref().unwrap();
    }

    let logs_root1 = log_rlp_hash(exec_result1.as_ref().map(|r| r.logs()).unwrap_or_default());
    let logs_root2 = log_rlp_hash(exec_result2.as_ref().map(|r| r.logs()).unwrap_or_default());

    let state_root1 = state_merkle_trie_root(
        evm.journaled_state
            .database
            .cache
            .trie_account()
            .into_iter(),
    );
    let _state_root2 = state_merkle_trie_root(
        evm2.0
            .journaled_state
            .database
            .cache
            .trie_account()
            .into_iter()
            .filter(|(addr, _)| !genesis_addresses.contains(addr)),
    );

    let print_json_output = |error: Option<String>| {
        if print_json_outcome {
            let json = json!({
                    "stateRoot": state_root1,
                    "logsRoot": logs_root1,
                    "output": exec_result1.as_ref().ok().and_then(|r| r.output().cloned()).unwrap_or_default(),
                    "gasUsed": exec_result1.as_ref().ok().map(|r| r.gas_used()).unwrap_or_default(),
                    "pass": error.is_none(),
                    "errorMsg": error.unwrap_or_default(),
                    "evmResult": exec_result1.as_ref().err().map(|e| e.to_string()).unwrap_or("Ok".to_string()),
                    "postLogsHash": logs_root1,
                    "fork": evm.ctx.cfg.spec,
                    "test": test_name,
                    "d": test.indexes.data,
                    "g": test.indexes.gas,
                    "v": test.indexes.value,
            });
            eprintln!("{json}");
        }
    };

    // If we expect exception revm should return error from execution.
    // So we do not check logs and state root.
    //
    // Note that some tests that have exception and run tests from before state clear
    // would touch the caller account and make it appear in state root calculation.
    // This is not something that we would expect as invalid tx should not touch state.
    // but as this is a cleanup of invalid tx it is not properly defined and in the end
    // it does not matter.
    // Test where this happens: `tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate.json`
    // and you can check that we have only two "hash" values for before and after state clear.
    match (&test.expect_exception, exec_result1) {
        // do nothing
        (None, Ok(result)) => {
            // check output
            if let Some((expected_output, output)) = expected_output.zip(result.output()) {
                if expected_output != output {
                    let kind = TestErrorKind::UnexpectedOutput {
                        expected_output: Some(expected_output.clone()),
                        got_output: result.output().cloned(),
                    };
                    print_json_output(Some(kind.to_string()));
                    return Err(TestError {
                        name: test_name.to_string(),
                        kind,
                    });
                }
            }
        }
        // return okay, exception is expected.
        (Some(_), Err(_)) => return Ok(()),
        _ => {
            let kind = TestErrorKind::UnexpectedException {
                expected_exception: test.expect_exception.clone(),
                got_exception: exec_result1.clone().err().map(|e| e.to_string()),
            };
            print_json_output(Some(kind.to_string()));
            return Err(TestError {
                name: test_name.to_string(),
                kind,
            });
        }
    }

    if logs_root1 != test.logs {
        let kind = TestErrorKind::LogsRootMismatch {
            got: logs_root1,
            expected: test.logs,
        };
        print_json_output(Some(kind.to_string()));
        return Err(TestError {
            name: test_name.to_string(),
            kind,
        });
    }

    if state_root1 != test.hash {
        let kind = TestErrorKind::StateRootMismatch {
            expected: test.hash,
            got: state_root1,
        };
        print_json_output(Some(kind.to_string()));
        return Err(TestError {
            name: test_name.to_string(),
            kind,
        });
    }

    // print_json_output(None);
    // return Ok(());

    let mut error_list: Vec<String> = vec![];

    macro_rules! error_eq {
        ($left:expr, $right:expr, $msg:literal $(,)?) => {
            if $left != $right {
                error_list.push(format!("{}: {} <> {}", $msg, $left, $right));
            }
        };
        ($left:expr, $right:expr, $msg:literal, $($arg:tt)+) => {
            if $left != $right {
                error_list.push(format!("{}: {} <> {}", format!($msg, $($arg)+), $left, $right));
            }
        };
    }

    if logs_root1 != logs_root2 {
        let logs1 = exec_result1.as_ref().map(|r| r.logs()).unwrap_or_default();
        println!("ORIGINAL logs ({}):", logs1.len());
        for log in logs1 {
            println!(
                " - {}: {}",
                hex::encode(log.address),
                log.topics()
                    .get(0)
                    .map(|v| hex::encode(&v))
                    .unwrap_or_default()
            )
        }
        let logs2 = exec_result2.as_ref().map(|r| r.logs()).unwrap_or_default();
        println!("FLUENT logs ({}):", logs2.len());
        for log in logs2 {
            println!(
                " - {}: {}",
                hex::encode(log.address),
                log.topics()
                    .get(0)
                    .map(|v| hex::encode(&v))
                    .unwrap_or_default()
            )
        }
        error_eq!(logs_root1, logs_root2, "EVM <> FLUENT logs root mismatch");
    }

    let exec_result1_res = exec_result1.as_ref().unwrap();
    let exec_result2_res = exec_result2.as_ref().unwrap();
    error_eq!(
        exec_result1_res.gas_used(),
        exec_result2_res.gas_used(),
        "EVM <> FLUENT gas used mismatch"
    );

    // compare contracts
    // for (k, v) in evm.journaled_state.database.cache.contracts.iter() {
    //     let v2 = evm2
    //         .0
    //         .journaled_state
    //         .database
    //         .cache
    //         .contracts
    //         .get(k)
    //         .expect("missing fluent contract");
    //     // we compare only evm bytecode
    //     error_eq!(v.bytecode(), v2.bytecode(), "EVM bytecode mismatch");
    // }
    let mut account_keys = evm
        .journaled_state
        .database
        .cache
        .accounts
        .keys()
        .collect::<Vec<_>>();
    account_keys.sort();
    for address in account_keys {
        let v1 = evm
            .journaled_state
            .database
            .cache
            .accounts
            .get(address)
            .unwrap();
        if cfg!(feature = "debug-print") {
            println!("comparing account (0x{})...", hex::encode(address));
        }
        let v2 = evm2.0.journaled_state.database.cache.accounts.get(address);
        if let Some(a1) = v1.account.as_ref().map(|v| &v.info) {
            let a2 = v2
                .expect("missing FLUENT account")
                .account
                .as_ref()
                .map(|v| &v.info)
                .expect("missing FLUENT account");
            if cfg!(feature = "debug-print") {
                println!(" - status: {:?}", v1.status);
            }
            // error_eq!(
            //     format!("{:?}", v1.status),
            //     format!("{:?}", v2.unwrap().status),
            //     "EVM account status mismatch"
            // );
            if cfg!(feature = "debug-print") {
                println!(" - balance: {}", a1.balance);
            }
            error_eq!(
                a1.balance,
                a2.balance,
                "EVM <> FLUENT account ({}) balance mismatch",
                address,
            );
            if cfg!(feature = "debug-print") {
                println!(" - nonce: {}", a1.nonce);
            }
            error_eq!(a1.nonce, a2.nonce, "EVM <> FLUENT account nonce mismatch");
            if cfg!(feature = "debug-print") {
                println!(" - code_hash: {}", hex::encode(a1.code_hash));
            }
            // assert_eq!(
            //     a1.code_hash, a2.code_hash,
            //     "EVM <> FLUENT account code_hash mismatch",
            // );
            // assert_eq!(
            //     a1.code.as_ref().map(|b| b.original_bytes()),
            //     a2.code.as_ref().map(|b| b.original_bytes()),
            //     "EVM <> FLUENT account code mismatch",
            // );
            if cfg!(feature = "debug-print") {
                println!(" - storage:");
            }
            if let Some(s1) = v1.account.as_ref().map(|v| &v.storage) {
                let mut sorted_keys = s1.keys().collect::<Vec<_>>();
                sorted_keys.sort();
                for slot in sorted_keys {
                    let value1 = s1.get(slot).unwrap();
                    if cfg!(feature = "debug-print") {
                        println!(
                            " - + slot ({}) => ({})",
                            hex::encode(&slot.to_be_bytes::<32>()),
                            hex::encode(&value1.to_be_bytes::<32>())
                        );
                    }
                    // let storage_key = calc_storage_key(address, slot.as_le_bytes().as_ptr());
                    // let fluent_evm_storage = evm2
                    //     .context
                    //     .evm
                    //     .db
                    //     .cache
                    //     .accounts
                    //     .get(&EVM_STORAGE_ADDRESS)
                    //     .expect("missing special EVM storage account");
                    // let value2 = fluent_evm_storage
                    //     .storage_slot(U256::from_le_bytes(storage_key))
                    //     .unwrap_or_else(|| panic!("missing storage key {}",
                    // hex::encode(storage_key)));
                    let value2 = v2
                        .expect("missing FLUENT account (cache)")
                        .account
                        .as_ref()
                        .map(|v| &v.storage);
                    let value2 = value2
                        .expect("missing FLUENT account (storage)")
                        .get(slot)
                        .unwrap_or_else(|| {
                            error_list.push(format!(
                                "missing storage key {}",
                                hex::encode(slot.to_be_bytes::<32>())
                            ));
                            &U256::ZERO
                        });
                    error_eq!(
                        *value1,
                        *value2,
                        "EVM <> FLUENT storage value ({}) mismatch",
                        hex::encode(&slot.to_be_bytes::<32>()),
                    );
                }
            }
        }
    }

    for (address, v1) in evm.journaled_state.database.cache.accounts.iter() {
        if cfg!(feature = "debug-print") {
            println!("comparing balances (0x{})...", hex::encode(address));
        }
        let v2 = evm2.0.journaled_state.database.cache.accounts.get(address);
        if let Some(a1) = v1.account.as_ref().map(|v| &v.info) {
            let a2 = v2
                .expect("missing FLUENT account")
                .account
                .as_ref()
                .map(|v| &v.info)
                .expect("missing FLUENT account");
            if cfg!(feature = "debug-print") {
                println!(" - balance1: {}", a1.balance);
                println!(" - balance2: {}", a2.balance);
            }
            let balance_diff = if a1.balance > a2.balance {
                a1.balance - a2.balance
            } else {
                a2.balance - a1.balance
            };
            if balance_diff != U256::from(0) {
                error_eq!(
                    a1.balance,
                    a2.balance,
                    "EVM <> FLUENT account balance mismatch"
                );
            }
        }
    }

    if error_list.len() > 0 {
        assert!(
            false,
            "----------------------\n{}\n----------------------\n",
            error_list.join("\n")
        );
    }

    print_json_output(None);

    // if state_root1 != state_root2 {
    //     let kind = TestErrorKind::StateRootMismatch2 {
    //         expected: state_root1,
    //         got: state_root2,
    //     };
    //     print_json_output(Some(kind.to_string()));
    //     return Err(TestError {
    //         name: test_name.to_string(),
    //         kind,
    //     });
    // }

    Ok(())
}

pub fn execute_test_suite(
    path: &Path,
    elapsed: &Arc<Mutex<Duration>>,
    trace: bool,
    print_json_outcome: bool,
) -> Result<(), TestError> {
    if skip_test(path) {
        return Ok(());
    }

    println!("Running test: {:?}", path);

    let s = std::fs::read_to_string(path).unwrap();
    let suite: TestSuite = serde_json::from_str(&s).map_err(|e| TestError {
        name: path.to_string_lossy().into_owned(),
        kind: e.into(),
    })?;

    let selected_test_cases = vec![];
    for (name, unit) in suite.0 {
        if selected_test_cases.len() > 0 && !selected_test_cases.contains(&name.as_str()) {
            continue;
        }
        println!("test case: {}", &name);
        // Create database and insert cache
        let mut cache_state = CacheState::new(false);
        let mut cache_state2 = CacheState::new(false);

        println!("\nloading genesis accounts:");
        let start = Instant::now();
        let mut genesis_addresses: HashSet<Address> = Default::default();
        for (address, genesis_account) in GENESIS_CONTRACTS_BY_ADDRESS.iter() {
            let start = Instant::now();
            print!("- loading genesis account ({address})... ");
            let bytecode = Bytecode::new_raw(genesis_account.rwasm_bytecode.clone());
            let acc_info = AccountInfo {
                balance: U256::ZERO,
                nonce: 0,
                code_hash: genesis_account.rwasm_bytecode_hash,
                code: Some(bytecode),
            };
            println!(
                "loaded in ({:?}): address={}, nonce={}, balance={} code_hash={}",
                start.elapsed(),
                address,
                acc_info.nonce,
                acc_info.balance,
                acc_info.code_hash
            );
            cache_state2.insert_account(*address, acc_info);
            genesis_addresses.insert(*address);
        }
        println!("loaded genesis accounts in: {:?}", start.elapsed());

        println!("\nloading EVM accounts:");
        let start = Instant::now();
        for (address, info) in &unit.pre {
            let acc_info = AccountInfo {
                balance: info.balance,
                code_hash: keccak256(&info.code),
                nonce: info.nonce,
                code: Some(Bytecode::new_raw(info.code.clone())),
                ..Default::default()
            };
            cache_state.insert_account_with_storage(*address, acc_info, info.storage.clone());
        }
        for (address, info) in unit.pre {
            let mut acc_info = cache_state2
                .accounts
                .get(&address)
                .and_then(|a| a.account.clone())
                .map(|a| a.info)
                .unwrap_or_else(AccountInfo::default);
            if !acc_info.balance.is_zero() && !info.balance.is_zero() {
                assert_eq!(
                    acc_info.balance, info.balance,
                    "genesis account balance mismatch, this test won't work"
                );
            }
            acc_info.balance = info.balance;
            acc_info.nonce = info.nonce;
            let prev_code_len = acc_info.code.as_ref().map(|v| v.len()).unwrap_or_default();
            if prev_code_len > 0 && info.code.len() > 0 {
                println!(
                    "WARN: code length collision for an account ({address}), this test might not work"
                );
            }
            let evm_code_hash = keccak256(&info.code);
            println!(
                " - address={address}, evm_code_hash={evm_code_hash}, evm_code_hash_u256={}, code_len={}",
                Into::<U256>::into(evm_code_hash), info.code.len(),
            );
            // write EVM code hash state
            if info.code.len() > 0 {
                // set account info bytecode to the proxy loader
                let mut metadata = vec![];
                metadata.extend_from_slice(evm_code_hash.as_slice());
                metadata.extend_from_slice(info.code.as_ref());
                let bytecode = Bytecode::OwnableAccount(OwnableAccountBytecode::new(
                    PRECOMPILE_EVM_RUNTIME,
                    metadata.into(),
                ));
                acc_info.code_hash = bytecode.hash_slow();
                acc_info.code = Some(bytecode);
            }
            // write evm account into state
            cache_state2.insert_account_with_storage(address, acc_info, info.storage);
        }
        println!("loaded evm accounts in: {:?}", start.elapsed());

        let mut cfg_env = CfgEnv::default();
        let mut block_env = BlockEnv::default();
        let mut tx_env = TxEnv::default();

        // for mainnet
        cfg_env.chain_id = 1;
        // cfg_env.blob_target_and_max_count;
        // env.cfg.spec_id is set down the road

        // block env
        block_env.number = unit.env.current_number.to();
        block_env.beneficiary = unit.env.current_coinbase;
        block_env.timestamp = unit.env.current_timestamp.to();
        block_env.gas_limit = unit.env.current_gas_limit.to();
        block_env.basefee = unit.env.current_base_fee.unwrap_or_default().to();
        block_env.difficulty = unit.env.current_difficulty.to();
        // after the Merge prevrandao replaces mix_hash field in block and replaced difficulty
        // opcode in EVM.
        block_env.prevrandao = unit.env.current_random;
        // EIP-4844
        if let Some(current_excess_blob_gas) = unit.env.current_excess_blob_gas {
            block_env.set_blob_excess_gas_and_price(
                current_excess_blob_gas.to(),
                BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE,
            );
        } else if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
            unit.env.parent_blob_gas_used,
            unit.env.parent_excess_blob_gas,
        ) {
            block_env.set_blob_excess_gas_and_price(
                calc_excess_blob_gas(parent_blob_gas_used.to(), parent_excess_blob_gas.to(), 0),
                BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE,
            );
        }

        // tx env
        tx_env.caller = if let Some(address) = unit.transaction.sender {
            address
        } else {
            recover_address(unit.transaction.secret_key.as_slice()).ok_or_else(|| TestError {
                name: name.clone(),
                kind: TestErrorKind::UnknownPrivateKey(unit.transaction.secret_key),
            })?
        };
        tx_env.gas_price = unit
            .transaction
            .gas_price
            .or(unit.transaction.max_fee_per_gas)
            .unwrap_or_default()
            .to();
        tx_env.gas_priority_fee = unit.transaction.max_priority_fee_per_gas.map(|v| v.to());
        // EIP-4844
        tx_env.blob_hashes = unit.transaction.blob_versioned_hashes;
        tx_env.max_fee_per_blob_gas = unit
            .transaction
            .max_fee_per_blob_gas
            .map(|v| v.to())
            .unwrap_or_default();

        // post and execution
        for (spec_name, tests) in unit.post {
            if matches!(
                spec_name,
                SpecName::ByzantiumToConstantinopleAt5
                    | SpecName::Constantinople
                    | SpecName::Unknown
            ) {
                continue;
            }
            if spec_name.lt(&SpecName::Prague) {
                continue;
            }

            let spec_id = spec_name.to_spec_id();
            cfg_env.spec = spec_id;

            for (index, test) in tests.into_iter().enumerate() {
                println!(
                    "\n\n\n\n\nRunning test with txdata: ({}) {}",
                    index,
                    hex::encode(test.txbytes.clone().unwrap_or_default().as_ref())
                );
                tx_env.gas_limit = unit.transaction.gas_limit[test.indexes.gas].saturating_to();

                tx_env.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                tx_env.value = unit.transaction.value[test.indexes.value];

                let access_list: Vec<AccessListItem> = unit
                    .transaction
                    .access_lists
                    .get(test.indexes.data)
                    .and_then(Option::as_deref)
                    .unwrap_or_default()
                    .iter()
                    .map(|item| AccessListItem {
                        address: item.address,
                        storage_keys: item.storage_keys.clone(),
                    })
                    .collect();
                tx_env.access_list = access_list.into();

                tx_env.kind = match unit.transaction.to {
                    Some(add) => TransactTo::Call(add),
                    None => TransactTo::Create,
                };

                tx_env.tx_type = Eip1559 as u8;
                tx_env.nonce = unit.transaction.nonce.to();

                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));
                let mut cache2 = cache_state2.clone();
                cache2.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));

                let state: State<InMemoryDB> = StateBuilder::default()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();

                let cfg_env2 = cfg_env.clone();
                let mut evm = MainnetContext::new(state, spec_id)
                    .with_cfg(cfg_env2)
                    .with_block(block_env.clone())
                    .build_mainnet();

                let state2: State<InMemoryDB> = StateBuilder::default()
                    .with_cached_prestate(cache2)
                    .with_bundle_update()
                    .build();
                let mut evm2 = RwasmContext::new(state2, spec_id)
                    .with_cfg(cfg_env.clone())
                    .with_block(block_env.clone())
                    .build_rwasm();

                // do the deed
                // if trace {
                //     evm = evm.with_inspector(TracerEip3155::new(Box::new(stderr())).
                // without_summary());     evm2 =
                // evm2.with_inspector(TracerEip3155::new(Box::new(stderr())).without_summary());
                // }
                let timer = Instant::now();
                print!("\n\nrunning original EVM tests... ");
                let start = Instant::now();
                let result_native = evm.transact_commit(tx_env.clone());
                println!("{:?}", start.elapsed());
                let start = Instant::now();
                print!("\n\nrunning RWASM tests... ");
                let result_fluent = evm2.transact_commit(tx_env.clone());
                println!("{:?}", start.elapsed());
                *elapsed.lock().unwrap() += timer.elapsed();

                // dump state and traces if the test failed
                let start = Instant::now();
                print!("\n\ncomparing EVM<>RWASM state... ");
                let output = check_evm_execution(
                    &test,
                    &spec_name,
                    unit.out.as_ref(),
                    &name,
                    &result_native,
                    &result_fluent,
                    &mut evm,
                    &mut evm2,
                    print_json_outcome,
                    &genesis_addresses,
                );
                println!("{:?}", start.elapsed());
                let Err(e) = output else {
                    continue;
                };

                // if we are already in trace mode, return error
                static FAILED: AtomicBool = AtomicBool::new(false);
                if trace || FAILED.swap(true, Ordering::SeqCst) {
                    return Err(e);
                }

                // re-build to run with tracing
                // let mut cache = cache_state.clone();
                // cache.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));
                // let mut cache2 = cache_state2.clone();
                // cache2.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));
                // let state = State::builder()
                //     .with_cached_prestate(cache)
                //     .with_bundle_update()
                //     .build();
                // let state2 = State::builder()
                //     .with_cached_prestate(cache2)
                //     .with_bundle_update()
                //     .build();

                let path = path.display();
                // println!("\nTraces:");
                // let mut evm = Evm::builder()
                //     .with_spec_id(spec_id)
                //     .with_db(state)
                //     .with_env(env.clone())
                //     .with_external_context(TracerEip3155::new(Box::new(stdout())).
                // without_summary())
                //     .append_handler_register(inspector_handle_register)
                //     .build();
                // let mut evm2 = Rwasm::builder()
                //     .with_spec_id(spec_id)
                //     .with_db(state2)
                //     .with_external_context(TracerEip3155::new(Box::new(stdout())))
                //     .append_handler_register(inspector_handle_register)
                //     .build();
                // let _ = evm.transact_commit();
                // let _ = evm2.transact_commit();

                println!("\nExecution result: {result_native:#?}");
                println!("\nExpected exception: {:?}", test.expect_exception);
                println!("\nState before: {cache_state:#?}");
                println!("\nState after: {:#?}", evm.journaled_state.database.cache);
                println!("\nSpecification: {spec_id:?}");
                // println!("\nEnvironment: {env:#?}");
                println!("\nTest name: {name:?} (index: {index}, path: {path}) failed:\n{e}");

                return Err(e);
            }
        }

        println!("FINISHED!!!!!!!!!!!\n\n")
    }
    Ok(())
}

pub fn run(
    test_files: Vec<PathBuf>,
    mut single_thread: bool,
    trace: bool,
    mut print_outcome: bool,
    keep_going: bool,
) -> Result<(), TestError> {
    // trace implies print_outcome
    if trace {
        print_outcome = true;
    }
    // print_outcome or trace implies single_thread
    if print_outcome {
        single_thread = true;
    }
    let n_files = test_files.len();

    let n_errors = Arc::new(AtomicUsize::new(0));
    let console_bar = Arc::new(ProgressBar::with_draw_target(
        Some(n_files as u64),
        ProgressDrawTarget::stdout(),
    ));
    let queue = Arc::new(Mutex::new((0usize, test_files)));
    let elapsed = Arc::new(Mutex::new(std::time::Duration::ZERO));

    let num_threads = match (single_thread, std::thread::available_parallelism()) {
        (true, _) | (false, Err(_)) => 1,
        (false, Ok(n)) => n.get(),
    };
    let num_threads = num_threads.min(n_files);
    let mut handles = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let queue = queue.clone();
        let n_errors = n_errors.clone();
        let console_bar = console_bar.clone();
        let elapsed = elapsed.clone();

        let thread = std::thread::Builder::new().name(format!("runner-{i}"));

        let f = move || loop {
            if !keep_going && n_errors.load(Ordering::SeqCst) > 0 {
                return Ok(());
            }

            let (_index, test_path) = {
                let (current_idx, queue) = &mut *queue.lock().unwrap();
                let prev_idx = *current_idx;
                let Some(test_path) = queue.get(prev_idx).cloned() else {
                    return Ok(());
                };
                *current_idx = prev_idx + 1;
                (prev_idx, test_path)
            };

            let result = execute_test_suite(&test_path, &elapsed, trace, print_outcome);

            // Increment after the test is done.
            console_bar.inc(1);

            if let Err(err) = result {
                n_errors.fetch_add(1, Ordering::SeqCst);
                if !keep_going {
                    return Err(err);
                }
            }
        };
        handles.push(thread.spawn(f).unwrap());
    }

    // join all threads before returning an error
    let mut thread_errors = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => thread_errors.push(e),
            Err(_) => thread_errors.push(TestError {
                name: format!("thread {i} panicked"),
                kind: TestErrorKind::Panic,
            }),
        }
    }
    console_bar.finish();

    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.lock().unwrap().as_secs_f64()
    );

    let n_errors = n_errors.load(Ordering::SeqCst);
    let n_thread_errors = thread_errors.len();
    if n_errors == 0 && n_thread_errors == 0 {
        println!("All tests passed!");
        Ok(())
    } else {
        println!("Encountered {n_errors} errors out of {n_files} total tests");

        if n_thread_errors == 0 {
            std::process::exit(1);
        }

        if n_thread_errors > 1 {
            println!("{n_thread_errors} threads returned an error, out of {num_threads} total:");
            for error in &thread_errors {
                println!("{error}");
            }
        }
        Err(thread_errors.swap_remove(0))
    }
}
