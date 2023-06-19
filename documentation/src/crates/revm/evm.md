# EVM Module Documentation

This document provides the documentation for the `EVM` module.

## Struct `EVM`

The primary struct in this module is `EVM`. The `EVM` struct takes a database and enables `transact` to update the state directly to the database. Additionally, it allows the user to set all environment parameters.

```rust
#[derive(Clone)]
pub struct EVM<DB> {
    pub env: Env,
    pub db: Option<DB>,
}
```

The parameters that can be set are divided between Config, Block, and Transaction (tx). For transacting on the EVM, you can call `transact_commit` that will automatically apply changes to the database.

## Database Abstractions

You can implement the traits Database, DatabaseRef or Database + DatabaseCommit depending on the desired handling of the struct.

- `Database`: Has mutable `self` in its functions. It's useful if you want to modify your cache or update some statistics on `get` calls. This trait enables `transact` and `inspect` functions.
- `DatabaseRef`: Takes a reference on the object, this is useful if you only have a reference on the state and don't want to update anything on it. It enables `transact_ref` and `inspect_ref` functions.
- `Database + DatabaseCommit`: Allows directly committing changes of a transaction. It enables `transact_commit` and `inspect_commit` functions.

## Functions

Below are the key functions provided by the `EVM` struct.

### `transact_commit()`

This function is used to execute a transaction and apply the result to the database.

```rust
pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
    let ResultAndState { result, state } = self.transact()?;
    self.db.as_mut().unwrap().commit(state);
    Ok(result)
}
```

### `inspect_commit()`

This function is used to inspect a transaction and commit changes to the database.

```rust
pub fn inspect_commit<INSP: Inspector<DB>>(
    &mut self,
    inspector: INSP,
) -> Result<ExecutionResult, EVMError<DB::Error>> {
    let ResultAndState { result, state } = self.inspect(inspector)?;
    self.db.as_mut().unwrap().commit(state);
    Ok(result)
}
```

### `transact()`

This function executes a transaction without writing to the DB and returns the change state.

```rust
pub fn transact(&mut self) -> EVMResult<DB::Error> {
    if let Some(db) = self.db.as_mut() {
        let mut noop = NoOpInspector {};
        let out = evm_inner::<DB, false>(&mut self.env, db, &mut noop).transact();
        out
    } else {
        panic!("Database needs to be set");
    }
}
```

### `inspect()`

This function executes a transaction with a given inspector, without writing to the DB, and returns the change state.

```rust
pub fn inspect<INSP: Inspector<DB>>(&mut
```
