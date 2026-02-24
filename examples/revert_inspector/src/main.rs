//! Revert inspector example.
//!
//! This example demonstrates how to handle a revert from a contract call and how to decode it.
//! Reference:
//!     1. https://docs.soliditylang.org/en/latest/internals/layout_in_calldata.html
//!     2. https://docs.soliditylang.org/en/latest/abi-spec.html#abi

use alloy_sol_types::SolValue;
use anyhow::bail;
use revm::{
    bytecode::opcode,
    context::{result::ExecutionResult, TxEnv},
    database::{CacheDB, EmptyDB},
    primitives::{address, hex, keccak256, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

#[rustfmt::skip]
/// Bytecode associared with a contract that alway revert when called. The bytecode is composed
/// as follow:
///     (0 - 4) | (5 - 36) | (37 - 68) | (69 - 100)
///        |         |           |           |
///        |         |           |           ---  "I revert"
///        |         |           ---  string length
///        |         ---  string encoding start
///        ---  error function selector
const REVERT_CODE: &[u8] = &[
    // Save in memory the function selector for the revert errror.
    // Since MSTORE always push a word, and left pad with zeros, we manually
    // insert all the 32 bytes.
    opcode::PUSH32,
    0x08, 0xc3, 0x79, 0xa0, // keccak256("Error(string)")[0..4]
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    opcode::PUSH0,
    opcode::MSTORE,
    // Save in memory the pointer to the start of the string.
    opcode::PUSH1, 0x20,
    opcode::PUSH1, 0x04,
    opcode::MSTORE,
    // Save in memory the length of the string.
    opcode::PUSH1, 0x08,
    opcode::PUSH1, 0x24,
    opcode::MSTORE,
    // Save in memory the error string value.
    opcode::PUSH32,
    0x49, 0x20, 0x72, 0x65, 0x76, 0x65, 0x72, 0x74, // b"I revert"
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    opcode::PUSH1, 0x44,
    opcode::MSTORE,
    // Trigger the revert.
    opcode::PUSH1, 0x64,
    opcode::PUSH0,
    opcode::REVERT,
];

/// Decode the revert data uisng the abi decoding.
fn decode_revert_output_with_abi(data: &Bytes) -> Option<String> {
    let function_selector = &keccak256("Error(string)")[0..4];

    if data.len() < 4 || &data[0..4] != function_selector {
        return None;
    }

    <String as SolValue>::abi_decode(&data[4..]).ok()
}

/// Decode the revert data manually.
fn decode_revert_output_manually(data: &Bytes) -> Option<String> {
    // Ensure data has the expected length given by the function selector + error string encoding.
    if data.len() != 100 {
        return None;
    }

    let function_selector = &keccak256("Error(string)")[0..4];
    if &data[0..4] != function_selector {
        return None;
    }

    let offset_bytes = &data[4..36];
    let offset = usize::try_from(U256::from_be_slice(offset_bytes)).unwrap();

    let length_start = 4 + offset;
    let length_end = length_start + 32;
    let length = usize::try_from(U256::from_be_slice(&data[length_start..length_end])).unwrap();

    let string_data = &data[length_end..length_end + length];

    String::from_utf8(string_data.to_vec()).ok()
}

/// Entrypoint for the revert inspector example.
pub fn main() -> anyhow::Result<()> {
    println!("=========================");
    println!("Revert Inspector Exampple");
    println!("=========================\n");

    let contract_address = address!("1000000000000000000000000000000000000001");

    let code_hash = keccak256(REVERT_CODE);

    let mut db = CacheDB::<EmptyDB>::default();
    // Instead of going through the contract deployment flow, we just insert the
    // bytedcode at the selected address.
    db.insert_account_info(
        contract_address,
        AccountInfo {
            code: Some(Bytecode::new_legacy(REVERT_CODE.into())),
            code_hash,
            ..Default::default()
        },
    );

    println!(
        "Bytecode added to the account with address: 0x{}",
        hex::encode(contract_address)
    );

    let mut evm = Context::mainnet().with_db(db).build_mainnet();
    // Directly call into the contract form the EVM.
    let result_and_state = evm.transact(
        TxEnv::builder()
            .kind(TxKind::Call(contract_address))
            .build()
            .unwrap(),
    )?;
    println!("Transaction executed.");

    // Handle the result of the contract call. We alway expected a revert here.
    match result_and_state.result {
        ExecutionResult::Revert { output, .. } => {
            println!("Transaction reverted as expected!");
            println!("Output raw bytes: 0x{}", hex::encode(output.clone()));

            if let Some(reason) = decode_revert_output_manually(&output) {
                println!("Revert reason (manual decoding): {reason}");
            } else {
                bail!("Expected a revert reason");
            }

            if let Some(reason) = decode_revert_output_with_abi(&output) {
                println!("Revert reason (abi decoding): {reason}");
            } else {
                bail!("Expected a revert reason");
            }
        }
        _ => bail!("Expected a revert"),
    }

    Ok(())
}
