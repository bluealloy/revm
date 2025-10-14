# Persistent Warming Usage Guide

## Overview

Persistent warming allows EVM state access patterns (account loads, storage reads) to persist across multiple transactions for the lifetime of the EVM instance. This means the first access is COLD (expensive) and subsequent accesses are WARM (cheap).

## Key Design

- **Single HashMap**: Tracks `Address -> HashSet<StorageKey>`
- **Address warming**: Implicit - if address exists as key in map, it's warm
- **Storage warming**: Explicit - if storage key exists in the set for that address, it's warm
- **Empty set**: An address with an empty `HashSet` means the address is warm but no storage slots are warmed
- **Lifetime**: Cache lives for the lifetime of the EVM instance (never explicitly cleared)
- **Block boundaries**: In practice, a fresh EVM instance is created per block (e.g., in Reth), so the cache is naturally cleared between blocks

## Usage

### Basic Block Execution

```rust
use revm::*;

fn execute_block(transactions: Vec<Transaction>) -> EvmState {
    let ctx = Context::mainnet().with_db(your_database);
    let mut evm = ctx.build_mainnet();

    // Enable persistent warming before first transaction
    evm.journal_mut().enable_persistent_warming();

    for tx in transactions {
        let result = evm.transact(tx)?;
        // Note: transact() calls finalize() internally but the warming cache persists!
        // Commit the state changes to the database
        your_database.commit(result.state);
    }

    // At the end of the block, the EVM instance is dropped and the warming cache
    // is automatically cleaned up. For the next block, create a fresh EVM instance.

    your_database.finalize_block()
}
```

### Multi-Block Execution

```rust
fn execute_blocks(blocks: Vec<Block>) {
    for block in blocks {
        // Create a fresh EVM instance per block
        let ctx = Context::mainnet().with_db(your_database.clone());
        let mut evm = ctx.build_mainnet();

        // Enable persistent warming
        evm.journal_mut().enable_persistent_warming();

        for tx in block.transactions {
            let result = evm.transact(tx)?;
            your_database.commit(result.state);
        }

        // EVM instance (and warming cache) dropped here automatically
    }
}
```

### Mixing Per-Transaction and Persistent Modes

```rust
// Per-transaction warming (default) - each tx starts cold
evm.transact(tx1)?;  // Cold
evm.transact(tx2)?;  // Cold again

// Enable persistent warming
evm.journal_mut().enable_persistent_warming();
evm.transact(tx3)?;  // Cold (first access)
evm.transact(tx4)?;  // Warm! (already accessed in tx3)

// Disable if needed (rarely useful in practice)
evm.journal_mut().disable_persistent_warming();
evm.transact(tx5)?;  // Cold (per-tx warming mode)
```

## Gas Cost Behavior

### Without Persistent Warming (Default)
```
3 transactions accessing address 0xABCD:

Tx 1: Load 0xABCD → 2600 gas (COLD)
Tx 2: Load 0xABCD → 2600 gas (COLD) ❌
Tx 3: Load 0xABCD → 2600 gas (COLD) ❌

Total: 7800 gas
```

### With Persistent Warming
```
3 transactions accessing address 0xABCD:

Tx 1: Load 0xABCD → 2600 gas (COLD)
Tx 2: Load 0xABCD → 100 gas (WARM) ✅
Tx 3: Load 0xABCD → 100 gas (WARM) ✅

Total: 2800 gas (saves 5000 gas!)
```

## Implementation Details

### What Gets Warmed
- **Account loads**: Any call to `load_account`, `load_account_code`
- **Storage reads**: Any `SLOAD` operation
- **Storage writes**: Any `SSTORE` operation (warms the slot)

### What Doesn't Get Warmed
- **Precompiles**: Always warm (tracked separately in `WarmAddresses`)
- **Coinbase**: Warm only within transaction (cleared on `commit_tx`)

### Interaction with transaction_id
- `transaction_id` still increments on `commit_tx()` ✅
- EIP-6780 selfdestruct logic works correctly ✅
- Account cleanup (selfdestructed accounts) works correctly ✅
- Persistent warm cache is checked **before** per-transaction warming

## Performance

### Memory Usage
- **Per warmed address**: ~56 bytes (HashMap entry + empty HashSet)
- **Per warmed slot**: ~32 bytes (StorageKey in HashSet)
- **Typical block**: ~1000 addresses, ~5000 slots = ~300 KB

### CPU Cost
- **HashSet lookup**: ~20-30ns
- **Negligible** compared to gas savings

## Example: Testing Persistent Warming

```rust
#[test]
fn test_persistent_warming() {
    let mut evm = /* setup */;
    let addr = address!("0x1234...");

    evm.journal_mut().enable_persistent_warming();

    // Tx 1: First access is COLD
    let result1 = evm.transact(tx_accessing_addr)?;
    assert_eq!(result1.gas_used, base_cost + 2600);

    // Tx 2: Second access is WARM (warming persists)
    let result2 = evm.transact(tx_accessing_addr)?;
    assert_eq!(result2.gas_used, base_cost + 100);

    // finalize() does NOT clear warming cache
    evm.finalize();

    // Tx 3: Still WARM (cache persists for lifetime of EVM)
    let result3 = evm.transact(tx_accessing_addr)?;
    assert_eq!(result3.gas_used, base_cost + 100);
}
```

## When to Use

✅ **Use persistent warming when:**
- Building/validating blocks in an L2 environment
- Creating an EVM where warming persists across transactions
- Implementing custom gas accounting
- Each block gets a fresh EVM instance (natural cache cleanup)

❌ **Don't use when:**
- Executing single isolated transactions
- Simulating Ethereum mainnet behavior (EIP-2929 is per-transaction)
- Memory is constrained (the cache adds ~300KB per block)

## Important Notes

- **Not Ethereum Mainnet**: Persistent warming is NOT part of Ethereum mainnet consensus (EIP-2929 is per-transaction)
- **L2-specific feature**: This is an optimization/feature for L2s or custom EVM implementations
- **Cache lifetime**: The cache lives for the lifetime of the EVM instance and is never explicitly cleared
- **Reth integration**: Reth creates a fresh EVM instance per block, so the cache is naturally cleaned up between blocks
