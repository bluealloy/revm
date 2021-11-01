use core::str::FromStr;

use bytes::{Bytes, BytesMut};
use primitive_types::{H160, U256};
use revm::{
    AccountInfo, DummyStateDB, Env, TransactTo, TxEnv as revmTxEnv, EVM as rEVM, KECCAK_EMPTY,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(module = "bn.js")]
extern "C" {
    pub type BN;

    #[wasm_bindgen(constructor)]
    pub fn new(number: String, base: u32) -> BN;

    #[wasm_bindgen(method)]
    fn toString(this: &BN, base: u32) -> String;
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

#[wasm_bindgen]
pub struct EVM {
    revm: rEVM<DummyStateDB>,
    env: Env,
}

// #[wasm_bindgen]
// pub struct CfgEnv {
//     pub chain_id: &'static [u8],
//     pub spec_id: u8,
// }

// #[wasm_bindgen]
// pub struct BlockEnv {
//     pub gas_limit: &'static [u8],
//     /// somebody call it nonce
//     pub number: &'static [u8],
//     /// Coinbase or miner or address that created and signed the block.
//     /// Address where we are going to send gas spend
//     pub coinbase: &'static [u8],
//     pub timestamp: &'static [u8],
//     pub difficulty: &'static [u8],
//     /// basefee is added in EIP1559 London upgrade
//     pub basefee: &'static [u8],
//     /// incrementaly added on every transaction. It can be cleared if needed
//     pub gas_used: &'static [u8],
// }

#[wasm_bindgen]
pub fn greet(name: &str) -> u32 {
    //console_log!("TEST:{}", name);
    10
}

#[wasm_bindgen(js_name = ret15)]
pub fn ret15() -> u32 {
    15
}

#[wasm_bindgen]
impl EVM {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EVM {
        let mut evm = EVM {
            revm: rEVM::new(),
            env: Env::default(),
        };
        let caller: H160 = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
        evm.revm.db().unwrap().insert_cache(
            caller.clone(),
            AccountInfo {
                nonce: 1,
                balance: U256::from(10000000),
                code: None,
                code_hash: KECCAK_EMPTY,
            },
        );
        evm.revm.env.tx.caller = caller;

        evm
    }

    pub fn transact(&mut self) -> u64 {
        let (_, _, gas) = self.revm.transact_commit();
        gas
    }

    /****** ALL ENV SETTERS ********/

    pub fn env_tx_caller(&mut self, tx_caller: &[u8]) {
        self.env.tx.caller = H160::from_slice(tx_caller);
    }
    pub fn env_tx_data(&mut self, data: &[u8]) {
        self.env.tx.data = BytesMut::from(data).freeze();
    }
    pub fn env_tx_transact_to_create(&mut self) {
        self.env.tx.transact_to = TransactTo::create();
    }
    pub fn env_tx_transact_to_call(&mut self, to: &[u8]) {
        self.env.tx.transact_to = TransactTo::Call(H160::from_slice(to));
    }
}
