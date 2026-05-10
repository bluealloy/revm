use revm::{
    context::TxEnv,
    database::{CacheDB, EmptyDB},
    primitives::{Address, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
};

const CALLER: Address = Address::new([0x10; 20]);
const CONTRACT: Address = Address::new([0x20; 20]);

/// Runtime bytecode used by this example.
///
/// It exercises memory, storage, logs, and a nested call:
/// - `MSTORE` stores `0x2a` at memory offset `0`
/// - `SSTORE` writes storage slot `0`
/// - `SSTORE` writes storage slot `2`
/// - `SLOAD` reads storage slot `0`
/// - `LOG1` emits one event
/// - `CALL` invokes the `ecrecover` precompile at address `0x01`
const CONTRACT_BYTECODE: &str = concat!(
    "602a600052",
    "6001600055",
    "61beef600255",
    "60005450",
    "7f1111111111111111111111111111111111111111111111111111111111111111",
    "60206000a1",
    "6000600060006000600060006001612710f1",
    "00",
);

pub(crate) fn build_db() -> Result<CacheDB<EmptyDB>, hex::FromHexError> {
    let mut db = CacheDB::new(EmptyDB::new());
    db.insert_account_info(CALLER, AccountInfo::default().with_balance(U256::MAX));
    db.insert_account_info(
        CONTRACT,
        AccountInfo::default().with_code(Bytecode::new_legacy(Bytes::from(hex::decode(
            CONTRACT_BYTECODE,
        )?))),
    );

    Ok(db)
}

pub(crate) fn build_tx() -> TxEnv {
    TxEnv::builder()
        .caller(CALLER)
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(30_000_000)
        .build_fill()
}
