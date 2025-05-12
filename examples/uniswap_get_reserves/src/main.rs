//! Example of uniswap getReserves() call emulation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_eips::BlockId;
use alloy_provider::ProviderBuilder;
use alloy_sol_types::{sol, SolCall};
use revm::{
    context_interface::result::{ExecutionResult, Output},
    database::{AlloyDB, CacheDB},
    database_interface::{DatabaseRef, EmptyDB, WrapDatabaseAsync},
    primitives::{address, StorageKey, TxKind, U256},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the Alloy provider and database
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    let provider = ProviderBuilder::new().connect(rpc_url).await?;

    let alloy_db = WrapDatabaseAsync::new(AlloyDB::new(provider, BlockId::latest())).unwrap();
    let cache_db = CacheDB::new(alloy_db);

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

    // Choose slot of storage that you would like to transact with
    let slot = StorageKey::from(8);

    // ETH/USDT pair on Uniswap V2
    let pool_address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");

    // Generate abi for the calldata from the human readable interface
    sol! {
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }

    // Encode abi into Bytes
    let encoded = getReservesCall::new(()).abi_encode();

    // Query basic properties of an account incl bytecode
    let acc_info = cache_db.basic_ref(pool_address).unwrap().unwrap();

    // Query value of storage slot at account address
    let value = cache_db.storage_ref(pool_address, slot).unwrap();

    // Initialise empty in-memory-db
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Insert basic account info which was generated via Web3DB with the corresponding address
    cache_db.insert_account_info(pool_address, acc_info);

    // Insert our pre-loaded storage slot to the corresponding contract key (address) in the DB
    cache_db
        .insert_account_storage(pool_address, slot, value)
        .unwrap();

    // Initialise an empty (default) EVM
    let mut evm = Context::mainnet()
        .with_db(cache_db)
        .modify_tx_chained(|tx| {
            // fill in missing bits of env struct
            // change that to whatever caller you want to be
            tx.caller = address!("0000000000000000000000000000000000000000");
            // account you want to transact with
            tx.kind = TxKind::Call(pool_address);
            // calldata formed via abigen
            tx.data = encoded.into();
            // transaction value in wei
            tx.value = U256::from(0);
        })
        .build_mainnet();

    // Execute transaction without writing to the DB
    let ref_tx = evm.replay().unwrap();
    // Select ExecutionResult struct
    let result = ref_tx.result;

    // Unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        _ => panic!("Execution failed: {result:?}"),
    };

    // Decode bytes to reserves + ts via alloy's abi decode
    let return_vals = getReservesCall::abi_decode_returns(&value)?;

    // Print emulated getReserves() call output
    println!("Reserve0: {:#?}", return_vals.reserve0);
    println!("Reserve1: {:#?}", return_vals.reserve1);
    println!("Timestamp: {:#?}", return_vals.blockTimestampLast);

    Ok(())
}
