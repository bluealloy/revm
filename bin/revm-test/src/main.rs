use std::{str::FromStr, time::Instant};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use revm::{
    db::{Database, StateDB},
    evm::{ExtHandler, Handler, EVM},
    machine::Machine,
    models::*,
    spec::BerlinSpec,
};

use hex::{self, ToHex};

extern crate alloc;

fn main() {
    println!("Hello, world!");
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
    let mut evm = EVM::<BerlinSpec, StateDB>::new(&mut db, envs);
    let timestamp = Instant::now();
    //for _ in 0..10000 {
    let exit_reason = evm.create(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        U256::zero(),
        //hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000b71b00")
        //	.unwrap(),
        Bytes::from(hex::decode("608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80632e64cec11461003b5780636057361d14610059575b600080fd5b610043610075565b60405161005091906100d9565b60405180910390f35b610073600480360381019061006e919061009d565b61007e565b005b60008054905090565b8060008190555050565b60008135905061009781610103565b92915050565b6000602082840312156100b3576100b26100fe565b5b60006100c184828501610088565b91505092915050565b6100d3816100f4565b82525050565b60006020820190506100ee60008301846100ca565b92915050565b6000819050919050565b600080fd5b61010c816100f4565b811461011757600080fd5b5056fea2646970667358221220404e37f487a89a932dca5e77faaf6ca2de3b991f93d230604b1b8daaef64766264736f6c63430008070033")
            .unwrap()),
        CreateScheme::Create,
        u64::MAX,
        Vec::new(),
    );
    println!("EVM CREATE({:?}):{:?}", timestamp.elapsed(), exit_reason);
    let timestamp = Instant::now();

    let output = evm.call(               
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0xa521a7d4fd9bd91af46cd678f4636dffb991742a").unwrap(),
        U256::zero(),
        Bytes::from(hex::decode(
            "6057361d0000000000000000000000000000000000000000000000000000000000001111",
        )
        .unwrap()),
        u64::MAX,
        Vec::new(),
    );
    println!("EVM CALL({:?}):{:?}", timestamp.elapsed(), output);
    let timestamp = Instant::now();

    let output = evm.call(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0xa521a7d4fd9bd91af46cd678f4636dffb991742a").unwrap(),
        U256::zero(),
        //hex::decode("0f14a4060000000000000000000000000000000000000000000000000000000000b71b00")
        //	.unwrap(),
        Bytes::from(hex::decode("2e64cec1").unwrap()),
        u64::MAX,
        Vec::new(),
    );
    println!("EVM CALL({:?}):{:?}", timestamp.elapsed(), output);
}

// will need to create workspace and extract these stuff to saparate binary crate
