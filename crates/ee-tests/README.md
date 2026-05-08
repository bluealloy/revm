# revm-ee-tests

Shared test utilities and integration tests for **revm**.

## Running Tests

```bash
# Run all tests
cargo test -p revm-ee-tests

# Run a specific test subset (e.g. EIP-8037 / TIP-1016 state gas tests)
cargo test -p revm-ee-tests eip8037
```

## Directory Structure

```
crates/ee-tests/
├── src/
│   ├── lib.rs              # Snapshot comparison utilities (TestdataConfig, compare_or_save_testdata)
│   └── revm_tests.rs       # Integration tests for mainnet revm
├── tests/
│   └── revm_testdata/      # Golden JSON snapshots for revm tests
├── eip8037.md              # EIP-8037 / TIP-1016 State Gas test plan
└── Cargo.toml
```

Snapshot files are auto-generated on first run and compared on subsequent runs.
