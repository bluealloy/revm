use std::sync::Arc;

use anyhow::{Ok, Result};
use bytes::Bytes;
use ethers::{
    abi::parse_abi,
    prelude::BaseContract,
    providers::{Http, Middleware, Provider},
    types::{Address, H256, U256},
};
use revm::{
    db::{CacheDB, EmptyDB, Web3DB},
    Database, TransactOut, TransactTo, EVM,
};

#[tokio::main]
async fn main() -> Result<()> {
    // create ethers client
    let client = Provider::<Http>::try_from(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
    )?;
    let client = Arc::new(client);

    // ----------------------------------------------------------- //
    //         Storage slots of a UniV2Pair like contract          //
    // *********************************************************** //
    // location[5] = factory: address                              //
    // location[6] = token0: address                               //
    // location[7] = token1: address                               //
    // location[8] = (res0, res1, ts): (uint112, uint112, uint32)  //
    // location[9] = price0CumulativeLast: uint256                 //
    // location[10] = price1CumulativeLast: uint256                //
    // location[11] = kLast: uint256                               //
    // *********************************************************** //

    // choose index of storage that you would like to transact with
    let index = 8;
    let location = H256::from_low_u64_be(index);

    // ETH/USDT pair on Uniswap V2
    let pool_address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse::<Address>()?;

    // load the reserve slot via ethers-rs
    let reserves_slot = client.get_storage_at(pool_address, location, None).await?;

    let mem = U256::from(reserves_slot.as_bytes());

    // bitshitOoor to extract uint112, uint112, uint32 from uint256
    let ts_raw = (mem >> (2 * 112)).as_u32();
    let reserve1_raw = ((mem >> 112) & ((U256::from(1u8) << 112) - 1u8)).as_u128();
    let reserve0_raw = (mem & ((U256::from(1u8) << 112) - 1u8)).as_u128();

    // generate abi for the calldata from the human readable interface
    let abi = BaseContract::from(
        parse_abi(&[
            "function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)",
        ])?
    );

    // encode abi into Bytes
    let encoded = abi.encode("getReserves", ())?;

    // initial new Web3DB
    let mut web3db = Web3DB::new(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        None,
    )
    .unwrap();

    // query basic properties of an account incl bytecode
    let acc_info = web3db.basic(pool_address).unwrap().unwrap();

    // initialise empty in-memory-db
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // insert basic account info which was generated via Web3DB with the corresponding address
    cache_db.insert_account_info(pool_address, acc_info);

    // insert our pre-loaded storage slot to the corresponding contract key (address) in the DB
    cache_db
        .insert_account_storage(
            pool_address,
            U256::from(index),
            U256::from(reserves_slot.to_fixed_bytes()),
        )
        .unwrap();

    // initialise an empty (default) EVM
    let mut evm = EVM::new();

    // insert pre-built database from above
    evm.database(cache_db);

    // fill in missing bits of env struc
    // change that to whatever caller you want to be
    evm.env.tx.caller = "0x0000000000000000000000000000000000000000".parse::<Address>()?;
    // account you want to transact with
    evm.env.tx.transact_to = TransactTo::Call(pool_address);
    // calldata formed via abigen
    evm.env.tx.data = Bytes::from(hex::decode(hex::encode(&encoded))?);
    // transaction value in wei
    evm.env.tx.value = U256::try_from(0)?;

    // execute transaction without writing to the DB
    let ref_tx = evm.transact_ref();
    // select ExecutionResult struct
    let result = ref_tx.0;

    // unpack output call enum into raw bytes
    let value = match result.out {
        TransactOut::Call(value) => Some(value),
        _ => None,
    };

    // decode bytes to reserves + ts via ethers-rs's abi decode
    let (reserve0, reserve1, ts): (u128, u128, u32) =
        abi.decode_output("getReserves", value.unwrap())?;

    // Print emualted getReserves() call output
    println!("Reserve0: {:#?}", reserve0);
    println!("Reserve1: {:#?}", reserve1);
    println!("Timestamp: {:#?}", ts);

    // make sure that the emulator's call is exactly the same as the raw call
    assert_eq!(reserve0_raw, reserve0);
    assert_eq!(reserve1_raw, reserve1);
    assert_eq!(ts_raw, ts);

    // GG if you are here with no errors
    Ok(())
}
