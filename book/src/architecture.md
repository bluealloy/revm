# Architecture and API

REVM is a flexible implementation of the Ethereum Virtual Machine (EVM). It follows the rules of the Ethereum mainnet and stays up to date with changes through hardforks as defined in the official [Ethereum 
execution specs](https://github.com/ethereum/execution-specs).

You can use REVM in two main ways:
1. Run regular Ethereum transactions using an Execution API
2. Create your own custom version of the EVM (for Layer 2 solutions or other chains) using the EVM framework

To see usage examples you can check the [examples folder](https://github.com/bluealloy/revm/tree/main/examples). Other than documentation, examples are the main resource to see and learn about Revm.

The main [`revm`](https://crates.io/crates/revm) library combines all crates into one package and reexports them, standalone libraries are useful if there is a need to import functionality with smaller scope. You can see an overview of all revm crates in the [crates folder](https://github.com/bluealloy/revm/tree/main/crates).

REVM works in `no_std` environments which means it can be used in zero-knowledge virtual machines (zkVMs) and it is the standard library in that use case. It also has very few external dependencies.

# Execution API

[`Evm`](https://docs.rs/revm-context/1.0.0/revm_context/evm/struct.Evm.html) the main structure for executing mainnet ethereum transaction is built with a [`Context`](https://docs.rs/revm-context/latest/revm_context/context/struct.Context.html) and a builder, code for it looks like this:

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet();
let out = evm.transact(tx);
```

[`Evm`](https://docs.rs/revm-context/1.0.0/revm_context/evm/struct.Evm.html) struct contains:
* [`Context`](https://docs.rs/revm-context/latest/revm_context/context/struct.Context.html) - Environment and evm state.
* [`Instructions`](https://docs.rs/revm-handler/latest/revm_handler/instructions/trait.InstructionProvider.html) - EVM opcode implementations
* [`Precompiles`](https://docs.rs/revm-handler/latest/revm_handler/trait.PrecompileProvider.html) - Built-in contract implementations
* [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) - Used for tracing.

And [`Context`](https://docs.rs/revm-context/latest/revm_context/context/struct.Context.html) contains data used in execution:
* Environment data, the data that is known before execution starts are [`Transaction`](https://docs.rs/revm-context-interface/latest/revm_context_interface/transaction/trait.Transaction.html), [`Block`](https://docs.rs/revm-context-interface/latest/revm_context_interface/block/trait.Block.html), [`Cfg`](https://docs.rs/revm-context-interface/latest/revm_context_interface/cfg/trait.Cfg.html).
* [`Journal`](https://docs.rs/revm-context-interface/latest/revm_context_interface/journaled_state/trait.JournalTr.html) is the place where internal state is stored. Internal state is returned after execution ends.
   * And `Database` is an interface that allows fetching external data that is needed at runtime. That data are account, storage and bytecode. When loaded they are stored in [`Journal`](https://docs.rs/revm-context-interface/latest/revm_context_interface/journaled_state/trait.JournalTr.html) 

REVM provides four ways to execute transactions through traits (API):

* `transact(tx)` and `replay()` are function of [`ExecuteEvm`](https://docs.rs/revm-handler/latest/revm_handler/api/trait.ExecuteEvm.html) trait that allow execution transactions. They return the status of execution with reason, changed state and in case of failed execution an error.
* `transact_commit(tx)` and `replay_commit()` are part of [`ExecuteCommitEvm`](https://docs.rs/revm-handler/latest/revm_handler/api/trait.ExecuteCommitEvm.html) that internally commits the state diff to the database and returns status of execution. Database is required to support `DatabaseCommit` trait.
* `inspect()`, `inspect_replay(tx)` and a few others are part of [`InspectEvm`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectEvm.html) trait that allow execution with inspection. This is how tracers are called.
* `inspect_commit()`,`inspect_replay_commit(tx)` are part of the [`InspectCommitEvm`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectCommitEvm.html) trait that extends `InspectEvm` to allow committing state diff after tracing.

For inspection API to be enabled, [`Evm`](https://docs.rs/revm-context/1.0.0/revm_context/evm/struct.Evm.html) needs to be created with inspector.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet().with_inspector(inspector);
let _ = evm.inspect_with_tx(tx);
```

# EVM Framework

To learn how to build your own custom EVM:
- Check out the [example-my-evm](https://github.com/bluealloy/revm/tree/main/examples/my_evm) guide
- Look at [op-revm](https://github.com/bluealloy/revm/tree/main/crates/op-revm) to see how Optimism uses REVM

Each trait needed to build custom EVM has detailed documentation explaining how it works and is worth reading.

In summary, REVM is built around several key traits that enable customizable EVM functionality. The core traits include:

* [`EvmTr`](https://docs.rs/revm-handler/latest/revm_handler/evm/trait.EvmTr.html): The core EVM trait that provides access to `Context`, `Instruction`, `Precompiles`:
* [`ContextTr`](https://docs.rs/revm-context-interface/latest/revm_context_interface/context/trait.ContextTr.html): Accessed through EvmTr, defines the execution environment including Tx/Block/Journal/Db.
* [`Handler`](https://docs.rs/revm-handler/latest/revm_handler/handler/trait.Handler.html): Implements the core execution logic, taking an EvmTr implementation. The default implementation follows Ethereum consensus.

And traits that provide support for inspection and tracing:

* [`InspectorEvmTr`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectorEvmTr.html): Extends EvmTr to enable inspection mode execution with an associated [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) type
* [`InspectorHandler`](https://docs.rs/revm-inspector/latest/revm_inspector/handler/trait.InspectorHandler.html): Extends Handler with inspection-enabled execution paths that make [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) callbacks
* [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html): User-implementable trait for EVM inspection/tracing
