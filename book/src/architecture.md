# Architecture and API

REVM is a flexible implementation of the Ethereum Virtual Machine (EVM). It follows the rules of the Ethereum mainnet and stays up to date with changes through hardforks as defined in the official [Ethereum 
execution specs](https://github.com/ethereum/execution-specs).

You can use REVM in two main ways:
1. Run regular Ethereum transactions using an Execution API
2. Create your own custom version of the EVM (for Layer 2 solutions or other chains) using the EVM framework

To see usage examples you can check the [examples folder](https://github.com/bluealloy/revm/tree/main/examples). Other than documentation, examples are the main resource to see and learn about Revm.

The main [`revm`](https://crates.io/crates/revm) library combines all crates into one package and reexports them, standalone libraries are useful if there is a need to import functionality with smaller scope. You can see an overview of all revm crates in the [crates folder](https://github.com/bluealloy/revm/tree/main/crates).

REVM works in `no_std` environments which means it can be used in zero-knowledge virtual machines (zkVMs) and it is the standard library in that use case. It also has very few external dependencies.

## Key Components

The main components of REVM are:

- **revm**: Main crate that combines all other crates
- **revm-primitives**: Basic types like addresses, numbers, and constants
- **revm-interpreter**: Opcode implementations and execution engine
- **revm-context**: Execution context, environment, and state management
- **revm-handler**: Execution flow control and frame management
- **revm-database**: Interface for accessing blockchain state
- **revm-precompile**: Built-in Ethereum contracts
- **revm-inspector**: Tracing and debugging tools

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
* `Database` is an interface that allows fetching external data that is needed at runtime. That data are account, storage and bytecode. When loaded they are stored in [`Journal`](https://docs.rs/revm-context-interface/latest/revm_context_interface/journaled_state/trait.JournalTr.html) 

REVM provides four ways to execute transactions through traits (API):

* `transact(tx)` and `replay()` are function of [`ExecuteEvm`](https://docs.rs/revm-handler/latest/revm_handler/api/trait.ExecuteEvm.html) trait that allow execution transactions. They return the status of execution with reason, changed state and in case of failed execution an error.
* `transact_commit(tx)` and `replay_commit()` are part of [`ExecuteCommitEvm`](https://docs.rs/revm-handler/latest/revm_handler/api/trait.ExecuteCommitEvm.html) that internally commits the state diff to the database and returns status of execution. Database is required to support `DatabaseCommit` trait.
* `inspect()`, `inspect_replay(tx)` and a few others are part of [`InspectEvm`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectEvm.html) trait that allow execution with inspection. This is how tracers are called.
* `inspect_commit()`,`inspect_replay_commit(tx)` are part of the [`InspectCommitEvm`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectCommitEvm.html) trait that extends `InspectEvm` to allow committing state diff after tracing.

For inspection API to be enabled, [`Evm`](https://docs.rs/revm-context/1.0.0/revm_context/evm/struct.Evm.html) needs to be created with inspector.

```rust,ignore
let mut evm = Context::mainnet().with_block(block).build_mainnet().with_inspector(inspector);
let _ = evm.inspect_tx(tx);
```

## Database Interface

The Database trait is how REVM gets blockchain data during execution. You can implement your own database or use existing ones. REVM provides several database implementations:

* **InMemoryDB**: Stores everything in memory, good for testing
* **CacheDB**: Wraps another database and caches data for better performance
* **AlloyDB**: Connects to Ethereum nodes using Alloy
* **DatabaseComponents**: Lets you split state and block hash access into separate parts

The DatabaseComponents pattern is useful when you want to handle state and block hashes differently. For example, you might want to:
- Store state in one database and block hashes in another
- Use different caching strategies for each type of data
- Have read-only access to block hashes but read-write access to state

Here's a simple example:

```rust,ignore
use revm::database_components::{DatabaseComponents, State, BlockHash};

let db = DatabaseComponents {
    state: MyStateDB,
    block_hash: MyBlockHashDB,
};

let mut evm = Context::mainnet().with_db(db).build();
```

# EVM Framework

REVM is designed to be customizable. You can create your own EVM variant by:
1. Creating custom handlers to change execution behavior
2. Adding new opcodes or modifying existing ones
3. Implementing custom precompiles
4. Changing gas calculations

## Creating a Custom EVM

Here's a basic example of creating your own EVM:

```rust,ignore
use revm::{Context, Evm, Handler, ExecuteEvm};

// Define your custom EVM type
pub struct MyEvm<CTX, INSP>(
    pub Evm<CTX, INSP, EthInstructions, EthPrecompiles, EthFrame>
);

// Create your custom handler
pub struct MyHandler<EVM> {
    _phantom: PhantomData<EVM>,
}

// Implement the Handler trait to customize behavior
impl<EVM> Handler for MyHandler<EVM> {
    // Override methods to customize execution
}
```

## Real-World Examples

To learn how to build your own custom EVM:
- **[example-my-evm](https://github.com/bluealloy/revm/tree/main/examples/my_evm)**: Basic custom EVM implementation
- **[op-revm](https://github.com/bluealloy/revm/tree/main/crates/op-revm)**: Optimism's Layer 2 EVM variant
- **[custom-opcodes](https://github.com/bluealloy/revm/tree/main/examples/custom_opcodes)**: How to add new opcodes
- **[erc20-gas](https://github.com/bluealloy/revm/tree/main/examples/erc20_gas)**: Pay gas with ERC20 tokens

Each trait needed to build custom EVM has detailed documentation explaining how it works and is worth reading.

In summary, REVM is built around several key traits that enable customizable EVM functionality. The core traits include:

* [`EvmTr`](https://docs.rs/revm-handler/latest/revm_handler/evm/trait.EvmTr.html): The core EVM trait that provides access to `Context`, `Instruction`, `Precompiles`:
* [`ContextTr`](https://docs.rs/revm-context-interface/latest/revm_context_interface/context/trait.ContextTr.html): Accessed through EvmTr, defines the execution environment including Tx/Block/Journal/Db.
* [`Handler`](https://docs.rs/revm-handler/latest/revm_handler/handler/trait.Handler.html): Implements the core execution logic, taking an EvmTr implementation. The default implementation follows Ethereum consensus.

And traits that provide support for inspection and tracing:

* [`InspectorEvmTr`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.InspectorEvmTr.html): Extends EvmTr to enable inspection mode execution with an associated [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) type
* [`InspectorHandler`](https://docs.rs/revm-inspector/latest/revm_inspector/handler/trait.InspectorHandler.html): Extends Handler with inspection-enabled execution paths that make [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html) callbacks
* [`Inspector`](https://docs.rs/revm-inspector/latest/revm_inspector/trait.Inspector.html): User-implementable trait for EVM inspection/tracing
