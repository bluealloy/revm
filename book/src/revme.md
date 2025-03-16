# Revm

Is a binary that allows running statetest and eof validation.

```bash, ignore
$: revme --help
Usage: revme <COMMAND>

Commands:
  statetest       Execute Ethereum state tests
  eof-validation  Execute EOF validation tests
  evm             Run arbitrary EVM bytecode
  bytecode        Print the structure of an EVM bytecode
  bench           Run bench from specified list
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Running eth tests

Eth tests are suite of tests from Ethereum Fondation that are used to test EVM implementations.
Part of these tests are included in revm repository in `tests` folder.

Download eth tests `git clone https://github.com/ethereum/tests`. They can be run with `revme` with command:
`cargo run --release -p revme -- statetest tests/GeneralStateTests/ tests/LegacyTests/Constantinople/GeneralStateTests`
All statetest that can be run by revme can be found in `GeneralStateTests` folder.
