# Rust EVM executor or short REVME

`revme` is a binary crate to execute the evm in multiple ways.

Currently it is mainly used to run ethereum tests with the `statetest` subcommand.

## State Tests

`statetest` takes a path to the directory where ethereum statetest json can be found.
It recursively parses all json files in the specified directory and executes them.

Running Ethereum execution-spec tests (EEST) checks that revm is compliant to the ethereum specs.

To download fixtures and run them locally, use the repo helper script (downloads EEST
fixtures into `test-fixtures/` by default):

```shell
./scripts/run-tests.sh
```

For older hardfork coverage, clone [legacytests][legacytests] and point `statetest` at a
`GeneralStateTests` tree:

```shell
git clone https://github.com/ethereum/legacytests
cargo run -p revme -- statetest legacytests/Cancun/GeneralStateTests
```

See also the [revme book page](https://bluealloy.github.io/revm/revme.html#running-eth-tests).

*Notice, in the [`.gitignore`](../../.gitignore), `bins/revme/tests` and `/test-fixtures`
are ignored so fixtures won't be checked into git.*

[legacytests]: https://github.com/ethereum/legacytests
