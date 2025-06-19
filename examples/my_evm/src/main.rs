//! Example of a custom EVM variant.
#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use example_my_evm::{evm::MyEvm, handler::MyHandler};
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    handler::{ExecuteCommitEvm, ExecuteEvm, Handler},
    inspector::InspectorHandler,
    Context, MainContext,
};

/// Example demonstrating various ways to use a custom EVM implementation.
///
/// This function showcases different usage patterns for MyEvm:
/// 1. Basic transaction execution without inspection
/// 2. Transaction execution with inspector support for debugging
/// 3. Single transaction execution with state finalization
/// 4. Transaction execution with automatic state commitment to the database
///
/// Each example demonstrates a different aspect of EVM customization and usage,
/// from simple execution to more complex patterns involving state management
/// and transaction inspection.
pub fn main() {
    // transact example
    let mut my_evm = MyEvm::new(Context::mainnet(), ());
    let _res = MyHandler::default().run(&mut my_evm);

    // inspector example
    let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
    let _res = MyHandler::default().inspect_run(&mut my_evm);

    // Evm Execute example
    let mut my_evm = MyEvm::new(Context::mainnet(), ());
    let _result = my_evm.transact_one(TxEnv::default());
    // or if you want to obtain the state by finalizing execution
    let _state = my_evm.finalize();

    // Evm Execute Commit example
    let mut my_evm = MyEvm::new(Context::mainnet().with_db(InMemoryDB::default()), ());
    let _res = my_evm.transact_commit(TxEnv::default());
}
