# Custom Precompile with Journal Access Example

This example demonstrates how to create a custom precompile for REVM that can access and modify the journal (state).

## Overview

The example shows:
1. How to create a custom precompile provider that extends the standard Ethereum precompiles
2. How to implement a precompile that can read from and write to the journaled state
3. How to modify account balances and storage from within a precompile

## Key Components

### CustomPrecompileProvider

A custom implementation of the `PrecompileProvider` trait that:
- Extends the standard Ethereum precompiles (`EthPrecompiles`)
- Adds a custom precompile at address `0x0000000000000000000000000000000000000100`
- Delegates to standard precompiles for all other addresses

### Custom Precompile Functionality

The precompile at `0x0100` supports two operations:

1. **Read Storage** (empty input data):
   - Reads a value from storage slot 0
   - Returns the value as output
   - Gas cost: 2,100

2. **Write Storage** (32 bytes input):
   - Stores the input value to storage slot 0
   - Transfers 1 wei from the precompile to the caller
   - Gas cost: 41,000 (21,000 base + 20,000 for SSTORE)

### Journal Access

The example demonstrates how to access the journal from within a precompile:

```rust
// Reading storage
let value = context
    .journal_mut()
    .sload(address, key)
    .map_err(|e| PrecompileError::Other(format!("Storage read failed: {:?}", e)))?
    .data;

// Writing storage
context
    .journal_mut()
    .sstore(address, key, value)
    .map_err(|e| PrecompileError::Other(format!("Storage write failed: {:?}", e)))?;

// Transferring balance
context
    .journal_mut()
    .transfer(from, to, amount)
    .map_err(|e| PrecompileError::Other(format!("Transfer failed: {:?}", e)))?;
```

## Usage

To use this custom precompile in your application:

```rust
use revm::handler::Handler;

// Create a handler with the custom precompile provider
let handler = Handler::mainnet()
    .with_precompiles(CustomPrecompileProvider::new_with_spec(SpecId::CANCUN));

// Use the handler to build your EVM instance
// The precompile will be available at address 0x0100
```

## Running the Example

```bash
cargo run -p custom_precompile_journal
```

The example will print information about the custom precompile implementation and how to integrate it into a REVM application.