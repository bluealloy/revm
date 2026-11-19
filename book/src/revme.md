# Revme

Revme is a binary for running Ethereum state tests and blockchain tests, and for inspecting EVM bytecode.

```bash, ignore
$: revme --help
Usage: revme <COMMAND>

Commands:
  statetest        Execute Ethereum state tests
  stest            Execute Ethereum state tests
  evm              Run arbitrary EVM bytecode
  bytecode         Print the structure of an EVM bytecode
  bench            Run bench from specified list
  blockchaintest   Execute Ethereum blockchain tests
  btest            Execute Ethereum blockchain tests
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Running eth tests

Eth tests are a suite of tests from the Ethereum Foundation that are used to test EVM implementations.

Test suites for the latest hardforks can be found in [EEST releases](https://github.com/ethereum/execution-spec-tests/releases), and there are additional tests that cover older hardforks in [legacytests](https://github.com/ethereum/legacytests).

Fixtures are not checked into this repository. Use `./scripts/run-tests.sh`, which downloads EEST fixtures into `test-fixtures/` (develop by default; set `REVM_STATETEST_STABLE=1` for stable).

Revm can run statetest type of tests with `revme` using the following command:
`cargo run --release -p revme -- statetest folder_path`

Blockchain tests can be run with:
`cargo run --release -p revme -- blockchaintest folder_path`

For legacy tests, we need to first download the repo `git clone https://github.com/ethereum/legacytests` and then run it with `cargo run --release -p revme -- statetest legacytests/Cancun/GeneralStateTests`
All statetest that can be run by revme can be found in the `GeneralStateTests` folder.
