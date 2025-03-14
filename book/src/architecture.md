# Architecture

REVM is a modular EVM implementation that follows Ethereum mainnet network and implements changes through hardforks defined in the [Ethereum execution specs](https://github.com/ethereum/execution-specs).

It has two main usages:
1. Execute mainnet transaction
2. As a EVM Framework, to create new EVM variant for other EVM like chains.

For a list of Revm crates and their usage, see the [Crates](./architecture/crates.md) section.

REVM is built around several key traits that enable customizable EVM functionality. The core execution traits include:

* **EvmTr**: The core EVM trait that provides access to the main EVM components:
  - Context - Environment and state access
  - Instructions - EVM opcode implementations
  - Precompiles - Built-in contract implementations
  - Interpreter execution

* **ContextTr**: Accessed through EvmTr, defines the execution environment including:
  - Block and transaction data
  - Database for account/storage access
  - Journal for tracking state changes and handling reverts

* **Handler**: Implements the core execution logic, taking an EvmTr implementation. The default implementation follows Ethereum consensus.

* **Frame**: Associated type of Handler containing call execution data and logic. The default EthFrame implementation handles standard Ethereum calls.

Additionally, REVM provides inspection capabilities through these traits:

* **InspectorEvmTr**: Extends EvmTr to enable inspection mode execution with an associated Inspector type

* **InspectorHandler**: Extends Handler with inspection-enabled execution paths that make Inspector callbacks

* **Inspector**: User-implementable trait for EVM inspection/tracing

The [my-evm example](https://github.com/bluealloy/revm/tree/rakita/my_evm/examples/my_evm) demonstrates how to implement these traits.

# 