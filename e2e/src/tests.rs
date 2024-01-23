use fluentbase_runtime::Runtime;
use fluentbase_rwasm::rwasm::{Compiler, CompilerConfig, FuncOrExport};
use hashbrown::HashMap;
use hex_literal::hex;
use revm::primitives::{
    address,
    db::{Database, DatabaseCommit},
    Account, AccountInfo, Address, Bytecode, Env, Eval, ExecutionResult, Log, Output,
    ResultAndState, TransactTo, TxEnv, B256, KECCAK_EMPTY, U256,
};
use revm::EVM;
use revm_interpreter::CreateScheme;
use revm_precompile::Bytes;

fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
}

fn wat2rwasm(wat: &str) -> Vec<u8> {
    let wasm_binary = wat::parse_str(wat).unwrap();
    wasm2rwasm(wasm_binary.as_slice())
}

fn wasm2rwasm(wasm_binary: &[u8]) -> Vec<u8> {
    let import_linker = Runtime::<()>::new_linker();
    let mut compiler =
        Compiler::new_with_linker(wasm_binary, CompilerConfig::default(), Some(&import_linker))
            .unwrap();
    compiler.translate(FuncOrExport::Export("deploy")).unwrap();
    compiler.finalize().unwrap()
}

#[derive(Default, Clone)]
struct TestingContext {
    accounts: HashMap<Address, AccountInfo>,
    sender: Option<Address>,
}

impl TestingContext {
    fn add_account(&mut self, address: Address, account_info: AccountInfo) {
        self.accounts.insert(address, account_info);
    }

    fn get_account_mut(&mut self, address: Address) -> &mut AccountInfo {
        if !self.accounts.contains_key(&address) {
            self.accounts.insert(address, AccountInfo::default());
        }
        self.accounts.get_mut(&address).unwrap()
    }

    fn call_contract(&self, caller: Address, to: Address, input: &[u8]) -> ResultAndState {
        let mut evm = EVM::with_env(Env {
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
        evm.database(self.clone());
        evm.transact().unwrap()
    }

    fn deploy_contract(&mut self, caller: Address, wasm_binary: &[u8]) -> ResultAndState {
        let mut evm = EVM::with_env(Env {
            cfg: Default::default(),
            block: Default::default(),
            tx: TxEnv {
                gas_limit: 1_000_000,
                transact_to: TransactTo::Create(CreateScheme::Create),
                data: Bytes::copy_from_slice(wasm_binary),
                caller,
                ..Default::default()
            },
        });
        evm.database(self.clone());
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

impl DatabaseCommit for TestingContext {
    fn commit(&mut self, _changes: HashMap<Address, Account>) {
        todo!()
    }
}

impl Database for TestingContext {
    type Error = ();

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(acc) = self.accounts.get(&address) {
            return Ok(Some(acc.clone()));
        }
        self.accounts.insert(address, AccountInfo::default());
        Ok(Some(self.accounts.get(&address).cloned().unwrap()))
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        for account in self.accounts.values() {
            if account.code_hash == code_hash {
                return Ok(account.code.clone().unwrap());
            }
        }
        panic!("not possible now :(")
    }

    fn storage(&mut self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        todo!()
    }

    fn block_hash(&mut self, _number: U256) -> Result<B256, Self::Error> {
        todo!()
    }
}

fn transact_wat(wat: &str) -> ResultAndState {
    let rwasm_binary = wat2wasm(wat);
    let caller = Address::from(hex!("390a4CEdBb65be7511D9E1a35b115376F39DbDF3"));
    let mut evm = EVM::with_env(Env {
        cfg: Default::default(),
        block: Default::default(),
        tx: TxEnv {
            gas_limit: 1_000_000,
            transact_to: TransactTo::Create(CreateScheme::Create),
            data: Bytes::copy_from_slice(&rwasm_binary),
            caller,
            ..Default::default()
        },
    });
    let mut test_db = TestingContext::default();
    test_db.add_account(
        caller,
        AccountInfo {
            balance: Default::default(),
            nonce: 0,
            code_hash: KECCAK_EMPTY,
            code: None,
        },
    );
    let bytecode = Bytecode::new_raw(rwasm_binary.into());
    test_db.add_account(
        Address::ZERO,
        AccountInfo {
            balance: Default::default(),
            nonce: 0,
            code_hash: bytecode.hash_slow(),
            code: Some(bytecode),
        },
    );
    evm.database(test_db);
    let res = evm.transact().unwrap();
    println!("{:?}", res);
    res
}

struct SuccessResult {
    reason: Eval,
    gas_used: u64,
    gas_refunded: u64,
    logs: Vec<Log>,
    output: Output,
}

fn check_success(res: ResultAndState) -> SuccessResult {
    match res.result {
        ExecutionResult::Success {
            reason,
            gas_used,
            gas_refunded,
            logs,
            output,
        } => SuccessResult {
            reason,
            gas_used,
            gas_refunded,
            logs,
            output,
        },
        ExecutionResult::Revert { output, .. } => {
            panic!("reverted: {}", hex::encode(output))
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!("halted: {:?}", reason)
        }
    }
}

#[test]
fn test_greeting() {
    let mut ctx = TestingContext::default();
    let res = check_success(ctx.deploy_contract(
        address!("0000000000000000000000000000000000000000"),
        &wat2wasm(include_str!("../bin/greeting-deploy.wat")),
    ));
    assert_eq!(res.reason, Eval::Return);
    let address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_contract(
        address!("0000000000000000000000000000000000000000"),
        address,
        &[],
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!(output, "Hello, World".as_bytes().to_vec());
}
