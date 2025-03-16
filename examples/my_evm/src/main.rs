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

pub fn main() {
    // transact example
    let mut my_evm = MyEvm::new(Context::mainnet(), ());
    let _res = MyHandler::default().run(&mut my_evm);

    // inspector example
    let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
    let _res = MyHandler::default().inspect_run(&mut my_evm);

    // Evm Execute example
    let mut my_evm = MyEvm::new(Context::mainnet(), ());
    let _res_and_state = my_evm.transact(TxEnv::default());
    // or if you want to replay last tx.
    let _res_and_state = my_evm.replay();

    // Evm Execute Commit example
    let mut my_evm = MyEvm::new(Context::mainnet().with_db(InMemoryDB::default()), ());
    let _res = my_evm.transact_commit(TxEnv::default());
}
