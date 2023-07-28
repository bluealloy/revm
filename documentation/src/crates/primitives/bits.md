# bits

This module houses the definitions for fixed-size bit arrays, `B160` and `B256`, showcasing its role in managing bits-related operations, to represent 256-bit and 160-bit fixed-size hashes respectively. These are defined using the `construct_fixed_hash!` macro from the `fixed_hash` crate.

The `AsRef` and `Deref` traits from `derive_more` crate are derived for both of these structures, providing convenient methods for converting these types to and from references of their underlying data.

The `Arbitrary` trait from the `arbitrary` crate and the `PropTestArbitrary` trait from `proptest_derive` crate are derived conditionally when either testing or the "arbitrary" feature is enabled. These traits are used for property testing, a form of testing where random inputs are generated and used to validate certain properties of your code.

The code also provides conversions between `B256`, `B160` and various other types such as `u64`, `primitive_types::H256`, `primitive_types::H160`, `primitive_types::U256`, and `ruint::aliases::U256`. The `impl` From blocks specify how to convert from one type to another.

`impl_fixed_hash_conversions!` macro is used to define conversions between `B256` and `B160` types.

If the "serde" feature is enabled, the Serialize and Deserialize traits from the serde crate are implemented for `B256` and `B160` using a custom serialization method that outputs/reads these types as hexadecimal strings. This includes a custom serialization/deserialization module for handling hexadecimal data.

This module (serialize) provides functionality to serialize a slice of bytes to a hexadecimal string, and deserialize a hexadecimal string to a byte vector with size checks. It handles both "0x" prefixed and non-prefixed hexadecimal strings. It also provides functions to convert raw bytes to hexadecimal strings and vice versa, handling potential errors related to non-hexadecimal characters. The module also defines the `ExpectedLen` enum which is used to specify the expected length of the byte vectors during deserialization.
