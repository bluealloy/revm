# revm - Revolutionary Machine

Is **Rust Ethereum Virtual Machine** with great name that is focused on **speed** and **simplicity**. It gets ispiration from `SputnikVM` (got opcodes/machine from here), `OpenEthereum` and `Geth` with a help from [wolflo/evm-opcodes](https://github.com/wolflo/evm-opcodes). This is probably one of the fastest implementation of EVM, from const EVM Spec to optimistic changelogs for subroutines to merging `eip2929` in EVM state so that it can be accesses only once are some of the things that are improving the speed of execution. 

Here is list of things that i would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of the most important things and most decision are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

## For more INFO check

Please check `bins/revm-test` for simple use case.

All ethereum state tests can be found `bins/revm-ethereum-tests` and can be run with `cargo run --release -- all`

Read more on REVM at [crates/revm/README.md](crates/revm/README.md)
