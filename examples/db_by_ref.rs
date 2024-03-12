use std::convert::Infallible;

use revm::{
    db::{CacheDB, EmptyDB, WrapDatabaseRef},
    handler::register::{HandleRegisterBox, HandleRegisterFn},
    inspector_handle_register,
    inspectors::TracerEip3155,
    primitives::ResultAndState,
    Database, DatabaseCommit, DatabaseRef, Evm, InspectorHandleRegister,
};

struct DebugContext<EXT, DB: Database> {
    ext: EXT,
    register_handles_fn: HandleRegisterBox<EXT, DB>,
}

fn run_transaction<EXT>(
    db: &CacheDB<EmptyDB>,
    ext: EXT,
    register_handles_fn: HandleRegisterFn<
        EXT,
        WrapDatabaseRef<&dyn DatabaseRef<Error = Infallible>>,
    >,
) -> anyhow::Result<ResultAndState> {
    let mut evm = Evm::builder()
        .with_ref_db(db as &dyn DatabaseRef<Error = Infallible>)
        .with_external_context(ext)
        .append_handler_register(register_handles_fn)
        .build();

    let result = evm.transact()?;

    drop(evm);

    Ok(result)
}

fn run_transaction_and_commit_with_ext<'a, EXT>(
    db: &mut CacheDB<EmptyDB>,
    ext: EXT,
    register_handles_fn: HandleRegisterFn<
        EXT,
        WrapDatabaseRef<&dyn DatabaseRef<Error = Infallible>>,
    >,
) -> anyhow::Result<()> {
    let ResultAndState { state: changes, .. } = { run_transaction(db, ext, register_handles_fn)? };

    // Compile error: error[E0502]: cannot borrow `*db` as mutable because it is also borrowed as immutable
    // The lifetime of `'evm` is extended beyond this function's scope because it is used in the `HandleRegister` function
    db.commit(changes);

    Ok(())
}

fn run_transaction_and_commit<'db>(db: &mut CacheDB<EmptyDB>) -> anyhow::Result<()> {
    let ref_db = &*db;
    let mut evm = Evm::builder().with_ref_db(ref_db).build();

    let ResultAndState { state: changes, .. } = evm.transact()?;
    drop(evm);

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
