# revm

[![CI](https://github.com/bluealloy/revm/actions/workflows/ci.yml/badge.svg)][gh-ci]
[![License](https://img.shields.io/badge/License-MIT-orange.svg)][mit-license]
[![Chat][tg-badge]][tg-url]

[mit-license]: https://opensource.org/license/mit/
[gh-ci]: https://github.com/bluealloy/revm/actions/workflows/ci.yml
[tg-url]: https://t.me/+Ig4WDWOzikA3MzA0
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue

**Rust Ethereum Virtual Machine**

![](./assets/revm-banner.png)

Revm is an EVM written in Rust that is focused on **speed** and **simplicity**.
It has a fast and flexible implementation with a simple interface and embedded Host.
It passes all `ethereum/tests` test suites.

Here is a list of guiding principles that Revm follows.

* **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In the blockchain industry, stability is the most desired attribute of any system.
* **Speed** - is one of the most important things and most decisions are made to complement this.
* **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other projects.
* **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

# Project

Structure:

* crates
  * revm -> main EVM library.
  * revm-primitives -> Primitive data types.
  * revm-interpreter -> Execution loop with instructions
  * revm-precompile -> EVM precompiles
* bins:
  * revme: cli binary, used for running state test jsons

There were some big efforts on optimization of revm:

* Optimizing interpreter loop: https://github.com/bluealloy/revm/issues/7
* Introducing Bytecode format (and better bytecode analysis): https://github.com/bluealloy/revm/issues/121
* Unification of instruction signatures: https://github.com/bluealloy/revm/pull/283

## Supported Rust Versions

<!--
When updating this, also update:
- clippy.toml
- Cargo.toml
- .github/workflows/ci.yml
-->

Revm will keep a rolling MSRV (minimum supported rust version) policy of **at
least** 6 months. When increasing the MSRV, the new Rust version must have been
released at least six months ago. The current MSRV is 1.66.0.

Note that the MSRV is not increased automatically, and only as part of a minor
release.

## Building from source

```shell
git clone https://github.com/bluealloy/revm.git
cd revm
cargo build --release
```

**_Note:_** `clang` is required for building revm with `c-kzg` or `secp256k1` feature flags as they
depend on `C` libraries. You might need to install the `clang` package using your system's package
manager.

## Running tests

Unit and integration tests:

```shell
cargo test --workspace --all-features
```

Ethereum Execution Tests ([`ethereum/tests`](https://github.com/ethereum/tests)):

```shell
# Clone the `ethereum/tests` repo to `ethtests` (this will take some time)
git clone https://github.com/ethereum/tests ethtests
# Run all relevant tests
# See `.github/workflows/ethereum-tests.yml` for the most up to date list of tests
cargo run --profile ethtests -p revme -- statetest \
  ethtests/GeneralStateTests/ \
  ethtests/LegacyTests/Constantinople/GeneralStateTests/ \
  ethtests/EIPTests/StateTests/stEIP1153-transientStorage/ \
  ethtests/EIPTests/StateTests/stEIP4844-blobtransactions/ \
  ethtests/EIPTests/StateTests/stEIP5656-MCOPY/
```

## Running benchmarks

Requires [`cargo-criterion`](https://github.com/bheisler/cargo-criterion):

```shell
cargo criterion
```

Alternatively, you can just use `cargo bench`, but this will produce less detailed results.

## Running examples

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
