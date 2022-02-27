# revm - Revolutionary Machine

Is **Rust Ethereum Virtual Machine** with great name that is focused on **speed** and **simplicity**. It gets ispiration from `SputnikVM` (got opcodes/interp from here), `OpenEthereum` and `Geth` with a help from [wolflo/evm-opcodes](https://github.com/wolflo/evm-opcodes). 

It is fast and flexible implementation of EVM with simple interface and embeded Host, there are multiple things done on Host part from const EVM Spec to optimistic changelogs for subroutines to merging `eip2929` in EVM state so that it can be accesses only once that are improving the speed of execution. There are still some improvements on Interepter part that needs to be done so that we can be comparable with evmone, for more info track [this issue](https://github.com/bluealloy/revm/issues/7).

Here is list of things that i would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of the most important things and most decision are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

Read more on REVM here [crates/revm/README.md](crates/revm/README.md)

For executable binary that contains `debuger`, `statetest` for running eth/tests, and `runner` check REVME at [bin/revme/README.md](bins/revme/README.md)
