use core::convert::TryInto;

use bn_rs::BN;
use bytes::Bytes;
use primitive_types::{H160, U256};
use revm::{AccountInfo, Bytecode, DatabaseCommit, InMemoryDB, SpecId, TransactTo, EVM as rEVM};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Wrapper around revm with InMemoryDB
#[wasm_bindgen]
pub struct EVM {
    revm: rEVM<InMemoryDB>,
}

impl Default for EVM {
    fn default() -> Self {
        EVM::new()
    }
}

impl EVM {
    pub fn new() -> EVM {
        console_log!("EVM created");
        let mut evm = EVM { revm: rEVM::new() };
        evm.revm.database(InMemoryDB::default());
        evm
    }

    pub fn transact(&mut self) -> u64 {
        let (exit, data, gas, state, logs) = self.revm.transact();
        console_log!(
            "Transact done, exit:{:?}, gas:{:?}, data:{:?}\nstate_chage:{:?}\nlogs:{:?}",
            exit,
            gas,
            data,
            state,
            logs,
        );
        self.revm.db().unwrap().commit(state);
        gas
    }

    /****** DATABASE RELATED ********/
    pub fn insert_account(&mut self, address: BN, nonce: u64, balance: BN, code: &[u8]) {
        let address = address.try_into().unwrap();
        let acc_info = AccountInfo::new(
            balance.try_into().unwrap(),
            nonce,
            Bytecode::new_raw(Bytes::copy_from_slice(code)),
        );
        console_log!("Added account:{:?} info:{:?}", address, acc_info);
        self.revm
            .db()
            .unwrap()
            .insert_account_info(address, acc_info);
    }

    /****** ALL CFG ENV SETTERS ********/

    pub fn cfg_chain_id(&mut self, gas_limit: BN) {
        self.revm.env.cfg.chain_id = gas_limit.try_into().unwrap();
    }
    pub fn cfg_spec_id(&mut self, spec_id: u8) {
        self.revm.env.cfg.spec_id = SpecId::try_from_u8(spec_id).unwrap_or(SpecId::LATEST);
    }

    /****** ALL BLOCK ENV SETTERS ********/

    pub fn block_gas_limit(&mut self, gas_limit: BN) {
        self.revm.env.block.gas_limit = gas_limit.try_into().unwrap();
    }
    pub fn block_number(&mut self, number: BN) {
        self.revm.env.block.number = number.try_into().unwrap();
    }
    pub fn block_coinbase(&mut self, coinbase: BN) {
        self.revm.env.block.coinbase = coinbase.try_into().unwrap();
    }
    pub fn block_timestamp(&mut self, timestamp: BN) {
        self.revm.env.block.timestamp = timestamp.try_into().unwrap();
    }
    pub fn block_difficulty(&mut self, difficulty: BN) {
        self.revm.env.block.difficulty = difficulty.try_into().unwrap();
    }
    pub fn block_basefee(&mut self, basefee: BN) {
        self.revm.env.block.basefee = basefee.try_into().unwrap();
    }

    /****** ALL TX ENV SETTERS ********/

    pub fn tx_caller(&mut self, tx_caller: BN) {
        self.revm.env.tx.caller = tx_caller.try_into().unwrap();
    }
    pub fn tx_gas_limit(&mut self, gas_limit: u64) {
        self.revm.env.tx.gas_limit = gas_limit;
    }
    pub fn tx_gas_price(&mut self, gas_price: BN) {
        self.revm.env.tx.gas_price = gas_price.try_into().unwrap();
    }
    pub fn tx_gas_priority_fee(&mut self, gas_priority_fee: Option<BN>) {
        self.revm.env.tx.gas_priority_fee = gas_priority_fee.map(|v| v.try_into().unwrap());
    }
    pub fn tx_value(&mut self, value: BN) {
        self.revm.env.tx.value = value.try_into().unwrap();
    }
    pub fn tx_chain_id(&mut self, chain_id: Option<u64>) {
        self.revm.env.tx.chain_id = chain_id;
    }
    pub fn tx_nonce(&mut self, nonce: Option<u64>) {
        self.revm.env.tx.nonce = nonce;
    }
    pub fn tx_data(&mut self, data: &[u8]) {
        self.revm.env.tx.data = data.to_vec().into();
    }
    pub fn tx_transact_to_create(&mut self) {
        self.revm.env.tx.transact_to = TransactTo::create();
    }
    pub fn tx_transact_to_call(&mut self, to: BN) {
        self.revm.env.tx.transact_to = TransactTo::Call(to.try_into().unwrap());
    }
    pub fn tx_accessed_account(&mut self, account: AccessedAccount) {
        self.revm.env.tx.access_list.push(account.into())
    }
}

/// Struct that allows setting AccessList for transaction.
#[wasm_bindgen]
pub struct AccessedAccount {
    account: H160,
    slots: Vec<U256>,
}

impl From<AccessedAccount> for (H160, Vec<U256>) {
    fn from(from: AccessedAccount) -> Self {
        (from.account, from.slots)
    }
}

impl AccessedAccount {
    pub fn new(account: BN) -> Self {
        Self {
            account: account.try_into().unwrap(),
            slots: Vec::new(),
        }
    }
    pub fn slot(&mut self, slot: BN) {
        self.slots.push(slot.try_into().unwrap())
    }
}
