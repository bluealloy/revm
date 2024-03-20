# State implementations

State inherits the `Database` trait and implements fetching of external state and storage, and various functionality on output of the EVM execution.
Most notably, caching changes while execution multiple transactions.

## Database Abstractions

You can implement the traits `Database`, `DatabaseRef` or `Database + DatabaseCommit` depending on the desired handling of the struct.

- `Database`:
  Has mutable `self` in its functions.
  It is useful if you want to modify your cache or update some statistics on `get` calls.
  This trait enables `preverify_transaction`, `transact_preverified`, `transact` and `inspect` functions.
- `DatabaseRef`:
  Takes a reference on the object.
  It is useful if you only have a reference on the state and don't want to update anything on it.
  It enables `previerify_transaction`, `transact_preverified_ref`, `transact_ref` and `inspect_ref` functions.
- `Database + DatabaseCommit`:
  Allows directly committing changes of a transaction.
  It enables `transact_commit` and `inspect_commit` functions.
