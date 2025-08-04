# Revme

Is a binary that allows running statetest and eof validation.

```bash, ignore
$: revme --help
Usage: revme <COMMAND>

Commands:
  statetest       Execute Ethereum state tests
  evm             Run arbitrary EVM bytecode
  bytecode        Print the structure of an EVM bytecode
  bench           Run bench from specified list
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Running eth tests

Eth tests are a suite of tests from the Ethereum Foundation that are used to test EVM implementations.
Part of these tests are included in the revm repository in the `tests` folder.

est suites for the latest hardforks can be found in [EEST releases](https://github.com/ethereum/execution-spec-tests/releases), and there are additional tests that cover older hardforks in [legacytests](https://github.com/ethereum/legacytests)

Revm can run statetest type of tests with `revme` using the following command:
`cargo run --release -p revme -- statetest folder_path`

For running EEST tests, we can use  the `./scripts/run-tests.sh.`

For legacy tests, we need to first download the repo `git clone https://github.com/ethereum/legacytests` and then run it with `cargo run --release -p revme -- statetest legacytests/Cancun/GeneralStateTests`
All statetest that can be run by revme can be found in the `GeneralStateTests` folder.
