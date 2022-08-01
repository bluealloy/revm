# revm - Rust Ethereum Virtual Machine

Is EVM written in rust that is focused on **speed** and **simplicity**. It has fast and flexible implementation with simple interface and embedded Host. It is passing all `ethereum/tests` test suits

Here is list of things that i would like to use as guide in this project:
- **EVM compatibility and stability** - this goes without saying but it is nice to put it here. In blockchain industry, stability is most desired attribute of any system.
- **Speed** - is one of the most important things and most decision are made to complement this.
- **Simplicity** - simplification of internals so that it can be easily understood and extended, and interface that can be easily used or integrated into other project.
- **interfacing** - `[no_std]` so that it can be used as wasm lib and integrate with JavaScript and cpp binding if needed.

# Project structure

* crates
    * revm -> main EVM library
    * revm_precompiles -> EVM precompiles are standalone
    * revmjs -> Binding for js. (in not finished state)
* bins:
    * revme: cli binary, used for running state test json
    * revm-test: test binaries with contracts, used mostly to checke performance (will proably merge it inside revme).
# Running eth tests

go to `cd bins/revme/`

Download eth tests from (this will take some time): `git clone https://github.com/ethereum/tests`

run tests with command: `cargo run --release -- statetest tests/GeneralStateTests/`

`GeneralStateTests` contains all tests related to EVM.

# Used by

* Foundry: https://github.com/foundry-rs/foundry

(If you want to add your project to the list, ping me or open the PR)


# Contact

There is public telegram group: https://t.me/+Ig4WDWOzikA3MzA0

Or you can contact me directly on email: dragan0rakita@gmail.com


