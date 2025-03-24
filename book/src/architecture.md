# Architecture and API

REVM is a flexible implementation of the Ethereum Virtual Machine (EVM). It follows the rules of the Ethereum mainnet and stays up to date with changes through hardforks as defined in the official [Ethereum 
execution specs](https://github.com/ethereum/execution-specs).

You can use REVM in two main ways:
1. Run regular Ethereum transactions using a Execution API
2. Create your own custom version of the EVM (for Layer 2 solutions or other chains) using EVM framework

To see usage examples you can check the [examples folder](https://github.com/bluealloy/revm/tree/main/examples). Other than documentation, examples are main resource to see and learn about Revm.

The main `revm` library combines all crates into one package and reexports them, standalone library are useful if there is need to import functionality with smaller scope. You can see overview of revm crates in [crates folder](https://github.com/bluealloy/revm/tree/main/crates).

REVM works in `no_std` environments which means it can be used in zero-knowledge virtual machines (zkVMs) and it is the standard library in that use case. It also has very few external dependencies.

# Execution API

`Evm` the main structure for executing mainnet ethereum transaction is built with a `Context` and a builder, code for it looks like this:

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);
```

`Evm` struct contains:
* `Context` - Environment and evm state.
* `Instructions` - EVM opcode implementations
* `Precompiles` - Built-in contract implementations
* `Inspector` - Used for tracing.

And `Context` contains data used in execution:
* Environment data, the data that is known before execution starts are `Transaction`, `Block`, `Cfg`.
* `Journal` is place where internal state is stored. Internal state is returned after execution ends.
   * And `Database` is a interface that allows fetching external data that is needed in runtime. That data are account, storage and bytecode. When loaded they are stored in `Journal` 

REVM provides four ways to execute transactions through traits (API):

* `transact(tx)` and `replay()` are function of `ExecuteEvm` trait that allow execution transactions. They return the status of execution with reason, changed state and in case of failed execution an error.
* `transact_commit(tx)` and `replay_commit()` are part of `ExecuteCommitEvm` that internally commits the state diff to the database and returns status of execution. Database is required to support `DatabaseCommit` trait.
* `inspect()`, `inspect_replay(tx)` and a few others are part of `InspectEvm` trait that allow execution with inspection. This is how tracers are called.
* `inspect_commit()`,`inspect_replay_commit(tx)` are part of the `InspectCommitEvm` trait that extends `InspectEvm` to allow committing state diff after tracing.

For inspection API to be enabled, `Evm` needs to be created with inspector.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet().with_inspector(inspector);
let _ = evm.inspect_with_tx(tx);
```

# EVM Framework

To learn how to build your own custom EVM:
- Check out the [example-my-evm](https://github.com/bluealloy/revm/tree/main/examples/my_evm) guide
- Look at [op-revm](https://github.com/bluealloy/revm/tree/main/crates/optimism) to see how Optimism uses REVM

Each trait needed to build custom EVM has detailed documentation explaining how it works and is worth reading.

In summary, REVM is built around several key traits that enable customizable EVM functionality. The core traits include:

* **EvmTr**: The core EVM trait that provides access to `Context`, `Instruction`, `Precompiles`:
* **ContextTr**: Accessed through EvmTr, defines the execution environment including Tx/Block/Journal/Db.
* **Handler**: Implements the core execution logic, taking an EvmTr implementation. The default implementation follows Ethereum consensus.

And traits that provide support for inspection and tracing:

* **InspectorEvmTr**: Extends EvmTr to enable inspection mode execution with an associated `Inspector` type
* **InspectorHandler**: Extends Handler with inspection-enabled execution paths that make `Inspector` callbacks
* **Inspector**: User-implementable trait for EVM inspection/tracing
