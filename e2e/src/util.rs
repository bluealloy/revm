use hashbrown::HashMap;
use revm::primitives::{
    Account,
    AccountInfo,
    Address,
    Bytecode,
    Bytes,
    CreateScheme,
    Env,
    Eval,
    ExecutionResult,
    Log,
    Output,
    ResultAndState,
    TransactTo,
    TxEnv,
    B256,
    U256,
};

pub(crate) fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
}

#[derive(Default, Clone)]
pub(crate) struct TestingContext {
    pub(crate) accounts: HashMap<Address, AccountInfo>,
    pub(crate) sender: Option<Address>,
}

impl TestingContext {
    pub(crate) fn add_account(&mut self, address: Address, account_info: AccountInfo) {
        self.accounts.insert(address, account_info);
    }

    pub(crate) fn get_account_mut(&mut self, address: Address) -> &mut AccountInfo {
        if !self.accounts.contains_key(&address) {
            self.accounts.insert(address, AccountInfo::default());
        }
        self.accounts.get_mut(&address).unwrap()
    }

    pub(crate) fn call_contract(
        &self,
        caller: Address,
        to: Address,
        input: &[u8],
    ) -> ResultAndState {
        let mut evm = revm::EVM::with_env(Env {
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

    pub(crate) fn deploy_contract(
        &mut self,
        caller: Address,
        input_binary: &[u8],
    ) -> ResultAndState {
        let mut evm = revm::EVM::with_env(Env {
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

impl revm::DatabaseCommit for TestingContext {
    fn commit(&mut self, _changes: HashMap<Address, Account>) {
        todo!()
    }
}

impl revm::Database for TestingContext {
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

pub(crate) struct SuccessResult {
    pub(crate) reason: Eval,
    pub(crate) gas_used: u64,
    pub(crate) gas_refunded: u64,
    pub(crate) logs: Vec<Log>,
    pub(crate) output: Output,
}

pub(crate) fn check_success(res: ResultAndState) -> SuccessResult {
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
