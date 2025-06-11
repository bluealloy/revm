# Custom EVM Implementation Example: MyEvm

This example demonstrates how to create a custom EVM variant that modifies core behavior,
specifically by disabling the beneficiary reward mechanism.

## Core Components

To implement a custom EVM variant, two key components are needed:

1. A custom EVM struct ([`crate::MyEvm`] in [`crate::evm`]) that implements [`revm::handler::EvmTr`]
2. A custom handler ([`MyHandler`]) in [`crate::handler`] that controls execution behavior and implements [`revm::handler::Handler`]

Basic usage after implementing these two components:
```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet(), ());
let _res = MyHandler::default().run(&mut my_evm);
```

## Adding Inspector Support

To enable transaction inspection capabilities, implement two additional traits:

- [`revm::inspector::InspectorEvmTr`] on [`MyEvm`]
- [`revm::inspector::InspectorHandler`] on [`MyHandler`]

This allows integration with [`revm::Inspector`] for transaction tracing:

```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
let _res = MyHandler::default().inspect_run(&mut my_evm);
```

## High-Level Execution APIs

The example includes several trait implementations in [`crate::api`] that provide
convenient high-level interfaces:

### [`revm::ExecuteEvm`]
Provides a simplified interface that abstracts away handler complexity:

```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet(), ());
// Execute a new transaction
let _result_and_state = my_evm.transact(TxEnv::default());
// Replay the last transaction
let _res_and_state = my_evm.replay();
```

### [`revm::ExecuteCommitEvm`]
Extends [`revm::ExecuteEvm`] with database commit functionality. Requires the database
to implement [`revm::DatabaseCommit`]:

```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet().with_db(InMemoryDB::default()), ());
let _res = my_evm.transact_commit(TxEnv::default());
```

### [`revm::InspectEvm`]
Extends [`revm::ExecuteEvm`] with inspection methods that allow monitoring execution
without committing changes:

```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
// Inspect without committing
let _res = my_evm.inspect_replay();
// Inspect and commit
let _res = my_evm.inspect_commit_replay();
```

### [`revm::SystemCallEvm`]
Allows executing system transaction, only input needed is system contract add address and input
Validation and pre-execution and most of post execution phases of ordinary transact flow will be skipped.

System calls are needed for inserting of fetching data on pre or post block state.

```rust,ignore
let mut my_evm = MyEvm::new(Context::mainnet(), revm::inspector::NoOpInspector);
// System call with given input to system contract address.
let _res = my_evm.transact_system_call(bytes!("0x0001"), address!("f529c70db0800449ebd81fbc6e4221523a989f05"));
```
