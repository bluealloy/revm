//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use core::error::Error;
use core::fmt::Debug;
use database::CacheDB;
use inspector::{
    inspector_handle_register,
    inspectors::{NoOpInspector, TracerEip3155},
};
use revm::{
    database_interface::{EmptyDB, WrapDatabaseRef},
    handler::register::HandleRegister,
    wiring::{
        result::{HaltReason, ResultAndState},
        EthereumWiring,
    },
    DatabaseCommit, DatabaseRef, Evm,
};

trait DatabaseRefDebugError: DatabaseRef<Error = Self::DBError> {
    type DBError: std::fmt::Debug + Error + Send + Sync + 'static;
}

impl<DBError, DB> DatabaseRefDebugError for DB
where
    DB: DatabaseRef<Error = DBError>,
    DBError: std::fmt::Debug + Error + Send + Sync + 'static,
{
    type DBError = DBError;
}

fn run_transaction<EXT: Debug, DB: DatabaseRefDebugError>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EthereumWiring<WrapDatabaseRef<DB>, EXT>>,
) -> anyhow::Result<(ResultAndState<HaltReason>, DB)> {
    let mut evm = Evm::<EthereumWiring<_, _>>::builder()
        .with_db(WrapDatabaseRef(db))
        .with_external_context(ext)
        .append_handler_register(register_handles_fn)
        .build();

    let result = evm.transact()?;
    Ok((result, evm.into_context().evm.inner.db.0))
}

fn run_transaction_and_commit_with_ext<EXT: Debug, DB: DatabaseRefDebugError + DatabaseCommit>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EthereumWiring<WrapDatabaseRef<DB>, EXT>>,
) -> anyhow::Result<()> {
    // To circumvent borrow checker issues, we need to move the database into the
    // transaction and return it after the transaction is done.
    let (ResultAndState { state: changes, .. }, mut db) =
        { run_transaction(db, ext, register_handles_fn)? };

    db.commit(changes);

    Ok(())
}

fn run_transaction_and_commit(db: &mut CacheDB<EmptyDB>) -> anyhow::Result<()> {
    let ResultAndState { state: changes, .. } = {
        let rdb = &*db;

        let mut evm = Evm::<EthereumWiring<_, _>>::builder()
            .with_db(WrapDatabaseRef(rdb))
            .with_external_context(NoOpInspector)
            .append_handler_register(inspector_handle_register)
            .build();

        evm.transact()?
    };

    // No compiler error because there is no lifetime parameter for the `HandleRegister` function
    db.commit(changes);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut cache_db = CacheDB::new(EmptyDB::default());

    let mut tracer = TracerEip3155::new(Box::new(std::io::stdout()));

    run_transaction_and_commit_with_ext(&mut cache_db, &mut tracer, inspector_handle_register)?;
    run_transaction_and_commit(&mut cache_db)?;

    Ok(())
}
