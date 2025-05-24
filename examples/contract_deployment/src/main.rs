//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::{anyhow, bail};
use revm::{
    bytecode::opcode,
    context::{Context, TxEnv},
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    primitives::{hex, Bytes, StorageValue, TxKind},
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext,
};

/// Load number parameter and set to storage with slot 0
const INIT_CODE: &[u8] = &[
    opcode::PUSH1,
    0x01,
    opcode::PUSH1,
    0x17,
    opcode::PUSH1,
    0x1f,
    opcode::CODECOPY,
    opcode::PUSH0,
    opcode::MLOAD,
    opcode::PUSH0,
    opcode::SSTORE,
];

/// Copy runtime bytecode to memory and return
const RET: &[u8] = &[
    opcode::PUSH1,
    0x02,
    opcode::PUSH1,
    0x15,
    opcode::PUSH0,
    opcode::CODECOPY,
    opcode::PUSH1,
    0x02,
    opcode::PUSH0,
    opcode::RETURN,
];

/// Load storage from slot zero to memory
const RUNTIME_BYTECODE: &[u8] = &[opcode::PUSH0, opcode::SLOAD];

fn main() -> anyhow::Result<()> {
    let param = 0x42;
    let bytecode: Bytes = [INIT_CODE, RET, RUNTIME_BYTECODE, &[param]].concat().into();
    let ctx = Context::mainnet().with_db(CacheDB::<EmptyDB>::default());

    let mut evm = ctx.build_mainnet();

    println!("bytecode: {}", hex::encode(&bytecode));
    let ref_tx = evm.transact_commit(TxEnv {
        kind: TxKind::Create,
        data: bytecode.clone(),
        ..Default::default()
    })?;
    let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = ref_tx
    else {
        bail!("Failed to create contract: {ref_tx:#?}");
    };

    println!("Created contract at {address}");
    let output = evm.transact_finalize(TxEnv {
        kind: TxKind::Call(address),
        data: Default::default(),
        nonce: 1,
        ..Default::default()
    })?;
    let Some(storage0) = output
        .state
        .get(&address)
        .ok_or_else(|| anyhow!("Contract not found"))?
        .storage
        .get::<StorageValue>(&Default::default())
    else {
        bail!(
            "Failed to write storage in the init code: {:#?}",
            output.result
        );
    };

    println!("storage U256(0) at {address}:  {storage0:#?}");
    assert_eq!(
        storage0.present_value(),
        param.try_into()?,
        "{:#?}",
        output.result
    );
    Ok(())
}
