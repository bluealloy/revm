use fluentbase_runtime::Runtime;
use fluentbase_rwasm::rwasm::{Compiler, CompilerConfig, FuncOrExport};
use hashbrown::HashMap;
use hex_literal::hex;
use revm::primitives::{
    address,
    db::{Database, DatabaseCommit},
    Account, AccountInfo, Address, Bytecode, Env, Eval, ExecutionResult, Log, Output,
    ResultAndState, TransactTo, TxEnv, B256, U256,
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

    fn deploy_contract(&mut self, caller: Address, input_binary: &[u8]) -> ResultAndState {
        let mut evm = EVM::with_env(Env {
            cfg: Default::default(),
            block: Default::default(),
            tx: TxEnv {
                gas_limit: 1_000_000,
                transact_to: TransactTo::Create(CreateScheme::Create),
                data: Bytes::copy_from_slice(input_binary),
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

#[test]
fn test_evm() {
    let hello_world_bytecode = hex!("608060405234801561000f575f80fd5b506101688061001d5f395ff3fe608060405234801561000f575f80fd5b5060043610610029575f3560e01c8063dffeadd01461002d575b5f80fd5b61003561004b565b6040516100429190610112565b60405180910390f35b60606040518060400160405280600c81526020017f48656c6c6f2c20576f726c640000000000000000000000000000000000000000815250905090565b5f81519050919050565b5f82825260208201905092915050565b5f5b838110156100bf5780820151818401526020810190506100a4565b5f8484015250505050565b5f601f19601f8301169050919050565b5f6100e482610088565b6100ee8185610092565b93506100fe8185602086016100a2565b610107816100ca565b840191505092915050565b5f6020820190508181035f83015261012a81846100da565b90509291505056fea2646970667358221220e37f1ddf5cf89f81a254d4ff46c19e6000be7de71326bd8a5106eeca92e3be6164736f6c63430008160033");
    let mut ctx = TestingContext::default();
    let res = check_success(ctx.deploy_contract(
        address!("0000000000000000000000000000000000000000"),
        &hello_world_bytecode,
    ));
    assert_eq!(res.reason, Eval::Return);
    let address = match res.output {
        Output::Create(_, address) => address.unwrap(),
        Output::Call(_) => panic!("not deployed"),
    };
    let res2 = ctx.call_contract(
        address!("0000000000000000000000000000000000000000"),
        address,
        &hex!("dffeadd0"),
    );
    assert_eq!(res.reason, Eval::Return);
    let output = res2.result.output().unwrap().to_vec();
    assert_eq!(&output[64..76], "Hello, World".as_bytes());
}
