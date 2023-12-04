//! Custom Precompile Example
//!
//! This example walks through the process of creating a custom precompile and
//! executing it with an optimism deposit transaction.

use revm::inspectors::TracerEip3155;
use revm::primitives::{
    address, b256, AccountInfo, Bytes, CanyonSpec, Env, OptimismFields, SpecId, TransactTo, U256,
};
use revm::{EVMImpl, InMemoryDB, JournaledState, Transact};
use revm_precompile::{Precompile, PrecompileResult, Precompiles};
use std::io::stdout;

const MINIMUM_GAS_LIMIT: u8 = 0xFF;

// A standard precompile function that returns opcodes to send 100 wei to a
// specific address.
fn precompile_func(_input: &[u8], _gas_limit: u64) -> PrecompileResult {
    // CALL Stack Input
    // [gas,  to,     value, arg_offset, arg_size, ret_offset, ret_size]
    // [0xFF, caller, 100,   0,          0,        0,          0       ]
    println!("---> Inside the precompile function!");

    let zero = 0x00_u8;
    let push1 = 0x60_u8;
    let hundred = 0x64_u8;
    let push20 = 0x73_u8;
    let to = address!("deadca11deadca11deadca11deadca11deadca11");
    let call = 0xF1_u8;

    let gas_used = 0;
    let ret_bytes = [
        &[
            push1, zero, push1, zero, push1, zero, push1, zero, push1, hundred, push20,
        ],
        to.as_slice(),
        &[push1, MINIMUM_GAS_LIMIT, call],
    ]
    .concat();

    println!("---> Precompile function returning: {:x?}", ret_bytes);
    Ok((gas_used, ret_bytes))
}

fn main() -> anyhow::Result<()> {
    // Build the custom precompile
    let precompile_addr = address!("0000000000000000000000000000000000000420");
    let precompile = revm_precompile::PrecompileWithAddress(
        precompile_addr,
        Precompile::Standard(precompile_func),
    );
    let precompiles = Precompiles {
        inner: vec![precompile],
    };

    // Build the evm environment {block, cfg, tx}
    let mut env = Env::default();
    let caller = address!("deadca11deadca11deadca11deadca11deadca11");

    env.block.number = U256::from(113055114);
    env.block.coinbase = address!("deaddeaddeaddeaddeaddeaddeaddeaddead0001");
    env.block.timestamp = U256::from(1_629_814_800);
    env.block.gas_limit = U256::from(30_000_000);
    env.block.basefee = U256::from(1);
    env.block.difficulty = U256::from(1);

    env.cfg.chain_id = 10;
    env.cfg.spec_id = SpecId::CANYON;
    env.cfg.optimism = true;

    env.tx.caller = caller;
    env.tx.gas_limit = 30000000;
    env.tx.gas_price = U256::from(50);

    env.tx.transact_to = TransactTo::call(precompile_addr);
    env.tx.value = U256::from(100);
    env.tx.data = Bytes::from_static(&[]);
    env.tx.nonce = Some(0);
    env.tx.chain_id = Some(10);
    env.tx.gas_priority_fee = Some(U256::from(1));

    env.tx.optimism = OptimismFields {
        source_hash: None,
        mint: Some(100_u128),
        is_system_transaction: Some(false),
        enveloped_tx: Some(Bytes::from_static(&[])),
    };

    // Insert the calling account into the database
    let mut db = InMemoryDB::default();
    db.insert_account_info(
        caller,
        AccountInfo {
            nonce: 0,
            balance: U256::from_str_radix("FFFFFFFFFFFF", 16).unwrap(),
            code_hash: b256!("0000000000000000000000000000000000000000000000000000000000000000"),
            code: None,
        },
    );
    let mut journal = JournaledState::new(SpecId::CANYON, vec![]);
    journal
        .initial_account_load(caller, &[U256::from(100)], &mut db)
        .unwrap();

    let mut inspector = TracerEip3155::new(Box::new(stdout()), true, true);

    let mut evm = Box::new(EVMImpl::<CanyonSpec, InMemoryDB>::new_with_spec(
        &mut db,
        &mut env,
        Some(&mut inspector),
        precompiles,
    ));

    // Preverify for sanity
    if let Err(e) = evm.preverify_transaction_inner() {
        println!("Preverification error: {:?}", e);
        return Err(anyhow::anyhow!("Preverification failed"));
    }
    println!("Preverification succeeded. Transacting...\n");

    // Transact
    let result = evm.transact();
    match result {
        Ok(r) => println!("\nTransact result\n{:?}", r),
        Err(e) => {
            println!("Transact error: {:?}", e);
            return Err(anyhow::anyhow!("Transact failed"));
        }
    }

    Ok(())
}
