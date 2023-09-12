# Precompile

The `precompile` crate contains the implementation of the Ethereum precompile opcodes in the EVM. Precompiles are a shortcut to execute a function implemented by the EVM itself, rather than an actual contract. Precompiled contracts are essentially predefined smart contracts on Ethereum, residing at hardcoded addresses and used for computationally heavy operations that are cheaper when implemented this way. There are 6 precompiles implemented in REVM, and they are: `blake2`, `bn128` curve, `identity`, `secp256k1`, `modexp`, and `sha256` and `ripemd160` hash functions.

Modules:

- [blake2](./precompile/blake2.md): This module implements the `BLAKE2` compression function, as specified in EIP-152.
- [bn128](./precompile/bn128.md): This module contains the implementations of precompiled contracts for addition, scalar multiplication, and optimal ate pairing check on the alt_bn128 elliptic curve.
- [hash](./precompile/hash.md): This module includes the implementations for the `SHA256` and `RIPEMD160` hash function precompiles.
- [identity](./precompile/identity.md): This module implements the Identity precompile, which returns the input data unchanged.
- [modexp](./precompile/modexp.md): This module implements the big integer modular exponentiation precompile.
- [secp256k1](./precompile/secp256k1.md): This module implements the ECDSA public key recovery precompile, based on the secp256k1 curve.

Types and Constants:

- `B160`: A type alias for an array of 20 bytes. This is typically used to represent Ethereum addresses.
- `B256`: A type alias for an array of 32 bytes, typically used to represent 256-bit hashes or integer values in Ethereum.
- `PrecompileOutput`: Represents the output of a precompiled contract execution, including the gas cost, output data, and any logs generated.
- `Log`: Represents an Ethereum log, with an address, a list of topics, and associated data.
- `Precompiles`: A collection of precompiled contracts available in a particular hard fork of Ethereum.
- `Precompile`: Represents a precompiled contract, which can either be a standard Ethereum precompile, or a custom precompile.
- `PrecompileAddress`: Associates a precompiled contract with its address.
- `SpecId`: An enumeration representing different hard fork specifications in Ethereum, such as Homestead, Byzantium, Istanbul, Berlin, and Latest.

Functions:

- `calc_linear_cost_u32`: A utility function to calculate the gas cost for certain precompiles based on their input length.
- `u64_to_b160`: A utility function for converting a 64-bit unsigned integer into a 20-byte Ethereum address.

External Crates:

- [alloc](https://doc.rust-lang.org/alloc/): The alloc crate provides types for heap allocation, and is used here for the `Vec` type.
- [core](https://doc.rust-lang.org/core/): The core crate provides fundamental Rust types, macros, and traits, and is used here for `fmt::Result`.

Re-exported Crates and Types:

- `revm_primitives`: This crate is re-exported, indicating it provides some types used by the precompile crate.
- `primitives`: Types from the `primitives` module of `revm_primitives` are re-exported, including `Bytes`, `HashMap`, and all types under `precompile`. The latter includes the `PrecompileError` type, which is aliased to `Error`.

Re-exported Functionality:

- `Precompiles` provides a static method for each Ethereum hard fork specification (e.g., `homestead`, `byzantium`, `istanbul`, `berlin`, and `latest`), each returning a set of precompiles for that specification.
- `Precompiles` also provides methods to retrieve the list of precompile addresses (`addresses`), to check if a given address is a precompile (`contains`), to get the precompile at a given address (`get`), to check if there are no precompiles (`is_empty`), and to get the number of precompiles (`len`).
