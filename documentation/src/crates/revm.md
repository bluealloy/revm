# Rust Ethereum Virtual Machine (revm)

The `crate` is focused on the implementation of Ethereum Virtual Machine (EVM) including database handling, state journaling, and an inspection system for observing and logging the execution of EVM. This crate pulls together everything described prior to deliver the rust evm.

Modules:

- [db](#): This module includes structures and functions for database interaction.
- [evm](#): This module is concerned with the Ethereum Virtual Machine (EVM) implementation.
- [evm_impl](#): This module likely includes more specific or complex implementations related to the EVM.
- [inspector](#): This module introduces the `Inspector` trait and its implementations for observing the EVM execution.
- [journaled_state](#): This module manages the state of the EVM and implements a journaling system to handle changes and reverts.

External Crates:

- alloc: The alloc crate is used to provide the ability to allocate memory on the heap. It's a part of Rust's standard library that can be used in environments without a full host OS.

Constants:

- USE_GAS: This constant determines whether gas measurement should be used. It's set to false if the no_gas_measuring feature is enabled.

Re-exported Crates:

- revm_precompile: This crate is re-exported, likely providing the precompiled contracts used in the EVM implementation.
- revm_interpreter: This crate is re-exported, providing the execution engine for EVM opcodes.
- revm_interpreter::primitives: This module from the `revm_interpreter` crate is re-exported, providing primitive types or functionality used in the EVM implementation.

Re-exported Types:

- Database, DatabaseCommit, InMemoryDB: These types from the `db` module are re-exported for handling the database operations.
- EVM: The EVM struct from the `evm` module is re-exported, serving as the main interface to the EVM implementation.
- EVMData: The EVMData struct from the `evm_impl` module is re-exported, likely providing data structures to encapsulate EVM execution data.
- JournalEntry, JournaledState: These types from the `journaled_state` module are re-exported, providing the journaling system for the EVM state.
- inspectors, Inspector: The `Inspector` trait and its implementations from the `inspector` module are re-exported for observing the EVM execution.
