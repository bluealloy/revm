use crate::burntpix::genesis_alloc::BURNTPIX_ADDRESS;
use crate::burntpix::genesis_alloc::GENESIS_ALLOCS;
use alloy_sol_macro::sol;
use alloy_sol_types::SolCall;
use criterion::Criterion;
use regex::bytes::Regex;
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        address, hex, keccak256, AccountInfo, Bytecode, ExecutionResult, Output, TransactTo, U256,
    },
    Evm,
};
use revm_precompile::B256;
use std::error::Error;
use std::fs::File;
use std::time::Duration;
use std::{io::Write, str::FromStr};
pub mod genesis_alloc;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    interface IBURNTPIX {
        function run( uint32 seed, uint256 iterations) returns (string);
    }
}
const DEFAULT_SEED: &str = "0";
const DEFAULT_ITERATIONS: &str = "0x7A120";

pub fn burntpix(c: &mut Criterion) {
    let (seed, iterations) = try_init_env_vars().expect("Failed to parse env vars");

    let run_call_data = IBURNTPIX::runCall { seed, iterations }.abi_encode();

    let db = init_db();

    let mut g = c.benchmark_group("burntpix");
    g.noise_threshold(0.03)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(130))
        .sample_size(10);

    let mut evm = Evm::builder()
        .modify_tx_env(|tx| {
            tx.caller = address!("1000000000000000000000000000000000000000");
            tx.transact_to = TransactTo::Call(BURNTPIX_ADDRESS.clone());
            tx.data = run_call_data.clone().into();
        })
        .with_db(db)
        .build();

    let id = format!("burntpix");
    g.bench_function(id, |b| b.iter(|| evm.transact().unwrap()));

    // transact again to get the return data and create the svg
    let tx_result = evm.transact().unwrap().result;
    let return_data = match tx_result {
        ExecutionResult::Success {
            output, gas_used, ..
        } => {
            println!("Gas used: {:?}", gas_used);
            match output {
                Output::Call(value) => value,
                _ => unreachable!("Unexpected output type"),
            }
        }
        _ => unreachable!("Execution failed: {:?}", tx_result),
    };

    // remove returndata offset and length from output
    let data = &return_data[64..];

    // remove trailing zeros
    let re = Regex::new(r"[0\x00]+$").unwrap();
    let trimmed_data = re.replace_all(data, &[]);
    let file_name = format!("{}_{}", seed, iterations);

    svg(file_name, &trimmed_data).expect("Failed to store svg");
}
fn svg(filename: String, svg_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    let svg_dir = current_dir.join("benches").join("burntpix").join("svgs");
    std::fs::create_dir_all(&svg_dir)?;

    let file_path = svg_dir.join(format!("{}.svg", filename));
    let mut file = File::create(file_path)?;
    file.write_all(svg_data)?;

    Ok(())
}

fn try_init_env_vars() -> Result<(u32, U256), Box<dyn Error>> {
    let seed_from_env = std::env::var("SEED").unwrap_or(DEFAULT_SEED.to_string());
    let seed: u32 = seed_from_env.parse()?;
    let iterations_from_env = std::env::var("ITERATIONS").unwrap_or(DEFAULT_ITERATIONS.to_string());
    let iterations = U256::from_str(&iterations_from_env)?;
    Ok((seed, iterations))
}

fn init_db() -> CacheDB<EmptyDB> {
    let mut cache_db = CacheDB::new(EmptyDB::default());
    for (addr, state) in GENESIS_ALLOCS.iter() {
        let code = state.code.clone().expect("Code is required");
        let code_hash = hex::encode(keccak256(&code));
        let account_info = AccountInfo::new(
            state.balance,
            state.nonce.unwrap_or(0),
            B256::from_str(&code_hash).unwrap(),
            Bytecode::new_raw(code.into()),
        );

        cache_db.insert_account_info(*addr, account_info);

        if let Some(storage) = &state.storage {
            for (key, value) in storage.iter() {
                cache_db
                    .insert_account_storage(*addr, *key, *value)
                    .unwrap();
            }
        }
    }
    cache_db
}
