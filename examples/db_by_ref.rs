use revm::{
    db::{CacheDB, EmptyDB, WrapDatabaseRef},
    handler::register::HandleRegister,
    inspector_handle_register,
    inspectors::TracerEip3155,
    primitives::ResultAndState,
    Database, DatabaseCommit, Evm,
};

struct DebugContext<'evm, EXT, DB: Database> {
    ext: EXT,
    register_handles_fn: HandleRegister<'evm, EXT, DB>,
}

fn run_transaction<'db, 'evm, EXT>(
    db: &'db CacheDB<EmptyDB>,
    ext: EXT,
    register_handles_fn: HandleRegister<'evm, EXT, WrapDatabaseRef<&'evm CacheDB<EmptyDB>>>,
) -> anyhow::Result<ResultAndState>
where
    'db: 'evm,
{
    let mut evm = Evm::builder()
        .with_ref_db(db)
        .with_external_context(ext)
        .append_handler_register(register_handles_fn)
        .build();

    let result = evm.transact()?;

    Ok(result)
}

fn run_transaction_and_commit_with_ext<'db, 'evm, EXT>(
    db: &'db mut CacheDB<EmptyDB>,
    ext: EXT,
    register_handles_fn: HandleRegister<'evm, EXT, WrapDatabaseRef<&'evm CacheDB<EmptyDB>>>,
) -> anyhow::Result<()>
where
    'db: 'evm,
{
    let ResultAndState { state: changes, .. } = {
        let db: &'evm _ = &*db;
        run_transaction(db, ext, register_handles_fn)?
    };

    // Compile error: error[E0502]: cannot borrow `*db` as mutable because it is also borrowed as immutable
    // The lifetime of `'evm` is extended beyond this function's scope because it is used in the `HandleRegister` function
    db.commit(changes);

    Ok(())
}

fn run_transaction_and_commit<'db>(db: &mut CacheDB<EmptyDB>) -> anyhow::Result<()> {
    let mut evm = Evm::builder().with_ref_db(db).build();

    let ResultAndState { state: changes, .. } = evm.transact()?;

    // No compiler error because there is no lifetime parameter for the `HandleRegister` function
    db.commit(changes);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut cache_db = CacheDB::new(EmptyDB::default());

    let mut tracer = TracerEip3155::new(Box::new(std::io::stdout()), true, true);

    run_transaction_and_commit_with_ext(&mut cache_db, &mut tracer, inspector_handle_register)?;
    run_transaction_and_commit(&mut cache_db)?;

    Ok(())
}
