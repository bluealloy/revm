# database

As its name suggests, it's responsible for database operations. This module is where the blockchain's state persistence is managed.
The module defines three primary traits (`Database`, `DatabaseCommit`, and `DatabaseRef`), a structure `RefDBWrapper`, and their associated methods.

The `Database` trait defines an interface for mutable interaction with the database. It has a generic associated type `Error` to handle different kinds of errors that might occur during these interactions. It provides methods to retrieve basic account information (`basic`), retrieve account code by its hash (`code_by_hash`), retrieve the storage value of an address at a certain index (`storage`), and retrieve the block hash for a certain block number (`block_hash`).

The `DatabaseCommit` trait defines a single `commit` method for committing changes to the database. The changes are a map between Ethereum-like addresses (type `B160`) and accounts.

The `DatabaseRef` trait is similar to the `Database` trait but is designed for read-only or immutable interactions. It has the same `Error` associated type and the same set of methods as `Database`, but these methods take `&self` instead of `&mut self`, indicating that they do not mutate the database.

The `RefDBWrapper` structure is a wrapper around a reference to a `DatabaseRef` type. It implements the `Database` trait, essentially providing a way to treat a `DatabaseRef` as a `Database` by forwarding the `Database` methods to the corresponding `DatabaseRef` methods.
