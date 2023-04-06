use std::{str::FromStr, sync::Arc};
use anyhow::{Ok, Result};
use bytes::Bytes;
use ethers::{
    abi::parse_abi,
    prelude::BaseContract,
    providers::{Http, Provider},
};
use revm::{
    db::{CacheDB, EmptyDB, EthersDB},
    primitives::{ExecutionResult, Output, TransactTo, B160, U256 as rU256},
    Database, EVM,
};

#[tokio::main]
async fn main() -> Result<()> {
    // create ethers client and wrap it in Arc<M>
    let client = Provider::<Http>::try_from(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
    )?;
    let client = Arc::new(client);

    // ----------------------------------------------------------- //
    //             Storage slots of UniV2Pair contract             //
    // =========================================================== //
    // storage[5] = factory: address                               //
    // storage[6] = token0: address                                //
    // storage[7] = token1: address                                //
    // storage[8] = (res0, res1, ts): (uint112, uint112, uint32)   //
    // storage[9] = price0CumulativeLast: uint256                  //
    // storage[10] = price1CumulativeLast: uint256                 //
    // storage[11] = kLast: uint256                                //
    // =========================================================== //

    // choose slot of storage that you would like to transact with
    let slot = rU256::from(8);

    // ETH/USDT pair on Uniswap V2
    let pool_address = B160::from_str("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852")?;

    // generate abi for the calldata from the human readable interface
    let abi = BaseContract::from(
        parse_abi(&[
            "function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)",
        ])?
    );

    // encode abi into Bytes
    let encoded = abi.encode("getReserves", ())?;

    // initialize new EthersDB
    let mut ethersdb = EthersDB::new(Arc::clone(&client), None).unwrap();

    // query basic properties of an account incl bytecode
    let acc_info = ethersdb.basic(pool_address).unwrap().unwrap();

    // query value of storage slot at account address
    let value = ethersdb.storage(pool_address, slot).unwrap();

    // initialise empty in-memory-db
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // insert basic account info which was generated via Web3DB with the corresponding address
    cache_db.insert_account_info(pool_address, acc_info);

    // insert our pre-loaded storage slot to the corresponding contract key (address) in the DB
    cache_db
        .insert_account_storage(pool_address, slot, value)
        .unwrap();

    // initialise an empty (default) EVM
    let mut evm = EVM::new();

    // insert pre-built database from above
    evm.database(cache_db);

    // fill in missing bits of env struc
    // change that to whatever caller you want to be
    evm.env.tx.caller = B160::from_str("0x0000000000000000000000000000000000000000")?;
    // account you want to transact with
    evm.env.tx.transact_to = TransactTo::Call(pool_address);
    // calldata formed via abigen
    evm.env.tx.data = Bytes::from(hex::decode(hex::encode(&encoded))?);
    // transaction value in wei
    evm.env.tx.value = rU256::try_from(0)?;

    // execute transaction without writing to the DB
    let ref_tx = evm.transact_ref().unwrap();
    // select ExecutionResult struct
    let result = ref_tx.result;

    // unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(value) => Some(value),
            _ => None,
        },
        _ => None,
    };

    // decode bytes to reserves + ts via ethers-rs's abi decode
    let (reserve0, reserve1, ts): (u128, u128, u32) =
        abi.decode_output("getReserves", value.unwrap())?;

    // Print emulated getReserves() call output
    println!("Reserve0: {:#?}", reserve0);
    println!("Reserve1: {:#?}", reserve1);
    println!("Timestamp: {:#?}", ts);

    Ok(())
}
