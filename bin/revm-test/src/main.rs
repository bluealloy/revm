use std::{str::FromStr, time::Instant};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use revm::{AccountInfo, BerlinSpec, BerlinSpecStatic, CreateScheme, GlobalEnv, StateDB, EVM};

use hex::{self, ToHex};

extern crate alloc;

pub fn simple_example() {
    let caller = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
    let create_data = Bytes::from(hex::decode("608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100d9565b60405180910390f35b610073600480360381019061006e919061009d565b61007e565b005b60008054905090565b8060008190555050565b60008135905061009781610103565b92915050565b6000602082840312156100b3576100b26100fe565b5b60006100c184828501610088565b91505092915050565b6100d3816100f4565b82525050565b60006020820190506100ee60008301846100ca565b92915050565b6000819050919050565b600080fd5b61010c816100f4565b811461011757600080fd5b5056fea2646970667358221220404e37f487a89a932dca5e77faaf6ca2de3b991f93d230604b1b8daaef64766264736f6c63430008070033").unwrap());
    let call_data_set = Bytes::from(
        hex::decode("6057361d0000000000000000000000000000000000000000000000000000000000000015")
            .unwrap(),
    );
    let call_data_get = Bytes::from(hex::decode("2e64cec1").unwrap());

    // StateDB is dummy state that implements Database trait.
    // add one account and some eth for testing.
    let mut db = StateDB::new();
    db.insert_cache(
        caller.clone(),
        AccountInfo {
            nonce: 1,
            balance: U256::from(10000000),
            code: None,
            code_hash: None,
        },
    );

    // execution globals block hash/gas_limit/coinbase/timestamp..
    let envs = GlobalEnv::default();
    let res = EVM::new(&mut db, envs.clone()).create::<BerlinSpec>(
        caller,
        U256::zero(), // value transfered
        create_data,
        CreateScheme::Create,
        u64::MAX,   // gas_limit
        Vec::new(), // access_list
    );
    println!("create simple set/get smart contract:{:?}\n", res);
    db.apply(res.3);
    let contract_address = res.1.unwrap();

    let res = EVM::new(&mut db, envs.clone()).call::<BerlinSpec>(
        caller,
        contract_address,
        U256::zero(), // value transfered
        call_data_set,
        u64::MAX,   //gas_limit
        Vec::new(), // access_list
    );
    println!("set value: {:?}\n", res);
    db.apply(res.3);

    let res = EVM::new(&mut db, envs.clone()).call::<BerlinSpecStatic>(
        caller,
        contract_address,
        U256::zero(), // value transfered
        call_data_get,
        u64::MAX,   // gas_limit
        Vec::new(), // access_list
    );
    println!("get value (StaticCall): {:?}\n", res);
    db.apply(res.3);
}

fn main() {
    println!("Hello, world!");
    simple_example();
    return;
    let mut db = StateDB::new();
    // Insert cache
    db.insert_cache(H160::from_str("0x1000000000000000000000000000000000000000").unwrap(), AccountInfo {
        nonce: 1,
        balance: U256::from(10000000),
        code: Some(Bytes::from(hex::decode("6080604052348015600f57600080fd5b506004361060285760003560e01c80630f14a40614602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b6000806000905060005b83811015608f5760018201915080806001019150506076565b508091505091905056fea26469706673582212202bc9ec597249a9700278fe4ce78da83273cb236e76d4d6797b441454784f901d64736f6c63430007040033").unwrap())),
        code_hash: None,
    });

    db.insert_cache(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        AccountInfo {
            nonce: 2,
            balance: U256::from(10000000),
            code: None,
            code_hash: None,
        },
    );

    let envs = GlobalEnv::default();
    let timestamp = Instant::now();
    let res = {
        let mut evm = EVM::<StateDB>::new(&mut db, envs.clone());
        //for _ in 0..10000 {
        evm.create::<BerlinSpec>(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        U256::zero(),
        //hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000b71b00")
        //	.unwrap(),
        Bytes::from(hex::decode("608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100d9565b60405180910390f35b610073600480360381019061006e919061009d565b61007e565b005b60008054905090565b8060008190555050565b60008135905061009781610103565b92915050565b6000602082840312156100b3576100b26100fe565b5b60006100c184828501610088565b91505092915050565b6100d3816100f4565b82525050565b60006020820190506100ee60008301846100ca565b92915050565b6000819050919050565b600080fd5b61010c816100f4565b811461011757600080fd5b5056fea2646970667358221220404e37f487a89a932dca5e77faaf6ca2de3b991f93d230604b1b8daaef64766264736f6c63430008070033")
            .unwrap()),
        CreateScheme::Create,
        u64::MAX,
        Vec::new(),
    )
    };
    println!("\nEVM CREATE({:?}):{:?}\n", timestamp.elapsed(), res);
    db.apply(res.3);
    let timestamp = Instant::now();

    let res = {
        let mut evm = EVM::<StateDB>::new(&mut db, envs.clone());
        evm.call::<BerlinSpec>(
            H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
            H160::from_str("0xa521a7d4fd9bd91af46cd678f4636dffb991742a").unwrap(),
            U256::zero(),
            hex::decode("6057361d0000000000000000000000000000000000000000001000003000004005000415")
                .unwrap()
                .into(),
            u64::MAX,
            Vec::new(),
        )
    };
    println!("\nEVM CALL({:?}):{:?}\n", timestamp.elapsed(), res);
    db.apply(res.3);
    let timestamp = Instant::now();

    let res = {
        let mut evm = EVM::<StateDB>::new(&mut db, envs);
        evm.call::<BerlinSpecStatic>(
            H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
            H160::from_str("0xa521a7d4fd9bd91af46cd678f4636dffb991742a").unwrap(),
            U256::zero(),
            //hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000b71b00")
            //	.unwrap(),
            hex::decode("2e64cec1").unwrap().into(),
            u64::MAX,
            Vec::new(),
        )
    };
    println!("\nEVM GET CALL({:?}):{:?}\n", timestamp.elapsed(), res);
    db.apply(res.3);

    // let out = evm.call(
    //     H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
    //     H160::from_str("0xa521a7d4fd9bd91af46cd678f4636dffb991742a").unwrap(),
    //     U256::zero(),
    //     //hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000b71b00")
    //     //	.unwrap(),
    //     hex::decode("2e64cec1").unwrap().into(),
    //     U256::max_value(),
    //     Vec::new(),
    // );
    // println!("EVM GET CALL({:?}):{:?}", timestamp.elapsed(), output);
}

// will need to create workspace and extract these stuff to saparate binary crate
