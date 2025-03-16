# Architecture

REVM is a flexible implementation of the Ethereum Virtual Machine (EVM). It follows the rules of the Ethereum mainnet and stays up to date with changes through hardforks as defined in the official [Ethereum 
execution specs](https://github.com/ethereum/execution-specs).

You can use REVM in two main ways:
1. Run regular Ethereum transactions using a Execution API
2. Create your own custom version of the EVM (for Layer 2 solutions or other chains) using EVM framework

The main `revm` library combines all the different crates into one package and reexports them. You can see overview revm crates in [crates folder](https://github.com/bluealloy/revm/tree/main/crates) and overview of examples in [examples folder](https://github.com/bluealloy/revm/tree/main/examples).

REVM works in no_std environments which means it can be used in zero-knowledge virtual machines (zkVMs). It also has very few external dependencies, which you can read more about in the [dev section](./dev.md).

# Execution API

REVM provides four ways to execute transactions through traits (interfaces).

The State system builds on the `Database` trait and handles:
- Getting data from external storage
- Managing the EVM's output
- Caching changes when running multiple transactions

You can implement one of three database interfaces depending on what you need:

- `Database`: Uses a mutable reference (`&mut self`). This is useful when you want to update a cache or track statistics while getting data. Enables basic transaction functions like `transact` and `inspect`.

- `DatabaseRef`: Uses a regular reference (`&self`). Good for when you only need to read data without making changes. Enables reference-based functions like `transact_ref`.

- `Database + DatabaseCommit`: Adds the ability to save transaction changes directly. Enables commit functions like `transact_commit`.

# EVM Framework

To learn how to build your own custom EVM:
- Check out the [example-my-evm](https://github.com/bluealloy/revm/tree/rakita/my_evm/examples/my_evm) guide
- Look at [op-revm](https://github.com/bluealloy/revm/tree/main/crates/optimism) to see how Optimism uses REVM

Each trait needed to build custom EVM has detailed documentation explaining how it works and it is worth reading.

In summary, REVM is built around several key traits that enable customizable EVM functionality. The core traits include:

* **EvmTr**: The core EVM trait that provides access to the main EVM components:
  - Context - Environment and state access
  - Instructions - EVM opcode implementations
  - Precompiles - Built-in contract implementations
  - Inspector - Used for tracing, only enabled with `InspectorEvmTr` trait.
* **ContextTr**: Accessed through EvmTr, defines the execution environment including Tx/Block/Journal/Db:
* **Handler**: Implements the core execution logic, taking an EvmTr implementation. The default implementation follows Ethereum consensus.

And traits that provide support for inspection and tracing:

* **InspectorEvmTr**: Extends EvmTr to enable inspection mode execution with an associated Inspector type
* **InspectorHandler**: Extends Handler with inspection-enabled execution paths that make Inspector callbacks
* **Inspector**: User-implementable trait for EVM inspection/tracing