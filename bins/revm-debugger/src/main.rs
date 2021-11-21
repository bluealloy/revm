
mod commands;
mod debugger;
mod runner;

use std::{env, path::PathBuf, str::FromStr};

use primitive_types::{H160, H256, U256};
use revm::{EVM, Env, TransactTo, db::Web3DB};
use bytes::Bytes;


pub fn main() {
    // TODO
    // full env should be cfg
    
    let args: Vec<String> = env::args().collect();
    println!("args:{:?}", args);

    let db = Web3DB::new("https://mainnet.infura.io/v3/0954246eab5544e89ac236b668980810",None).unwrap();

    let mut revm = EVM::new();
    revm.database(db);
    revm.env.cfg.perf_all_precompiles_have_balance = true;
    revm.env.tx.caller = H160::from_str("0x393616975ff5A88AAB4568983C1dcE96FBb5b67a").unwrap();
    revm.env.tx.value = U256::from(11234);
    revm.env.tx.transact_to = TransactTo::Call(H160::from_str("0x393616975ff5A88AAB4568983C1dcE96FBb5b67b").unwrap());

    //let input: Bytes = hex::decode(&args[1]).unwrap().into();
    println!("STATE OUT:{:?}",revm.transact());
}
