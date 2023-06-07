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

The `JumpMap` structure represents a map of valid jump destinations. It contains a bit vector that represents the validity of each program counter (PC) value. It provides methods to access the raw `bytes` of the jump map, construct a jump map from raw `bytes`, and check if a given PC value is a valid jump destination.

The `BytecodeState` enumeration represents the different states of bytecode. It can be in a raw state, a checked state with a specified length, or an analyzed state with a length and a `JumpMap`. It is used to track the state of bytecode during processing.

The `Bytecode` structure represents bytecode data along with its hash and state. It contains the `bytecode` as a `Bytes` object, which is an immutable sequence of `bytes`. It also holds the hash of the bytecode and the current state of the bytecode. There are various methods provided to interact with the bytecode, such as creating a new Bytecode instance with a `STOP` opcode, creating a new raw bytecode with a given `Bytes` object and hash, creating a new checked bytecode with a specified length and optional hash, accessing the raw `bytes` of the bytecode, obtaining the original `bytes` without any modifications, retrieving the bytecode hash, checking if the bytecode is empty, and getting the length of the bytecode. Additionally, there is a to_checked method that converts the bytecode to a checked state by appending zero `bytes` to the bytecode.

The code also includes some conditional attributes `(cfg_attr)` for `serde` serialization and deserialization, which enable serialization support for the `JumpMap`, `BytecodeState`, and `Bytecode` types when the "serde" feature is enabled.

The code relies on other external crates such as `fixed_hash`, `bitvec`, `bytes`, and `alloc` to define the necessary types and provide functionality for working with byte sequences, bit vectors, and memory allocation in a `no_std` environment. It also uses the keccak256 function from the crate's own module to compute the hash of the bytecode.

## `constants`

Holds constant values used throughout the system. It's a go-to module for all the static values used in the application.

## `db`

As its name suggests, it's responsible for database operations. This module is where the blockchain's state persistence is managed.

## `env`

A significant module that manages the execution environment of the EVM.

## `log`

A dedicated module for logging operations, ensuring seamless monitoring and debugging.

## `precompile`:

This module implements precompiled contracts in the EVM, adding a layer of pre-set functionalities. These are documented in the prior section.

## `result`

It defines types that represent the results of computations and manages errors that occur during operations.

## `specification`

Holds data related to Ethereum's technical specifications, serving as a reference point for Ethereum's rules and procedures.

## `state`

Manages the EVM's state, including account balances, contract storage, and more.

## `utilities`

A versatile module, it's a repository of utility functions and types for broader uses that don't fit into the other specific modules.

In addition to these, the Primitives crate employs the alloc crate, providing memory allocation functionality even in no-std environments. Furthermore, it re-exports several types from various crates such as bitvec, hashbrown, ruint, among others.

The crate prominently defines two type aliases: Address as B160 and Hash as B256. The Address is a 160-bit (20-byte) value, perfectly sized for an Ethereum address, and the Hash is a 256-bit value, corresponding to the output size of the Keccak-256 hash function, which is a staple in Ethereum.
