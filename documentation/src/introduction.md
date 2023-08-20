# Introduction

`revm` is an Ethereum Virtual Machine (EVM) written in Rust that is focused on speed and simplicity. This documentation is very much a work in progress and a community effort. If you would like to contribute and improve these docs please make a pr to the [github repo](https://github.com/bluealloy/revm/tree/main). Importantly Revm is just the execution environment for ethereum, there is no networking or consensus related work in this repository.

## Crates

The project has 4 main crates that are used to build the revm. The crates are:

- `revm`: The main EVM library.
- `revm-primitives`: Primitive data types.
- `revm-interpreter`: Execution loop with instructions.
- `revm-precompile`: EVM precompiles.

## Binaries

- `revme`: A CLI binary, used for running state test json.
- `revm-test`: test binaries with contracts; used mostly to check performance.
