# Bits

This module houses the definitions for fixed-size bit arrays, `B160` and `B256`, to represent 256-bit and 160-bit fixed-size hashes respectively. These are defined using the `construct_fixed_hash!` macro from the `fixed_hash` crate.

The `AsRef` and `Deref` traits from `derive_more` crate are derived for both of these structures, providing convenient methods for converting these types to and from references of their underlying data.

The `Arbitrary` trait from the `arbitrary` crate and the `PropTestArbitrary` trait from `proptest_derive` crate are derived conditionally when either testing or the "arbitrary" feature is enabled. 

The module provides conversions between `B256`, `B160` and various other types such as `u64`, `primitive_types::H256`, `primitive_types::H160`, `primitive_types::U256`, and `ruint::aliases::U256`. The `impl` From blocks specify how to convert from one type to another.

`impl_fixed_hash_conversions!` macro is used to define conversions between `B256` and `B160` types.

If the "serde" feature is enabled, the Serialize and Deserialize traits from the serde crate are implemented for `B256` and `B160` using a custom serialization method that outputs/reads these types as hexadecimal strings. This includes a custom serialization/deserialization module for handling hexadecimal data.
