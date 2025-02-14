# Dev Section

Contains information for developers who want to build and develop revm.

### Building from source

```shell
git clone https://github.com/bluealloy/revm.git
cd revm
cargo build --release
```

**_Note:_** This project tends to use the newest rust version, so if you're encountering a build error try running rustup update first.

**_Note:_** `clang` is required for building revm with `c-kzg` or `secp256k1` feature flags as they depend on `C` libraries. If you don't have it installed, you can install it with `apt install clang`.

### Importing crates

Main crate is revm, inspection is in saparate crate revm-inspection and revm-database has a few database implementations.

Curent versions of those crates are:
```toml
revm = "20.0.0-alpha.1"
revm-inspection = "0.1.0"
revm-database = "0.1.0"
```

Additional tools that can be useful can be found in [Awesome Revm](./awesome.md) section of the book. For example paradigm/revm-inspector crate contains a lot of Inspector used for tracing inside Reth and Foundry.

### no_std support

If you are in need of no_std support you would need to use no-default-features flag and disable default features. All crates (expect revme binary) support no_std in some capacity.

Downside of no_std support is that of right now we don't have a bls12-381 implementation that is no_std compatible. PR that would solve this: 


### Use case

Below is a simple usage example of Execution API: where we create an EVM and transact a transaction or when we want to generate traces with the inspector.

```rust
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);

// or you can use powerful inspection tool to trace it
let mut evm = evm.with_inspector(tracer);
let out = evm.inspect_with_tx(tx);
```

The usage of the Evm Framework API is a bit more complex, but it is extensively explained here. It allows extending logic, adding different context types, and supporting inspection out of the box. For an example, you can check the revm-optimism crate.

Examples section of the book lists all examples that are available in the repository. And it is good starting point for understanding how to use revm.

### Running eth tests

Eth tests are suite of tests from Ethereum Fondation that are used to test EVM implementations.
Part of these tests are included in revm repository in `tests` folder.

Download eth tests from (this will take some time): `git clone https://github.com/ethereum/tests` all statetest that can be run by revme can be found in `GeneralStateTests` folder.

When you have downlodaed the test they can be run via `revme` with command:
`cargo run --release -p revme -- statetest tests/GeneralStateTests/ tests/LegacyTests/Constantinople/GeneralStateTests`

After running `cargo build --release -p revme` the `revme` binary can be found in `target/release` folder.


### CI/CD

CI/CD runs on GitHub actions and will test no_std/clippy/fmt correctness of the code. It will also run el statetests and benchmarks with codspeed. You can find the configuration in `.github/workflows` folder.