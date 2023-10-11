# Utilities

This Rust module provides utility functions and constants for handling Keccak hashing (used in Ethereum) and creating Ethereum addresses via legacy and `CREATE2` methods. It also includes serialization and deserialization methods for hexadecimal strings representing byte arrays.

The `KECCAK_EMPTY` constant represents the Keccak-256 hash of an empty input.

The `keccak256` function takes a byte slice input and returns its Keccak-256 hash as a `B256` value.
