# Introduction

`revm` is an Ethereum Virtual Machine (EVM) written in Rust that is focused on speed and simplicity. This documentation is very much a work in progress and a community effort. If you would like to contribute and improve these docs please make a pr to the [github repo](https://github.com/bluealloy/revm/tree/main). Importantly Revm is just the execution environment for ethereum, there is no networking or consensus related work in this repository.

## Crates

The project hase 4 main crates that are used to build the revm. The crates are:

- `revm`: The main EVM library.
- `primitives`: Primitive data types.
- `interpreter`: Execution loop with instructions.
- `precompile`: EVM precompiles.

## Binaries

- `revme`: A CLI binary, used for running state test json. Currently it is used to run ethereum tests:
* statetest: takes path to folder where ethereum statetest json can be found. It recursively searches for all json files and execute them. This is how I run all https://github.com/ethereum/tests to check if revm is compliant. Example `revme statests test/GenericEvmTest/`
- `revm-test`: test binaries with contracts; used mostly to check performance.
