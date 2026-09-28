# Rust EVM executor or short REVME

`revme` is a binary crate to execute the evm in multiple ways.

Currently it is mainly used to run ethereum tests with the `statetest` subcommand.

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

## Jumpdest analysis benchmark fixtures

The [CREATE jumpdest-analysis fixtures](fixtures/jumpdest-analysis/README.md)
reproduce execution-specs PR #3631 on Amsterdam, including all four randomized
patterns and nine periodic controls:

```shell
cargo run --release -p revme -- btest bins/revme/fixtures/jumpdest-analysis
```
