# Primitives

This crate is a core component of the Revm system. It is designed to provide definitions for a range of types and structures commonly used throughout the application. It is set up to be compatible with environments that do not include Rust's standard library, as indicated by the `no_std` attribute.
Here's a comprehensive breakdown of the modules included in the crate:

## `bits`

This module houses the definitions for fixed-size bit arrays, `B160` and `B256`, showcasing its role in managing bits-related operations, to represent 256-bit and 160-bit fixed-size hashes respectively. These are defined using the `construct_fixed_hash!` macro from the `fixed_hash` crate.

The `AsRef` and `Deref` traits from `derive_more` crate are derived for both of these structures, providing convenient methods for converting these types to and from references of their underlying data.

The `Arbitrary` trait from the `arbitrary` crate and the `PropTestArbitrary` trait from `proptest_derive` crate are derived conditionally when either testing or the "arbitrary" feature is enabled. These traits are used for property testing, a form of testing where random inputs are generated and used to validate certain properties of your code.

The code also provides conversions between `B256`, `B160` and various other types such as `u64`, `primitive_types::H256`, `primitive_types::H160`, `primitive_types::U256`, and `ruint::aliases::U256`. The `impl` From blocks specify how to convert from one type to another.

`impl_fixed_hash_conversions!` macro is used to define conversions between `B256` and `B160` types.

If the "serde" feature is enabled, the Serialize and Deserialize traits from the serde crate are implemented for `B256` and `B160` using a custom serialization method that outputs/reads these types as hexadecimal strings. This includes a custom serialization/deserialization module for handling hexadecimal data.

This module (serialize) provides functionality to serialize a slice of bytes to a hexadecimal string, and deserialize a hexadecimal string to a byte vector with size checks. It handles both "0x" prefixed and non-prefixed hexadecimal strings. It also provides functions to convert raw bytes to hexadecimal strings and vice versa, handling potential errors related to non-hexadecimal characters. The module also defines the `ExpectedLen` enum which is used to specify the expected length of the byte vectors during deserialization.

## `bytecode`

Entrusted with tasks related to EVM bytecode, handling operations like parsing, interpretation, and transformation.
This module defines structures and methods to manipulate Ethereum bytecode and manage its state. It's built around three main components: `JumpMap`, `BytecodeState`, and `Bytecode`.

The `JumpMap` structure stores a map of valid `jump` destinations within a given Ethereum bytecode sequence. It is essentially an `Arc` (Atomic Reference Counter) wrapping a `BitVec` (bit vector), which can be accessed and modified using the defined methods, such as `as_slice()`, `from_slice()`, and `is_valid()`.

The `BytecodeState` is an enumeration, capturing the three possible states of the bytecode: `Raw`, `Checked`, and `Analysed`. In the `Checked` and `Analysed` states, additional data is provided, such as the length of the bytecode and, in the `Analysed` state, a `JumpMap`.

The `Bytecode` struct holds the actual bytecode, its hash, and its current state (`BytecodeState`). It provides several methods to interact with the bytecode, such as getting the length of the bytecode, checking if it's empty, retrieving its state, and converting the bytecode to a checked state. It also provides methods to create new instances of the `Bytecode` struct in different states.

## `constants`

Holds constant values used throughout the system. This module defines important constants that help limit and manage resources in the Ethereum Virtual Machine (EVM). These limits are integral to maintaining the efficiency, security, and future extensibility of the EVM. The constants include `STACK_LIMIT` and `CALL_STACK_LIMIT`, which restrict the size of the interpreter stack and the EVM call stack, respectively. Both are set to 1024.

A vital constant established here is `MAX_CODE_SIZE`, which is set according to [EIP-170](https://eips.ethereum.org/EIPS/eip-170)'s specification. [EIP-170](https://eips.ethereum.org/EIPS/eip-170) imposes a maximum limit on the contract code size to mitigate potential vulnerabilities and inefficiencies in Ethereum. Without this cap, the act of calling a contract can trigger costly operations that scale with the size of the contract's code. These operations include reading the code from disk, preprocessing the code for VM execution, and adding data to the block's proof-of-validity. By implementing `MAX_CODE_SIZE` (set to `0x6000` or ~25kb), the EVM ensures that the cost of these operations remains manageable, even under high gas levels that could be encountered in the future. [EIP-170](https://eips.ethereum.org/EIPS/eip-170)'s implementation thus offers crucial protection against potential DoS attacks and maintains efficiency, especially for future light clients verifying proofs of validity or invalidity.

Another essential constant defined here is `MAX_INITCODE_SIZE`, set in accordance with [EIP-3860](https://eips.ethereum.org/EIPS/eip-3860). [EIP-3860](https://eips.ethereum.org/EIPS/eip-3860) extends EIP-170 by introducing a maximum size limit for initialization code (initcode) and enforcing a gas charge for every 32-byte chunk of initcode, to account for the cost of jump destination analysis. Before [EIP-3860](https://eips.ethereum.org/EIPS/eip-3860), initcode analysis during contract creation wasn't metered, nor was there an upper limit for its size, resulting in potential inefficiencies and vulnerabilities. By setting `MAX_INITCODE_SIZE` to 2 \* `MAX_CODE_SIZE` and introducing the said gas charge, [EIP-3860](https://eips.ethereum.org/EIPS/eip-3860) ensures that the cost of initcode analysis scales proportionately with its size. This constant, therefore, facilitates fair charging, simplifies EVM engines by setting explicit limits, and helps to create an extendable cost system for the future.

## `bd`

As its name suggests, it's responsible for database operations. This module is where the blockchain's state persistence is managed.
The module defines three primary traits (`Database`, `DatabaseCommit`, and `DatabaseRef`), a structure `RefDBWrapper`, and their associated methods.

The `Database` trait defines an interface for mutable interaction with the database. It has a generic associated type `Error` to handle different kinds of errors that might occur during these interactions. It provides methods to retrieve basic account information (`basic`), retrieve account code by its hash (`code_by_hash`), retrieve the storage value of an address at a certain index (`storage`), and retrieve the block hash for a certain block number (`block_hash`).

The `DatabaseCommit` trait defines a single `commit` method for committing changes to the database. The changes are a map between Ethereum-like addresses (type `B160`) and accounts.

The `DatabaseRef` trait is similar to the `Database` trait but is designed for read-only or immutable interactions. It has the same `Error` associated type and the same set of methods as `Database`, but these methods take `&self` instead of `&mut self`, indicating that they do not mutate the database.

The `RefDBWrapper` structure is a wrapper around a reference to a `DatabaseRef` type. It implements the `Database` trait, essentially providing a way to treat a `DatabaseRef` as a `Database` by forwarding the `Database` methods to the corresponding `DatabaseRef` methods.

## `env`

A significant module that manages the execution environment of the EVM. The module containts objects and methods associated with processing transactions and blocks within such a blockchain environment. It defines several structures: `Env`, `BlockEnv`, `TxEnv`, `CfgEnv`, `TransactTo`, and `CreateScheme`. These structures contain various fields representing the block data, transaction data, environmental configurations, transaction recipient details, and the method of contract creation respectively.

The `Env` structure, which encapsulates the environment of the EVM, contains methods for calculating effective gas prices and for validating block and transaction data. It also checks transactions against the current state of the associated account, which is necessary to validate the transaction's nonce and the account balance. Various Ethereum Improvement Proposals (EIPs) are also considered in these validations, such as [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) for the base fee, [EIP-3607](https://eips.ethereum.org/EIPS/eip-3607) for rejecting transactions from senders with deployed code, and [EIP-3298](https://eips.ethereum.org/EIPS/eip-3298) for disabling gas refunds. The code is structured to include optional features and to allow for changes in the EVM specifications.

## `log`

This piece of Rust code defines a structure called Log which represents an Ethereum log entry. These logs are integral parts of the Ethereum network and are typically produced by smart contracts during execution. Each Log has three components:

- `address`: This field represents the address of the log originator, typically the smart contract that generated the log. The `B160` data type signifies a 160-bit Ethereum address.

- `topics`: This field is a vector of `B256` type. In Ethereum, logs can have multiple '`topics`'. These are events that can be used to categorize and filter logs. The `B256` type denotes a 256-bit hash, which corresponds to the size of a topic in Ethereum.

- `data`: This is the actual data of the log entry. The Bytes type is a dynamically-sized byte array, and it can contain any arbitrary data. It contains additional information associated with the event logged by a smart contract.

## `precompile`:

This module implements precompiled contracts in the EVM, adding a layer of pre-set functionalities. These are documented in the prior section. The module defines the types and the enum that are used to handle precompiled contracts.

`PrecompileResult`: This is a type alias for a `Result` type. The `Ok` variant of this type contains a tuple (`u64`, `Vec<u8>`), where the `u64` integer likely represents gas used by the precompiled contract, and the `Vec<u8>` holds the output data. The Err variant contains a PrecompileError.

`StandardPrecompileFn` and `CustomPrecompileFn`: These are type aliases for function pointers. Both functions take a byte slice and a `u64` (probably the available gas) as arguments and return a `PrecompileResult`. The naming suggests that the former refers to built-in precompiled contracts, while the latter may refer to custom, user-defined contracts.

`PrecompileError`: This is an enumeration (enum) which describes the different types of errors that could occur while executing a precompiled contract. The listed variants suggest these errors are related to gas consumption, `Blake2` hash function, modular exponentiation ("`Modexp`"), and `Bn128`, which is a specific elliptic curve used in cryptography.

## `result`

At the core of this module is the `ExecutionResult` enum, which describes the possible outcomes of an EVM execution: `Success`, `Revert`, and `Halt`. `Success` represents a successful transaction execution, and it holds important information such as the reason for `success` (an Eval enum), the gas used, the gas refunded, a vector of logs (`Vec<Log>`), and the output of the execution. This aligns with the stipulation in [EIP-658](https://eips.ethereum.org/EIPS/eip-658) that introduces a status code in the receipt of a transaction, indicating whether the top-level call was successful or failed.

`Revert` represents a transaction that was reverted by the `REVERT` opcode without spending all of its gas. It stores the gas used and the output. `Halt` represents a transaction that was reverted for various reasons and consumed all its gas. It stores the reason for halting (a `Halt` enum) and the gas used.

The `ExecutionResult` enum provides several methods to extract important data from an execution result, such as `is_success()`, `logs()`, `output()`, `into_output()`, `into_logs()`, and `gas_used()`. These methods facilitate accessing key details of a transaction execution.

The `EVMError` and `InvalidTransaction` enums handle different kinds of errors that can occur in an EVM, including database errors, errors specific to the transaction itself, and errors that occur due to issues with gas, among others.

The `Output` enum handles different kinds of outputs of an EVM execution, including `Call` and `Create`. This is where the output data from a successful execution or a reverted transaction is stored.

## `specification`

Holds data related to Ethereum's technical specifications, serving as a reference point for Ethereum's rules and procedures obtained from the [Ethereum execution specifications](https://github.com/ethereum/execution-specs). The module is primarily used to enumerate and handle Ethereum's network upgrades or "hard forks" within the Ethereum Virtual Machine (EVM). These hard forks are referred to as `SpecId` in the code, representing different phases of Ethereum's development.

The `SpecId` enum assigns a unique numerical value and a unique string identifier to each Ethereum hard fork. These upgrades range from the earliest ones such as `FRONTIER` and `HOMESTEAD`, through to the most recent ones, including `LONDON`, `MERGE`, `SHANGHAI`, and `LATEST`.

The code also includes conversion methods such as `try_from_u8()` and `from()`. The former attempts to create a `SpecId` from a given u8 integer, while the latter creates a `SpecId` based on a string representing the name of the hard fork.

The `enabled()` method in `SpecId` is used to check if one spec is enabled on another, considering the order in which the hard forks were enacted.

The `Spec` trait is used to abstract the process of checking whether a given spec is enabled. It only has one method, `enabled()`, and a constant `SPEC_ID`.

The module then defines various `Spec` structs, each representing a different hard fork. These structs implement the `Spec` trait and each struct's `SPEC_ID` corresponds to the correct `SpecId` variant.

This module provides the necessary framework to handle and interact with the different Ethereum hard forks within the EVM, making it possible to handle transactions and contracts differently depending on which hard fork rules apply. It also simplifies the process of adapting to future hard forks by creating a new `SpecId` and corresponding `Spec` struct.

## `state`

Manages the EVM's state, including account balances, contract storage, and more.

This module models an Ethereum account and its state, which includes balance, nonce, code, storage, and status flags. The module also includes methods for interacting with the account's state.

The `Account` struct includes fields for info (of type `AccountInfo`), storage (a `HashMap` mapping a `U256` value to a `StorageSlot`), and status (of type `AccountStatus`). `AccountInfo` represents the basic information about an Ethereum account, including its balance (`balance`), nonce (`nonce`), code (`code`), and a hash of its code (`code_hash`).

The `AccountStatus` is a set of bitflags, representing the state of the account. The flags include `Loaded`, `Created`, `SelfDestructed`, `Touched`, and `LoadedAsNotExisting`. The different methods provided within the `Account` struct allow for manipulating these statuses.

The `StorageSlot` struct represents a storage slot in the Ethereum Virtual Machine. It holds an `original_value` and a `present_value` and includes methods for creating a new slot and checking if the slot's value has been modified.

Two `HashMap` type aliases are created: `State` and `Storage`. `State` maps from a `B160` address to an `Account` and `Storage` maps from a `U256` key to a `StorageSlot`.

The module includes a series of methods implemented for `Account` to manipulate and query the account's status. These include methods like `mark_selfdestruct`, `unmark_selfdestruct`, `is_selfdestructed`, `mark_touch`, `unmark_touch`, `is_touched`, `mark_created`, `is_newly_created`, `is_empty`, and `new_not_existing`.

## `utilities`

A versatile module, it's a repository of utility functions and types for broader uses that don't fit into the other specific modules.

In addition to these, the Primitives crate employs the alloc crate, providing memory allocation functionality even in no-std environments. Furthermore, it re-exports several types from various crates such as bitvec, hashbrown, ruint, among others.

The crate prominently defines two type aliases: Address as B160 and Hash as B256. The Address is a 160-bit (20-byte) value, perfectly sized for an Ethereum address, and the Hash is a 256-bit value, corresponding to the output size of the Keccak-256 hash function, which is a staple in Ethereum.
