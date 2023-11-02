use crate::{
    primitives::{
        Account, AccountInfo, Address, Bytecode, HashMap, TxEnv, B256, KECCAK_EMPTY, U256,
    },
    Database, DatabaseCommit, EVM,
};
#[cfg(feature = "runtime")]
use fluentbase_rwasm::rwasm::{Compiler, ImportLinker};
use hex_literal::hex;
use revm_interpreter::primitives::{Env, TransactTo};

fn wat2rwasm(wat: &str) -> Vec<u8> {
    let wasm_binary = wat::parse_str(wat).unwrap();
    let mut compiler = Compiler::new(&wasm_binary).unwrap();
    compiler.finalize().unwrap()
}

fn wasm2rwasm(wasm_binary: &[u8], import_linker: &ImportLinker) -> Vec<u8> {
    Compiler::new_with_linker(&wasm_binary.to_vec(), Some(import_linker))
        .unwrap()
        .finalize()
        .unwrap()
}

#[derive(Default)]
struct TestDb {
    accounts: HashMap<Address, AccountInfo>,
}

impl TestDb {
    pub fn add_account(&mut self, address: Address, account_info: AccountInfo) {
        self.accounts.insert(address, account_info);
    }
}

impl DatabaseCommit for TestDb {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        todo!()
    }
}

impl Database for TestDb {
    type Error = ();

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
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

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        todo!()
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        todo!()
    }
}

#[test]
fn test_simple() {
    let rwasm_binary = wat2rwasm(
        r#"
(module
  (func $main
    global.get 0
    global.get 1
    call $add
    global.get 2
    call $add
    drop
    )
  (func $add (param $lhs i32) (param $rhs i32) (result i32)
    local.get $lhs
    local.get $rhs
    i32.add
    )
  (global (;0;) i32 (i32.const 100))
  (global (;1;) i32 (i32.const 20))
  (global (;2;) i32 (i32.const 3))
  (export "main" (func $main)))
    "#,
    );

    let caller = Address::from(hex!("390a4CEdBb65be7511D9E1a35b115376F39DbDF3"));
    let mut evm = EVM::with_env(Env {
        cfg: Default::default(),
        block: Default::default(),
        tx: TxEnv {
            gas_limit: 1_000_000,
            transact_to: TransactTo::Call(Address::zero()),
            data: Default::default(),
            caller,
            ..Default::default()
        },
    });
    let mut test_db = TestDb::default();
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
        Address::zero(),
        AccountInfo {
            balance: Default::default(),
            nonce: 0,
            code_hash: bytecode.hash_slow(),
            code: Some(bytecode),
        },
    );
    evm.database(test_db);
    let res = evm.transact().unwrap();
    println!("{:?}", res)
}
