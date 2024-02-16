use std::cell::RefCell;
use std::rc::Rc;

use fluentbase_codec::Encoder;
use fluentbase_runtime::JournaledTrie;
use fluentbase_runtime::zktrie::ZkTrieStateDb;
use fluentbase_sdk::evm::ContractInput;
use fluentbase_sdk::LowLevelSDK;
use fluentbase_types::InMemoryAccountDb;

use revm::primitives::{address, Address, BlockEnv, Bytecode, Bytes, CfgEnv, CreateScheme, Env, Eval, ExecutionResult, Output, ResultAndState, TransactTo, TxEnv};

use crate::util::{check_success, TestingContext};

impl TestingContext {
    pub(crate) fn deploy_wasm_contract(
        &mut self,
        caller: Address,
        input_binary: &[u8],
    ) -> ResultAndState {
        let mut evm = revm_rwasm::RWASM::with_env(Env {
            cfg: CfgEnv::default(),
            block: BlockEnv::default(),
            tx: TxEnv {
                gas_limit: 10_000_000,
                transact_to: TransactTo::Create(CreateScheme::Create),
                data: Bytes::copy_from_slice(input_binary),
                caller,
                ..Default::default()
            },
        });
        let res = evm.transact().unwrap();
        match &res.result {
            ExecutionResult::Success { output, .. } => match output {
                Output::Create(bytecode, address) => {
                    let bytecode = Bytecode::new_raw(bytecode.clone());
                    let mut account_info = self.get_account_mut(address.unwrap());
                    account_info.code = Some(bytecode.clone());
                    account_info.code_hash = bytecode.hash_slow();
                }
                _ => {}
            },
            _ => {}
        }
        res
    }

    pub(crate) fn call_wasm_contract(
        &self,
        caller: Address,
        to: Address,
        input: &[u8],
    ) -> ResultAndState {
        let mut evm = revm_rwasm::RWASM::with_env(Env {
            cfg: Default::default(),
            block: Default::default(),
            tx: TxEnv {
                gas_limit: 10_000_000,
                transact_to: TransactTo::Call(to),
                data: Bytes::copy_from_slice(input),
                caller,
                ..Default::default()
            },
        });
        evm.transact().unwrap()
    }
}

#[test]
fn test_greeting() {
    let mut ctx = TestingContext::default();

    let contract_wasm_bytes = include_bytes!("../bin/greeting-deploy.wasm");
    let caller_address = address!("000000000000000000000000000000000000000c");

    let contract_input = ContractInput {
        .. Default::default()
    };
    let raw_input = contract_input.encode_to_vec(0);

    let db = InMemoryAccountDb::default();
    let storage = ZkTrieStateDb::new_empty(db);
    let journal = JournaledTrie::new(storage);
    LowLevelSDK::with_test_input(raw_input);
    LowLevelSDK::with_jzkt(Rc::new(RefCell::new(journal)));

    let res = check_success(ctx.deploy_wasm_contract(
        caller_address,
        contract_wasm_bytes,
    ));
    assert_eq!(res.reason, Eval::Return);
    let contract_address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_wasm_contract(
        caller_address,
        contract_address,
        &[],
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!("Hello, World".as_bytes().to_vec(), output);
}
