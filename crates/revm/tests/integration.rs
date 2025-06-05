//! Integration tests for the `op-revm` crate.
mod common;

use alloy_primitives::KECCAK256_EMPTY;
use common::compare_or_save_testdata;
use context::ContextTr;
use primitives::{b256, hardfork::SpecId};
use revm::{
    bytecode::opcode,
    context::TxEnv,
    database::{BenchmarkDB, BENCH_TARGET},
    primitives::U256,
    state::Bytecode,
    Context, ExecuteEvm, MainBuilder, MainContext,
};

#[test]
fn test_multi_tx() {
    let mut evm = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::BERLIN)
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(
            [
                opcode::PUSH2,
                0xFF,
                0xFF,
                opcode::SELFDESTRUCT,
                opcode::STOP,
            ]
            .into(),
        )))
        .build_mainnet();

    // trigger selfdestruct
    let result1 = evm
        .transact(TxEnv::builder_for_bench().build_fill())
        .unwrap();

    let destroyed_acc = evm.ctx.journal_mut().state.get_mut(&BENCH_TARGET).unwrap();

    // balance got transferred to 0x0000..00FFFF
    assert_eq!(destroyed_acc.info.balance, U256::ZERO);
    assert_eq!(destroyed_acc.info.nonce, 1);
    assert_eq!(
        destroyed_acc.info.code_hash,
        b256!("0x9125466aa9ef15459d85e7318f6d3bdc5f6978c0565bee37a8e768d7c202a67a")
    );

    // call on destroyed account. This accounts gets loaded and should contain empty code_hash afterwards.
    let result2 = evm
        .transact(TxEnv::builder_for_bench().nonce(1).build_fill())
        .unwrap();

    let destroyed_acc = evm.ctx.journal_mut().state.get_mut(&BENCH_TARGET).unwrap();

    assert_eq!(destroyed_acc.info.code_hash, KECCAK256_EMPTY);
    assert_eq!(destroyed_acc.info.nonce, 0);
    assert_eq!(destroyed_acc.info.code, Some(Bytecode::default()));

    let output = evm.finalize();

    compare_or_save_testdata("test_multi_tx.json", (result1, result2, output));
}
