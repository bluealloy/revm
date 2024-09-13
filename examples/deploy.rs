use anyhow::{anyhow, bail};
use revm::{
    primitives::{bytes::Bytes, hex, EthereumWiring, ExecutionResult, Output, TxKind, U256},
    Evm, InMemoryDB,
};

/// [ PUSH1, 0x01, PUSH1, 0x17, PUSH1, 0x1f, CODECOPY, PUSH0, MLOAD, PUSH0, SSTORE ]
const INIT_CODE: &[u8] = &[
    0x60, 0x01, 0x60, 0x17, 0x60, 0x1f, 0x39, 0x5f, 0x51, 0x5f, 0x55,
];

/// [ PUSH1, 0x02, PUSH1, 0x15, PUSH0, CODECOPY, PUSH1, 0x02, PUSH0, RETURN]
const RET: &[u8] = &[0x60, 0x02, 0x60, 0x15, 0x5f, 0x39, 0x60, 0x02, 0x5f, 0xf3];

/// [ PUSH0 SLOAD ]
const RUNTIME_BYTECODE: &[u8] = &[0x5f, 0x54];

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
