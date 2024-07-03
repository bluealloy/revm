# Rust EVM executor or short REVME

`revme` is a binary crate to execute the evm in multiple ways.


## Usage
You can run it directly from the command line with the following command:
```shell
cargo run -p revme  <FLAGS> <SUBCOMMAND>
```

or build an optimized bin and re-use with:
```shell
cargo build -p revme --profile release
```

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

## Evm

`evm` executes any given bytecode and returns the result, for example:

```shell
cargo run -p revme evm 60FF60005261000F600020
```

### Benchmarks

Adding the `--bench` flag will run the benchmarks. It is important to run all the benchmarks in the release mode, as the results can be misleading in the debug mode.

Example of running the benchmarks:
```shell
cargo run -p revme --profile release evm 60FF60005261000F600020 --bench
```

The detailed reports and comparisons can be found in the `target/criterion` directory.