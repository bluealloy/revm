use crate::util::{check_success, wat2wasm, TestingContext};
use fluentbase_sdk::LowLevelSDK;
use revm::primitives::{
    address,
    Address,
    Bytecode,
    Bytes,
    CreateScheme,
    Env,
    Eval,
    ExecutionResult,
    Output,
    ResultAndState,
    TransactTo,
    TxEnv,
};

impl TestingContext {
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
                gas_limit: 1_000_000,
                transact_to: TransactTo::Call(to),
                data: Bytes::copy_from_slice(input),
                caller,
                ..Default::default()
            },
        });
        evm.transact().unwrap()
    }

    pub(crate) fn deploy_wasm_contract(
        &mut self,
        caller: Address,
        input_binary: &[u8],
    ) -> ResultAndState {
        let mut evm = revm_rwasm::RWASM::with_env(Env {
            cfg: Default::default(),
            block: Default::default(),
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
                    self.get_account_mut(address.unwrap()).code = Some(bytecode.clone());
                    self.get_account_mut(address.unwrap()).code_hash = bytecode.hash_slow();
                }
                _ => {}
            },
            _ => {}
        }
        res
    }
}

#[test]
fn test_greeting() {
    let mut ctx = TestingContext::default();
    LowLevelSDK::with_test_input();
    let res = check_success(ctx.deploy_wasm_contract(
        address!("0000000000000000000000000000000000000000"),
        &wat2wasm(include_str!("../bin/greeting-deploy.wat")),
    ));
    assert_eq!(res.reason, Eval::Return);
    let address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_wasm_contract(
        address!("0000000000000000000000000000000000000000"),
        address,
        &[],
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!(output, "Hello, World".as_bytes().to_vec());
}
