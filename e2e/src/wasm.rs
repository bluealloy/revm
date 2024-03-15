use crate::util::{check_success, TestingContext};
use fluentbase_codec::Encoder;
use fluentbase_runtime::{zktrie::ZkTrieStateDb, JournaledTrie};
use fluentbase_sdk::{evm::ContractInput, LowLevelSDK};
use fluentbase_types::{InMemoryAccountDb, U256};
use hex_literal::hex;
use revm::primitives::{address, keccak256, Address, Bytes, Eval, Output};
use revm_precompile::B256;
use std::{cell::RefCell, rc::Rc};

#[test]
fn test_greeting_deploy() {
    let mut ctx = TestingContext::default();

    let contract_wasm_bytes = include_bytes!("../bin/greeting-deploy.wasm");
    let caller_address = address!("000000000000000000000000000000000000000c");

    // let contract_input = ContractInput {
    //     ..Default::default()
    // };
    // let raw_input = contract_input.encode_to_vec(0);

    let db = InMemoryAccountDb::default();
    let storage = ZkTrieStateDb::new_empty(db);
    let journal = JournaledTrie::new(storage);
    // LowLevelSDK::with_test_input(raw_input);
    LowLevelSDK::with_jzkt(Rc::new(RefCell::new(journal)));

    let res = check_success(ctx.deploy_wasm_contract(caller_address, contract_wasm_bytes));
    assert_eq!(res.reason, Eval::Return);
    let contract_address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = check_success(ctx.call_wasm_contract(caller_address, contract_address, &[]));
    let output = res2.output.data().to_vec();
    assert_eq!("Hello, World".as_bytes().to_vec(), output);
}

#[test]
fn test_contract_input_check_recode() {
    let mut ctx = TestingContext::default();

    let contract_wasm_bytes = include_bytes!("../bin/contract_input_check_recode-deploy.wasm");
    let caller_address = address!("000000000000000000000000000000000000000c");
    let block_hash = B256::left_padding_from(&hex!("0123456789abcdef"));
    let contract_value = U256::from_be_slice(&hex!("0123456789abcdef"));
    let contract_is_static = false;
    let block_coinbase: Address = address!("0000000000000000000000000000000000000012");
    let env_chain_id = 23;

    let contract_input_data_str = "i am a contract input";

    let db = InMemoryAccountDb::default();
    let storage = ZkTrieStateDb::new_empty(db);
    let journal = JournaledTrie::new(storage);
    LowLevelSDK::with_jzkt(Rc::new(RefCell::new(journal)));

    let res = check_success(ctx.deploy_wasm_contract(caller_address, contract_wasm_bytes));
    assert_eq!(res.reason, Eval::Return);
    let contract_address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };

    let contract_input = ContractInput {
        contract_input: Bytes::copy_from_slice(contract_input_data_str.as_bytes()),
        contract_input_size: contract_input_data_str.as_bytes().len() as u32,

        env_chain_id,
        contract_address,
        contract_caller: caller_address,
        contract_bytecode: Bytes::copy_from_slice(contract_wasm_bytes),
        contract_code_size: contract_wasm_bytes.len() as u32,
        contract_code_hash: keccak256(contract_wasm_bytes),

        contract_value,
        contract_is_static,
        block_hash,
        block_coinbase,
        tx_gas_priority_fee: Some(U256::from_be_slice(&hex!("12345678"))),
        tx_caller: caller_address,

        ..Default::default()
    };
    let raw_input = contract_input.encode_to_vec(0);
    let res2 = check_success(ctx.call_wasm_contract(caller_address, contract_address, &raw_input));
    assert_eq!(res2.reason, Eval::Return);
    let output = res2.output.data().to_vec();
    assert_eq!(raw_input, output);
}
