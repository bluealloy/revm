# Building from source

It requires running
```bash
git clone https://github.com/bluealloy/revm.git
cd revm
cargo build --release
```

**_Note:_** This project tends to use the newest rust version, so if you're encountering a build error try running rustup update first.

**_Note:_** `clang` is required for building revm with `c-kzg` or `secp256k1` feature flags as they depend on `C` libraries. If you don't have it installed, you can install it with `apt install clang`.

# External core dependencies

REVM relies on several key external dependencies to provide its core functionality:

* The `alloy-primitives` crate provides essential native types including:
  - `ruint` U256 big number type
  - Address, Bytes, and B256 types
  - `hashbrown` implementations of HashMap/HashSet with custom hashing algorithms
* For broad compatibility, REVM is `no_std` compliant and uses the `alloc` crate for heap allocation needs.
* Precompile functionality requires several cryptography and math libraries:
  - `c-kzg`: Implements Polynomial Commitments API for EIP-4844 (C implementation with Rust bindings)
  - `modexp`: Handles big integer modular exponentiation
  - `secp256k1`: Provides ECDSA public key recovery based on secp256k1 curves
  - `k256`: Serves as a no_std compatible fallback for secp256k1
* Transaction environment AccessList and AuthorizationList come from:
  - `alloy-eip7702`
  - `alloy-eip2930`
* Optional serialization support through feature-flagged:
  - `serde`
  - `serde-json` 
* Various utility crates enhance development:
  - `auto_impl`
  - `derive_more`
  - Specialized libraries like `nnum` for specific use cases