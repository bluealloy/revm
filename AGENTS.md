# AGENTS.md

REVM is a highly efficient Rust implementation of the Ethereum Virtual Machine (EVM). It serves both as:
1. A standard EVM for executing Ethereum transactions
2. A framework for building custom EVM variants (like Optimism's op-revm)

The project is used by major Ethereum infrastructure including Reth, Foundry, Hardhat, Optimism, Scroll, and many zkVMs.

## Commands

```bash
cargo build --workspace
cargo nextest run --workspace
cargo nextest run --workspace --no-default-features
cargo nextest run --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check

cargo check --target riscv32imac-unknown-none-elf --no-default-features
cargo check --target riscv64imac-unknown-none-elf --no-default-features
cargo check --target riscv32imac-unknown-none-elf -p revm-database --no-default-features
cargo check --target riscv64imac-unknown-none-elf -p revm-database --no-default-features

./scripts/run-tests.sh
cargo run --release -p revme -- statetest test-fixtures/main/stable/state_tests
```

Run `cargo fmt --all` before committing.

## Structure

- `crates/revm`: main crate and re-exports.
- `crates/primitives`: primitive types and constants.
- `crates/bytecode`: bytecode analysis, EOF validation, opcode tables.
- `crates/interpreter`: opcode execution and interpreter internals.
- `crates/context-interface`: context, environment, journal, and frame stack traits.
- `crates/context`: default context, journal, and `Evm` container.
- `crates/handler`: mainnet execution flow, frames, validation, APIs.
- `crates/database-interface`: database traits.
- `crates/database`: database implementations.
- `crates/state`: account/storage/state types.
- `crates/precompile`: precompiled contracts.
- `crates/inspector`: tracing and inspector APIs.
- `crates/statetest-types`: Ethereum state test types.
- `crates/ee-tests`: execution-spec-test helpers.
- `bins/revme`: CLI for tests and validation.
- `examples`: API usage examples.
- `book/src`: docs.
