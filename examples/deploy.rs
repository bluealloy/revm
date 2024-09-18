use anyhow::{anyhow, bail};
use revm::{
    interpreter::opcode,
    primitives::{bytes::Bytes, hex, EthereumWiring, ExecutionResult, Output, TxKind, U256},
    Evm, InMemoryDB,
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
    let mut evm: Evm<'_, EthereumWiring<InMemoryDB, ()>> =
        Evm::<EthereumWiring<InMemoryDB, ()>>::builder()
            .with_default_db()
            .with_default_ext_ctx()
            .modify_tx_env(|tx| {
                tx.transact_to = TxKind::Create;
                *tx.data = bytecode.clone();
            })
            .build();

    println!("bytecode: {}", hex::encode(bytecode));
    let ref_tx = evm.transact_commit()?;
    let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = ref_tx
    else {
        bail!("Failed to create contract: {ref_tx:#?}");
    };

    println!("Created contract at {address}");
    evm = evm
        .modify()
        .modify_tx_env(|tx| {
            tx.transact_to = TxKind::Call(address);
            *tx.data = Default::default();
            tx.nonce += 1;
        })
        .build();

    let result = evm.transact()?;
    let Some(storage0) = result
        .state
        .get(&address)
        .ok_or_else(|| anyhow!("Contract not found"))?
        .storage
        .get::<U256>(&Default::default())
    else {
        bail!("Failed to write storage in the init code: {result:#?}");
    };

    println!("storage U256(0) at{address}:  {storage0:#?}");
    assert_eq!(storage0.present_value(), param.try_into()?, "{result:#?}");
    Ok(())
}
