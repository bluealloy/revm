pub mod static_data;

use criterion::Criterion;
use primitives::{StorageKey, StorageValue};
use static_data::{
    BURNTPIX_ADDRESS_ONE, BURNTPIX_ADDRESS_THREE, BURNTPIX_ADDRESS_TWO, BURNTPIX_BYTECODE_FOUR,
    BURNTPIX_BYTECODE_ONE, BURNTPIX_BYTECODE_THREE, BURNTPIX_BYTECODE_TWO, BURNTPIX_MAIN_ADDRESS,
    STORAGE_ONE, STORAGE_TWO, STORAGE_ZERO,
};

use alloy_sol_types::{sol, SolCall};
use database::{CacheDB, BENCH_CALLER};
use revm::{
    database_interface::EmptyDB,
    primitives::{hex, keccak256, Address, Bytes, TxKind, B256, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

use std::{error::Error, fs::File, io::Write};

use std::str::FromStr;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    interface IBURNTPIX {
        function run( uint32 seed, uint256 iterations) returns (string);
    }
}

pub fn run(criterion: &mut Criterion) {
    let (seed, iterations) = try_init_env_vars().expect("Failed to parse env vars");

    let run_call_data = IBURNTPIX::runCall { seed, iterations }.abi_encode();

    let db = init_db();

    let mut evm = Context::mainnet()
        .with_db(db)
        .modify_tx_chained(|tx| {
            tx.caller = BENCH_CALLER;
            tx.kind = TxKind::Call(BURNTPIX_MAIN_ADDRESS);
            tx.data = run_call_data.clone().into();
            tx.gas_limit = u64::MAX;
        })
        .build_mainnet();

    criterion.bench_function("burntpix", |b| {
        b.iter(|| {
            evm.replay().unwrap();
        })
    });

    //Collects the data and uses it to generate the svg after running the benchmark
    /*
    let tx_result = evm.replay().unwrap();
    let return_data = match tx_result.result {
        context::result::ExecutionResult::Success {
            output, gas_used, ..
        } => {
            println!("Gas used: {:?}", gas_used);
            match output {
                context::result::Output::Call(value) => value,
                _ => unreachable!("Unexpected output type"),
            }
        }
        _ => unreachable!("Execution failed: {:?}", tx_result),
    };

    // Remove returndata offset and length from output
    let returndata_offset = 64;
    let data = &return_data[returndata_offset..];

    // Remove trailing zeros
    let trimmed_data = data
        .split_at(data.len() - data.iter().rev().filter(|&x| *x == 0).count())
        .0;
    let file_name = format!("{}_{}", seed, iterations);

    svg(file_name, trimmed_data).expect("Failed to store svg");
    */
}

/// Actually generates the svg
pub fn svg(filename: String, svg_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let svg_dir = current_dir.join("burntpix").join("svgs");
    std::fs::create_dir_all(&svg_dir)?;

    let file_path = svg_dir.join(format!("{}.svg", filename));
    let mut file = File::create(file_path)?;
    file.write_all(svg_data)?;

    Ok(())
}

const DEFAULT_SEED: &str = "0";
const DEFAULT_ITERATIONS: &str = "0x4E20"; // 20_000 iterations
fn try_init_env_vars() -> Result<(u32, U256), Box<dyn Error>> {
    let seed_from_env = std::env::var("SEED").unwrap_or(DEFAULT_SEED.to_string());
    let seed: u32 = try_from_hex_to_u32(&seed_from_env)?;
    let iterations_from_env = std::env::var("ITERATIONS").unwrap_or(DEFAULT_ITERATIONS.to_string());
    let iterations = U256::from_str(&iterations_from_env)?;
    Ok((seed, iterations))
}

fn try_from_hex_to_u32(hex: &str) -> Result<u32, Box<dyn Error>> {
    let trimmed = hex.strip_prefix("0x").unwrap_or(hex);
    u32::from_str_radix(trimmed, 16).map_err(|e| format!("Failed to parse hex: {}", e).into())
}

fn insert_account_info(cache_db: &mut CacheDB<EmptyDB>, addr: Address, code: &str) {
    let code = Bytes::from(hex::decode(code).unwrap());
    let code_hash = hex::encode(keccak256(&code));
    let account_info = AccountInfo::new(
        U256::from(0),
        0,
        B256::from_str(&code_hash).unwrap(),
        Bytecode::new_raw(code),
    );
    cache_db.insert_account_info(addr, account_info);
}

fn init_db() -> CacheDB<EmptyDB> {
    let mut cache_db = CacheDB::new(EmptyDB::default());

    insert_account_info(&mut cache_db, BURNTPIX_ADDRESS_ONE, BURNTPIX_BYTECODE_ONE);
    insert_account_info(&mut cache_db, BURNTPIX_MAIN_ADDRESS, BURNTPIX_BYTECODE_TWO);
    insert_account_info(&mut cache_db, BURNTPIX_ADDRESS_TWO, BURNTPIX_BYTECODE_THREE);
    insert_account_info(
        &mut cache_db,
        BURNTPIX_ADDRESS_THREE,
        BURNTPIX_BYTECODE_FOUR,
    );

    cache_db
        .insert_account_storage(
            BURNTPIX_MAIN_ADDRESS,
            StorageKey::from(0),
            StorageValue::from_be_bytes(*STORAGE_ZERO),
        )
        .unwrap();

    cache_db
        .insert_account_storage(
            BURNTPIX_MAIN_ADDRESS,
            StorageKey::from(1),
            StorageValue::from_be_bytes(*STORAGE_ONE),
        )
        .unwrap();

    cache_db
        .insert_account_storage(
            BURNTPIX_MAIN_ADDRESS,
            StorageValue::from(2),
            StorageKey::from_be_bytes(*STORAGE_TWO),
        )
        .unwrap();

    cache_db
}
