use bytes::{Bytes, BytesMut};
use primitive_types::{H160, H256, U256};
use revm::{AccountInfo, DatabaseCommit, DummyStateDB, SpecId, TransactTo, EVM as rEVM};
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

/// Wrapper arround revm with DummyStateDB
#[wasm_bindgen]
pub struct EVM {
    revm: rEVM<DummyStateDB>,
}

#[wasm_bindgen]
impl EVM {
    pub fn new() -> EVM {
        console_log!("EVM created");
        let mut evm = EVM { revm: rEVM::new() };
        evm.revm.database(DummyStateDB::new());
        evm
    }

    pub fn transact(&mut self) -> u64 {
        let (exit, data, gas, state) = self.revm.transact();
        console_log!(
            "Transact done, exit:{:?}, gas:{:?}, data:{:?}\nstate_chage:{:?}",
            exit,
            gas,
            data,
            state
        );
        self.revm.db().unwrap().commit(state);
        gas
    }

    /****** DATABASE RELATED ********/
    pub fn insert_account(&mut self, address: &[u8], nonce: u64, balance: &[u8], code: &[u8]) {
        let address = H160::from_slice(address);
        let acc_info = AccountInfo::new(
            U256::from_big_endian(balance),
            nonce,
            Bytes::copy_from_slice(code),
        );
        console_log!("Added account:{:?} info:{:?}", address, acc_info);
        self.revm.db().unwrap().insert_cache(address, acc_info);
    }

    /****** ALL CFG ENV SETTERS ********/

    pub fn cfg_chain_id(&mut self, gas_limit: &[u8]) {
        self.revm.env.cfg.chain_id = U256::from_big_endian(gas_limit);
    }
    pub fn cfg_spec_id(&mut self, spec_id: u8) {
        self.revm.env.cfg.spec_id = SpecId::try_from_u8(spec_id).unwrap_or_else(|| SpecId::LATEST);
    }

    /****** ALL BLOCK ENV SETTERS ********/

    pub fn block_gas_limit(&mut self, gas_limit: &[u8]) {
        self.revm.env.block.gas_limit = U256::from_big_endian(gas_limit);
    }
    pub fn block_number(&mut self, number: &[u8]) {
        self.revm.env.block.number = U256::from_big_endian(number);
    }
    pub fn block_coinbase(&mut self, coinbase: &[u8]) {
        self.revm.env.block.coinbase = H160::from_slice(coinbase);
    }
    pub fn block_timestamp(&mut self, timestamp: &[u8]) {
        self.revm.env.block.timestamp = U256::from_big_endian(timestamp);
    }
    pub fn block_difficulty(&mut self, difficulty: &[u8]) {
        self.revm.env.block.difficulty = U256::from_big_endian(difficulty);
    }
    pub fn block_basefee(&mut self, basefee: &[u8]) {
        self.revm.env.block.basefee = U256::from_big_endian(basefee);
    }
    pub fn block_gas_used(&mut self, gas_used: &[u8]) {
        self.revm.env.block.gas_used = U256::from_big_endian(gas_used);
    }

    /****** ALL TX ENV SETTERS ********/

    pub fn tx_caller(&mut self, tx_caller: &[u8]) {
        self.revm.env.tx.caller = H160::from_slice(tx_caller);
    }
    pub fn tx_gas_limit(&mut self, gas_limit: u64) {
        self.revm.env.tx.gas_limit = gas_limit;
    }
    pub fn tx_gas_price(&mut self, gas_price: &[u8]) {
        self.revm.env.tx.gas_price = U256::from_big_endian(gas_price);
    }
    pub fn tx_gas_priority_fee(&mut self, gas_priority_fee: &[u8]) {
        self.revm.env.tx.gas_priority_fee = if gas_priority_fee.len() == 0 {
            None
        } else {
            Some(U256::from_big_endian(gas_priority_fee))
        };
    }
    pub fn tx_value(&mut self, value: &[u8]) {
        self.revm.env.tx.value = U256::from_big_endian(value);
    }
    pub fn tx_chain_id(&mut self, chain_id: Option<u64>) {
        self.revm.env.tx.chain_id = chain_id;
    }
    pub fn tx_nonce(&mut self, nonce: Option<u64>) {
        self.revm.env.tx.nonce = nonce;
    }
    pub fn tx_data(&mut self, data: &[u8]) {
        self.revm.env.tx.data = BytesMut::from(data).freeze();
    }
    pub fn tx_transact_to_create(&mut self) {
        self.revm.env.tx.transact_to = TransactTo::create();
    }
    pub fn tx_transact_to_call(&mut self, to: &[u8]) {
        self.revm.env.tx.transact_to = TransactTo::Call(H160::from_slice(to));
    }
    pub fn tx_accessed_account(&mut self, account: AccessedAccount) {
        self.revm.env.tx.access_list.push(account.into())
    }
}

/// Struct that allows setting AccessList for transaction.
#[wasm_bindgen]
pub struct AccessedAccount {
    account: H160,
    slots: Vec<H256>,
}

impl Into<(H160, Vec<H256>)> for AccessedAccount {
    fn into(self) -> (H160, Vec<H256>) {
        (self.account, self.slots)
    }
}

#[wasm_bindgen]
impl AccessedAccount {
    pub fn new(account: &[u8]) -> Self {
        Self {
            account: H160::from_slice(account),
            slots: Vec::new(),
        }
    }
    pub fn slot(&mut self, slot: &[u8]) {
        self.slots.push(H256::from_slice(slot))
    }
}
