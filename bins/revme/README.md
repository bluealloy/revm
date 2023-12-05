# Rust EVM executor or short REVME

`revme` is a binary crate to execute revm in multiple ways.

Each section below provides details for the various `revme` subcommands.

## State Tests

`statetest` takes a path to the directory where ethereum statetest json can be found.
It recursively parses all json files in the specified directory and executes them.

Running all [ethereum tests][et] checks that revm is compliant to the ethereum specs.

To run [ethereum tests][et] locally, clone the [tests][et] repository and provide the
test directory. Below, we clone the repo and execute the `GeneralStateTests` suite of
tests.

```shell
git clone https://github.com/ethereum/tests
cargo run -p revme statetest tests/GeneralStateTests
```

*Notice, in the [`.gitignore`](../../.gitignore), the `bins/revme/tests` directory
is ignored so it won't be checked into git.*

[et]: https://github.com/ethereum/tests

## Traversal

`traverse` queries all tx-enriched blocks and executes transactions sequentially
against revm. The starting block can be specified through the `--start-block` flag
(or `-s` shorthand). If not provided, execution will begin at block `0`.  Similarly,
the end block can be provided through the `--end-block` flag (or `-e` shorthand),
and will use the latest block number if missing.

Executing `traverse` requires you to pass a valid JSON RPC http provider endpoint
to the `--rpc` (or `-r` shorthand) flag. See the below shell command for an example
of running `traverse`.

```shell
cargo run -p revme traverse \
    --rpc https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27
```

*Note, this will take a long time to execute and may induce significant load on the
specified rpc provider. It it recommended to specify a start and end block with
`--start-block` and `--end-block` respectively to test. An example of this is provided
below.*

```shell
cargo run -p revme traverse \
    --start-block 100 \
    --end-block 101 \
    --rpc https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27
```
