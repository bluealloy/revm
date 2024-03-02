use std::{convert::Infallible, error::Error};

use revm::{
    db::{CacheDB, EmptyDB, WrapDatabaseRef},
    handler::register::HandleRegister,
    inspector_handle_register,
    inspectors::{NoOpInspector, TracerEip3155},
    primitives::ResultAndState,
    DatabaseCommit, DatabaseRef, Evm,
};

struct DebugContext<EXT, DB: DatabaseRef> {
    ext: EXT,
    register_handles_fn: HandleRegister<EXT, WrapDatabaseRef<DB>>,
}

fn run_transaction<'a, 'w, EXT, DB: DatabaseRef>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EXT, WrapDatabaseRef<DB>>,
) -> anyhow::Result<(ResultAndState, DB)>
where
    <DB as DatabaseRef>::Error: std::fmt::Debug + Error + Send + Sync + 'static,
{
    let mut evm = Evm::builder()
        .with_ref_db(db)
        .with_external_context(ext)
        .append_handler_register(register_handles_fn)
        .build();

    let result = evm.transact()?;
    Ok((result, evm.into_context().evm.db.0))
}

fn run_transaction_and_commit_with_ext<'a, EXT, DB: DatabaseRef + DatabaseCommit>(
    db: DB,
    ext: EXT,
    register_handles_fn: HandleRegister<EXT, WrapDatabaseRef<DB>>,
) -> anyhow::Result<()>
where
    <DB as DatabaseRef>::Error: std::fmt::Debug + Error + Send + Sync + 'static,
{
    let (ResultAndState { state: changes, .. }, mut db) =
        { run_transaction(db, ext, register_handles_fn)? };

    db.commit(changes);

    Ok(())
}

fn run_transaction_and_commit<'db>(db: &mut CacheDB<EmptyDB>) -> anyhow::Result<()> {
    let rdb = &*db;
    let mut evm = Evm::builder()
        .with_ref_db(rdb)
        .with_external_context(NoOpInspector)
        .append_handler_register(inspector_handle_register)
        .build();

    let ResultAndState { state: changes, .. } = evm.transact()?;
    drop(evm);
    // No compiler error because there is no lifetime parameter for the `HandleRegister` function
    db.commit(changes);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut cache_db = CacheDB::new(EmptyDB::default());

    let mut tracer = TracerEip3155::new(Box::new(std::io::stdout()), true, true);

    //let db = WrapDatabaseRef(&cache_db);
    run_transaction_and_commit_with_ext(&mut cache_db, &mut tracer, inspector_handle_register)?;
    run_transaction_and_commit(&mut cache_db)?;

    Ok(())
}
