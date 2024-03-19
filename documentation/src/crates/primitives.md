# Primitives

This crate is a core component of the revm system.
It is designed to provide definitions for a range of types and structures commonly used throughout the application.
It is set up to be compatible with environments that do not include Rust's standard library, as indicated by the `no_std` attribute.

### Modules:

- [bits](./primitives/bits.md): This module provides types for handling specific sizes of byte arrays (Address and B256).
- [bytecode](./primitives/bytecode.md): This module provides functionality related to EVM bytecode.
- [constants](./primitives/constants.md): This module contains constant values used throughout the EVM implementation.
- [db](./primitives/database.md): This module contains data structures and functions related to the EVM's database implementation.
- [env](./primitives/environment.md): This module contains types and functions related to the EVM's environment, including block headers, and environment values.
- [log](./primitives/log.md): This module provides types and functionality for Ethereum logs.
- [precompile](./primitives/precompile.md): This module contains types related to Ethereum's precompiled contracts.
- [result](./primitives/result.md): This module provides types for representing execution results and errors in the EVM.
- [specification](./primitives/specifications.md): This module defines types related to Ethereum specifications (also known as hard forks).
- [state](./primitives/state.md): This module provides types and functions for managing Ethereum state, including accounts and storage.
- [utilities](./primitives/utils.md): This module provides utility functions used in multiple places across the EVM implementation.

### External Crates:

- `alloc`: The alloc crate provides types for heap allocation.
- `bitvec`: The bitvec crate provides a data structure to handle sequences of bits.
- `bytes`: The bytes crate provides utilities for working with bytes.
- `hex`: The hex crate provides utilities for encoding and decoding hexadecimal.
- `hex_literal`: The hex_literal crate provides a macro for including hexadecimal data directly in the source code.
- `hashbrown`: The hashbrown crate provides high-performance hash map and hash set data structures.
- `ruint`: The ruint crate provides types and functions for big unsigned integer arithmetic.

### Type Aliases:

- `Hash`: An alias for B256, typically used to represent 256-bit hashes or integer values in Ethereum.

### Re-exported Types:

- `Address`: A type representing a 160-bit (or 20-byte) array, typically used for Ethereum addresses.
- `B256`: A type representing a 256-bit (or 32-byte) array, typically used for Ethereum hashes or integers.
- `Bytes`: A type representing a sequence of bytes.
- `U256`: A 256-bit unsigned integer type from the `ruint` crate.
- `HashMap` and `HashSet`: High-performance hash map and hash set data structures from the hashbrown crate.

### Re-exported Modules:
All types, constants, and functions from the `bytecode`, `constants`, `env`, `log`, `precompile`, `result`, `specification`, `state`, and `utilities` modules are re-exported, allowing users to import these items directly from the `primitives` crate.
