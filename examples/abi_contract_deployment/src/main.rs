//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::{anyhow, bail};
use database::InMemoryDB;
use revm::{
    bytecode::opcode,
    primitives::{hex, Bytes, TxKind, U256},
    wiring::{
        result::{ExecutionResult, Output},
        EthereumWiring,
    },
    Evm,
};
use serde_json::from_str;

fn main() -> anyhow::Result<()> {
    // Get contract bytecode
    let contract_bytecode = include_bytes!("./Counter.json");
    let contract_data: serde_json::Value = from_str(&String::from_utf8_lossy(contract_bytecode))?;
    let bytecode = hex::decode(contract_data["bytecode"]["object"].as_str().unwrap())?;

    // Instantiate EVM
    let mut evm: Evm<'_, EthereumWiring<InMemoryDB, ()>> =
        Evm::<EthereumWiring<InMemoryDB, ()>>::builder()
            .with_default_db()
            .with_default_ext_ctx()
            .modify_tx_env(|tx| {
                tx.transact_to = TxKind::Create;
                tx.data = Bytes::from(bytecode.clone());
            })
            .build();

    // 1. Deploy contract
    let deploy_tx = evm.transact_commit()?;
    let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = deploy_tx
    else {
        anyhow::bail!("Contract deployment failed: {deploy_tx:#?}");
    };

    println!("Deployed contract address: {address}");

    // 2. Interact with contract - increment `number`
    let increment_selector = &hex::decode(
        contract_data["methodIdentifiers"]["increment()"]
            .as_str()
            .unwrap(),
    )?;
    evm = evm
        .modify()
        .modify_tx_env(|tx| {
            tx.transact_to = TxKind::Call(address);
            tx.data = Bytes::from(increment_selector.to_vec());
            tx.nonce += 1;
        })
        .build();

    let increment_tx = evm.transact_commit()?;
    println!("Number increment transaction executed: {increment_tx:#?}");

    // 3. Interact with contract - get `number`
    let number_selector = &hex::decode(
        contract_data["methodIdentifiers"]["number()"]
            .as_str()
            .unwrap(),
    )?;

    evm = evm
        .modify()
        .modify_tx_env(|tx| {
            tx.transact_to = TxKind::Call(address);
            tx.data = Bytes::from(number_selector.to_vec());
            tx.nonce += 1;
        })
        .build();

    let number_tx = evm.transact_commit()?;
    if let ExecutionResult::Success {
        output: Output::Call(value),
        ..
    } = number_tx
    {
        println!("Actual 'number' value: {value}");
    } else {
        anyhow::bail!("Failed to get 'number' value: {number_tx:#?}");
    }

    // TODO: Implement setNumber function
    // 4. Interact with contract - set `number`

    Ok(())
}
