# Custom Precompile with Journal Access Example

This example demonstrates how to create a custom precompile for REVM that can access and modify the journal (state), integrated into a custom EVM implementation similar to MyEvm.

## Overview

The example shows:
1. How to create a custom precompile provider that extends the standard Ethereum precompiles
2. How to implement a precompile that can read from and write to the journaled state
3. How to modify account balances and storage from within a precompile
4. How to integrate custom precompiles into a custom EVM implementation
5. How to create handlers for transaction execution

## Architecture

### CustomPrecompileProvider

A custom implementation of the `PrecompileProvider` trait that:
- Extends the standard Ethereum precompiles (`EthPrecompiles`)
- Adds a custom precompile at address `0x0000000000000000000000000000000000000100`
- Delegates to standard precompiles for all other addresses
- Implements journal access for storage and balance operations

### CustomEvm

A custom EVM implementation that:
- Wraps the standard REVM `Evm` struct with `CustomPrecompileProvider`
- Follows the same pattern as the MyEvm example
- Maintains full compatibility with REVM's execution model
- Supports both regular and inspector-based execution

### CustomHandler

A handler implementation that:
- Implements the `Handler` trait for transaction execution
- Supports both `Handler` and `InspectorHandler` traits
- Can be used with `handler.run(&mut evm)` for full transaction execution

## Custom Precompile Functionality

The precompile at `0x0100` supports two operations:

1. **Read Storage** (empty input data):
   - Reads a value from storage slot 0
   - Returns the value as output
   - Gas cost: 2,100

2. **Write Storage** (32 bytes input):
   - Stores the input value to storage slot 0
   - Transfers 1 wei from the precompile to the caller as a reward
   - Gas cost: 41,000 (21,000 base + 20,000 for SSTORE)

## Journal Access Patterns

The example demonstrates how to access the journal from within a precompile:

```rust
// Reading storage
let value = context
    .journal_mut()
    .sload(address, key)
    .map_err(|_| PrecompileError::StorageOperationFailed)?
    .data;

// Writing storage
context
    .journal_mut()
    .sstore(address, key, value)
    .map_err(|_| PrecompileError::StorageOperationFailed)?;

// Transferring balance
context
    .journal_mut()
    .transfer(from, to, amount)
    .map_err(|_| PrecompileError::TransferFailed)?;

// Incrementing balance
context
    .journal_mut()
    .balance_incr(address, amount)
    .map_err(|_| PrecompileError::BalanceOperationFailed)?;
```

## Usage

To use this custom EVM in your application:

```rust
use custom_precompile_journal::{CustomEvm, CustomHandler};
use revm::{context::Context, inspector::NoOpInspector, MainContext};

// Create the custom EVM
let context = Context::mainnet().with_db(db);
let mut evm = CustomEvm::new(context, NoOpInspector);

// Create the handler
let handler = CustomHandler::<CustomEvm<_, _>>::default();

// Execute transactions
let result = handler.run(&mut evm);
```

## Safety Features

- **Static call protection**: Prevents state modification in view calls
- **Gas accounting**: Proper gas cost calculation and out-of-gas protection
- **Error handling**: Comprehensive error types and result handling
- **Type safety**: Full Rust type safety with generic constraints

## Running the Example

```bash
cargo run -p custom_precompile_journal
```

The example will demonstrate the custom EVM architecture and show how the various components work together to provide journal access functionality within precompiles.

## Integration with Existing Code

This example extends the op-revm pattern and demonstrates how to:
- Create custom precompile providers that can access the journal
- Integrate custom precompiles into REVM's execution model
- Maintain compatibility with existing REVM patterns and interfaces
- Build custom EVM variants similar to MyEvm but with enhanced precompile capabilities