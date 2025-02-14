# External core dependencies

Revm project is no_std compliant, and it uses `alloc` crate for heap allocation.

Native types are imported from `alloy-primitives` crate, this is where we get Address, Bytes, B256 types where big number U256 is exported from `ruint`. Additionally `hashbrown` crate or `HashMap/HashSet` with different hashing algorithm are imported from it.

`alloy-eip7702` and `alloy-eip2930` are used in TxEnv as parts of transaction.

Precompiles require a lot of cryprography and math libs:
- `c-kzg` is a minimal implementation of the Polynomial Commitments API for EIP-4844, written in C. (With rust bindings)
- `modexp` is a big integer modular exponentiation precompile.
- `secp256k1` is an ECDSA public key recovery precompile, based on `secp256k1` curves. `k256` is used as a fallback for no_std environments.
- ...

`serde` and `serde-json` are behind a feature flag to serialize and deserialize data structures.

Some utility crates are used throughout the codebase: `auto_impl`, `derive_more` etc. Where in some places few libs are imported for specific use cases like `nnum` or `dawdaw`.

