use std::str::FromStr;

use bytes::Bytes;
use primitive_types::{H160, U256};
use revm::{AccountInfo, DummyStateDB, NoOpInspector, TransactOut, TransactTo, KECCAK_EMPTY};

extern crate alloc;

pub fn simple_example() {
    let caller = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
    // StateDB is dummy state that implements Database trait.
    // add one account and some eth for testing.
    let mut evm = revm::new();
    evm.database(DummyStateDB::new());
    evm.db().unwrap().insert_cache(
        caller.clone(),
        AccountInfo {
            nonce: 1,
            balance: U256::from(10000000),
            code: None,
            code_hash: KECCAK_EMPTY,
        },
    );

    // execution globals block hash/gas_limit/coinbase/timestamp..
    evm.env.tx.caller = caller.clone();

    evm.env.tx.transact_to = TransactTo::create();
    evm.env.tx.data = Bytes::from(hex::decode("608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100d9565b60405180910390f35b610073600480360381019061006e919061009d565b61007e565b005b60008054905090565b8060008190555050565b60008135905061009781610103565b92915050565b6000602082840312156100b3576100b26100fe565b5b60006100c184828501610088565b91505092915050565b6100d3816100f4565b82525050565b60006020820190506100ee60008301846100ca565b92915050565b6000819050919050565b600080fd5b61010c816100f4565b811461011757600080fd5b5056fea2646970667358221220404e37f487a89a932dca5e77faaf6ca2de3b991f93d230604b1b8daaef64766264736f6c63430008070033").unwrap());

    let (_, out, _) = evm.transact_commit();
    let contract_address = match out {
        TransactOut::Create(_, Some(add)) => add,
        _ => panic!("not gona happen"),
    };

    evm.env.tx.transact_to = TransactTo::Call(contract_address);
    evm.env.tx.data =
        hex::decode("6057361d0000000000000000000000000000000000000000000000000000000000000015")
            .unwrap()
            .into();
    evm.transact();

    evm.env.tx.transact_to = TransactTo::Call(contract_address);
    evm.env.tx.data = hex::decode("2e64cec1").unwrap().into();

    let (_, out, _) = evm.transact_commit();

    println!("get value (StaticCall): {:?}\n", out);
}

fn main() {
    println!("Hello, world!");
    simple_example();
    return;
}
