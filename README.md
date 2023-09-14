# revm - Rust Ethereum Virtual Machine

Is EVM written in rust that is focused on **speed** and **simplicity**. It has fast and flexible implementation with simple interface and embedded Host. It is passing all `ethereum/tests` test suits

Here is a list of things that I would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In the blockchain industry, stability is the most desired attribute of any system.
- **Speed** - is one of the most important things and most decisions are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other projects.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.


# Project

Structure:
* crates
    * revm -> main EVM library.
    * revm-primitives -> Primitive data types.
    * revm-interpreter -> Execution loop with instructions
    * revm-precompile -> EVM precompiles
* bins:
    * revme: cli binary, used for running state test jsons

This project tends to use the newest rust version, so if you're encountering a build error try running `rustup update` first.

There were some big efforts on optimization of revm:
* Optimizing interpreter loop: https://github.com/bluealloy/revm/issues/7
* Introducing Bytecode format (and better bytecode analysis): https://github.com/bluealloy/revm/issues/121
* Unification of instruction signatures: https://github.com/bluealloy/revm/pull/283

# Running eth tests

go to `cd bins/revme/`

Download eth tests from (this will take some time): `git clone https://github.com/ethereum/tests`

run tests with command: `cargo run --release -- statetest tests/GeneralStateTests/ tests/LegacyTests/Constantinople/GeneralStateTests`

`GeneralStateTests` contains all tests related to EVM.

## Running benchmarks

TODO needs to be updated. Benches can now be found inside `crates/revm/benches`

```shell
cargo run --package revm-test --release --bin snailtracer
```

```shell
cargo flamegraph --root --freq 4000 --min-width 0.001 --package revm-test --bin snailtracer
```

## Running example

```shell
cargo run -p revm --features ethersdb --example fork_ref_transact
```

# Used by:

* [Foundry](https://github.com/foundry-rs/foundry) is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.
* [Helios](https://github.com/a16z/helios) is a fully trustless, efficient, and portable Ethereum light client written in Rust.
* [Reth](https://github.com/paradigmxyz/reth) Modular, contributor-friendly and blazing-fast implementation of the Ethereum protocol
* [Arbiter](https://github.com/primitivefinance/arbiter) is a framework for stateful Ethereum smart-contract simulation
* [Zeth](https://github.com/risc0/zeth) is an open-source ZK block prover for Ethereum built on the RISC Zero zkVM.
* ...

(If you want to add project to the list, ping me or open the PR)


# Documentation

The book can be found at github page here: https://bluealloy.github.io/revm/

The documentation (alas needs some love) can be found here: https://bluealloy.github.io/revm/docs/

To serve the mdbook documentation in a local environment, ensure you have mdbook installed (if not install it with cargo) and then run:

```shell
mdbook serve documentation
```

# Contact

There is public telegram group: https://t.me/+Ig4WDWOzikA3MzA0

Or if you want to hire me or contact me directly, here is my email: dragan0rakita@gmail.com and telegram: https://t.me/draganrakita
