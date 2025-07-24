//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// reexport dependencies
#[doc(inline)]
pub use bytecode;
#[doc(inline)]
pub use context;
#[doc(inline)]
pub use context_interface;
#[doc(inline)]
pub use database;
#[doc(inline)]
pub use database_interface;
#[doc(inline)]
pub use handler;
#[doc(inline)]
pub use inspector;
#[doc(inline)]
pub use interpreter;
#[doc(inline)]
pub use precompile;
#[doc(inline)]
pub use primitives;
#[doc(inline)]
pub use state;

// Export items.

pub use context::journal::{Journal, JournalEntry};
pub use context::Context;
pub use database_interface::{Database, DatabaseCommit, DatabaseRef};
pub use handler::{
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext, MainnetEvm, SystemCallCommitEvm,
    SystemCallEvm,
};
pub use inspector::{InspectCommitEvm, InspectEvm, Inspector};

/// a
#[no_mangle]
pub fn f() -> impl Sized {
    let mut evm = context::Context::mainnet()
        .with_db(database_interface::EmptyDB::default())
        .build_mainnet();
    evm.transact(Default::default()).unwrap();
    inspector::inspect_instructions(
        &mut evm.ctx,
        &mut evm.frame_stack.get().interpreter,
        &mut inspector::NoOpInspector as &mut dyn Inspector<_, _>,
        &evm.instruction.instruction_table,
    );
    evm
}
