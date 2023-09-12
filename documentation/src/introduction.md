# Introduction

`revm` is an Ethereum Virtual Machine (EVM) written in Rust that is focused on speed and simplicity. This documentation is very much a work in progress and a community effort. If you would like to contribute and improve these docs please make a pr to the [github repo](https://github.com/bluealloy/revm/tree/main). Most importantly, Revm is just the execution environment for ethereum; there is no networking or consensus related work in this repository.

## Crates

The project has 4 main crates that are used to build revm. These are:

- `revm`: The main EVM library.
- `primitives`: Primitive data types.
- `interpreter`: Execution loop with instructions.
- `precompile`: EVM precompiles.

## Testing with the binaries

There are two binaries both of which are used for testing. To install them run `cargo install --path bins/<binary-name>`. The binaries are:

- `revme`: A CLI binary, used for running state test json. Currently it is used to run [ethereum tests](https://github.com/ethereum/tests) to check if revm is compliant. For example if you have the eth tests cloned into a directory called eth tests and the EIP tests in the following directories you can run 
```bash
cargo run --profile ethtests -p revme -- \                                           
    statetest \
    ../ethtests/GeneralStateTests/ \
    ../ethtests/LegacyTests/Constantinople/GeneralStateTests/ \
    bins/revme/tests/EIPTests/StateTests/stEIP5656-MCOPY/ \
    bins/revme/tests/EIPTests/StateTests/stEIP1153-transientStorage/
```

- `revm-test`: test binaries with contracts; used mostly to check performance

If you are interested in contributing, be sure to run the statetests. It is recommeded to read about the [ethereum tests](https://ethereum-tests.readthedocs.io/en/latest/).  
