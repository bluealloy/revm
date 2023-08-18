# state

Manages the EVM's state, including account balances, contract storage, and more.

This module models an Ethereum account and its state, which includes balance, nonce, code, storage, and status flags. The module also includes methods for interacting with the account's state.

The `Account` struct includes fields for info (of type `AccountInfo`), storage (a `HashMap` mapping a `U256` value to a `StorageSlot`), and status (of type `AccountStatus`). `AccountInfo` represents the basic information about an Ethereum account, including its balance (`balance`), nonce (`nonce`), code (`code`), and a hash of its code (`code_hash`).

The `AccountStatus` is a set of bitflags, representing the state of the account. The flags include `Loaded`, `Created`, `SelfDestructed`, `Touched`, and `LoadedAsNotExisting`. The different methods provided within the `Account` struct allow for manipulating these statuses.

The `StorageSlot` struct represents a storage slot in the Ethereum Virtual Machine. It holds an `original_value` and a `present_value` and includes methods for creating a new slot and checking if the slot's value has been modified.

Two `HashMap` type aliases are created: `State` and `Storage`. `State` maps from a `Address` address to an `Account` and `Storage` maps from a `U256` key to a `StorageSlot`.

The module includes a series of methods implemented for `Account` to manipulate and query the account's status. These include methods like `mark_selfdestruct`, `unmark_selfdestruct`, `is_selfdestructed`, `mark_touch`, `unmark_touch`, `is_touched`, `mark_created`, `is_newly_created`, `is_empty`, and `new_not_existing`.
