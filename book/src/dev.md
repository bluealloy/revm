# Building from source

To build REVM from source:

```bash
git clone https://github.com/bluealloy/revm.git
cd revm
cargo build --release
```

**_Note:_** This project tends to use the newest rust version, so if you're encountering a build error try running `rustup update` first.

**_Note:_** `clang` is required for building revm with `c-kzg` or `secp256k1` feature flags as they depend on `C` libraries. If you don't have it installed, you can install it with `apt install clang`.

## Running Tests

REVM has a comprehensive test suite. Here are the main commands:

```bash
# Run all tests
cargo nextest run --workspace

# Run tests for a specific crate
cargo test -p revm

# Run with all features enabled
cargo test --workspace --all-features
```

## Linting and Formatting

Before submitting code, make sure it passes all checks:

```bash
# Format code
cargo fmt --all

# Run clippy linter
cargo clippy --workspace --all-targets --all-features

# Check no_std compatibility
cargo check --target riscv32imac-unknown-none-elf --no-default-features
```

## Running Ethereum Tests

REVM is tested against the official Ethereum test suite:

```bash
# Download and run ethereum tests
./scripts/run-tests.sh

# Clean test fixtures and re-run
./scripts/run-tests.sh clean

# Run with release profile for better performance
./scripts/run-tests.sh release

# Run specific state tests
cargo run -p revme statetest path/to/tests
```

## Feature Flags

REVM supports various feature flags:

- `std`: Standard library support (enabled by default)
- `serde`: Serialization support
- `c-kzg`: KZG commitment support for EIP-4844
- `secp256k1`: Secp256k1 curve operations
- `optimism`: Optimism L2 support

To build with specific features:

```bash
cargo build --features "serde,c-kzg"
cargo build --no-default-features  # for no_std
```