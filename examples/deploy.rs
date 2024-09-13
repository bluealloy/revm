use anyhow::{anyhow, bail};
use revm::{
    primitives::{bytes::Bytes, hex, EthereumWiring, ExecutionResult, Output, TxKind},
    Evm, InMemoryDB,
};

/// [ PUSH1, 0x01, PUSH0, SSTORE ]
const INIT_CODE: &[u8] = &[0x60, 0x01, 0x5f, 0x55];

/// [ PUSH1, 0x02, PUSH1, 0x0e, PUSH0, CODECOPY, PUSH1, 0x02, PUSH0, RETURN]
const RET: &[u8] = &[0x60, 0x02, 0x60, 0x0e, 0x5f, 0x39, 0x60, 0x02, 0x5f, 0xf3];

/// [ PUSH0 SLOAD ]
const RUNTIME_BYTECODE: &[u8] = &[0x5f, 0x54];

/// []
const PARAMS: &[u8] = &hex!("");

fn main() -> anyhow::Result<()> {
    let bytecode: Bytes = [INIT_CODE, RET, RUNTIME_BYTECODE, PARAMS].concat().into();
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
        bail!("Failed to create contract");
    };

    println!("Created contract at {address}");
    evm = evm
        .modify()
        .modify_tx_env(|tx| {
            tx.transact_to = TxKind::Call(address.clone());
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
        .get(&Default::default())
    else {
        bail!("Failed to write storage in the init code");
    };

    println!("storage U256(0) at{address}:  {storage0:#?}");
    assert_eq!(storage0.present_value(), 1.try_into()?);
    Ok(())
}
