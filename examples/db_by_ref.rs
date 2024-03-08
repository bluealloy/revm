use revm::{
    db::{CacheDB, EmptyDB, WrapDatabaseRef},
    handler::register::HandleRegister,
    inspector_handle_register,
    inspectors::{NoOpInspector, TracerEip3155},
    primitives::ResultAndState,
    DatabaseCommit, DatabaseRef, Evm,
};
use std::error::Error;

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

fn run_transaction<EXT, DB: DatabaseRefDebugError>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EXT, WrapDatabaseRef<DB>>,
) -> anyhow::Result<(ResultAndState, DB)> {
    let mut evm = Evm::builder()
        .with_ref_db(db)
        .with_external_context(ext)
        .append_handler_register(register_handles_fn)
        .build();

    let result = evm.transact()?;
    Ok((result, evm.into_context().evm.db.0))
}

fn run_transaction_and_commit_with_ext<EXT, DB: DatabaseRefDebugError + DatabaseCommit>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EXT, WrapDatabaseRef<DB>>,
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

        let mut evm = Evm::builder()
            .with_ref_db(rdb)
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

    let mut tracer = TracerEip3155::new(Box::new(std::io::stdout()), true);

    run_transaction_and_commit_with_ext(&mut cache_db, &mut tracer, inspector_handle_register)?;
    run_transaction_and_commit(&mut cache_db)?;

    Ok(())
}
