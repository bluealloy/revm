//! MyEvm variant of the EVM that disables beneficiary reward.
//!
//! The most basic variant of the EVM that does not include helper trains can be
//! run as
//! ```rust
//! let mut my_evm = MyEvm::new(Context::mainnet(), ());
//! let _res = MyHandler::default().run(&mut my_evm);
//! ```
//!
//! This requires creating a [`MyEvm`] struct (found in [`example_my_evm::evm`]), impelementing the [`revm::handler::EvmTr`] trait and
//! adding [`MyHandler`] implementation).
//!
//! Main logic that is slightly changes from Ethereum can be found in [`MyHandler`].
//! [`MyHandler`] implemented the [`Handler`] trait overrides the default behaviour
//! of rewarding of beneficiary that is skipped.
//!
//! Adding two more traits. [`revm::inspector::InspectorEvmTr`] on [`MyEvm`] and
//! [`revm::inspector::InspectorHandler`] for [`MyHandler`] allow us to support
//! [`revm::Inspector`] without much of effort. With it we now inspect the code:
//!
//! ```rust
//! let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
//! let _res = MyHandler::default().inspect_run(&mut my_evm);
//! ```
//!
//! Other things in this repo are utilities to make it easier to consume [`MyEvm`]
//! and they can be found in [`example_my_evm::api`].
//!
//! * [`revm::ExecuteEvm`] allows us to hide MyHandler and all Context generics.
//!     It allows us to call MyEvm as .
//! ```rust
//!     // Evm Execute example
//!     let mut my_evm = MyEvm::new(Context::mainnet(), ());
//!     let _result_and_state = my_evm.transact(TxEnv::default());
//!     // or if you want to replay last tx.
//!     let _res_and_state = my_evm.replay();
//! ```
//! * [`revm::ExecuteCommitEvm`] extends [`revm::ExecuteEvm`] with `replay_commit` and `transact_commit` methods.
//! That apply the transaction change directly to the database. This requires Database to implement [`revm::DatabaseCommit`] trait.
//! ```rust
//!     let mut my_evm = MyEvm::new(Context::mainnet().with_db(InMemoryDB::default()), ());
//!     let _res = my_evm.transact_commit(TxEnv::default());
//! ```
//!
//! * [`revm::InspectEvm`] extends [`revm::ExecuteEvm`] with `inspect_replay` and `inspect_commit_previous` methods.
//!     It allows us to inspect the code without applying the transaction changes to the database.
//! ```rust
//!     let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
//!     let _res = my_evm.inspect_replay();
//! ```
//!
//!
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
