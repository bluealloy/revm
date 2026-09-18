# Block Access List Construction

Use `revm::state::bal::BalBuilder`, or a database configured with
`with_bal_builder()`, to generate an EIP-7928 BAL from execution results.
`BalState::commit` and `commit_one` feed this builder before the underlying
database receives each result. `BalDatabase` applies this ordering for both
`DatabaseCommit::commit` and `commit_iter`.

## Originals and net changes

An execution result's `Account::original_info()` and
`EvmStorageSlot::original_value` describe the state before that execution. The
journal refreshes account originals on reload and storage originals across
transaction IDs. Multiple separately finalized system calls can share one
`block_access_index`, so their originals need not describe the start of that
index.

For example, two commits can contain `original=0, present=42` followed by
`original=42, present=0`. The database ends at zero. Under
[EIP-7928 storage rules](https://eips.ethereum.org/EIPS/eip-7928#storage), the BAL
must compare that final zero with the index's starting zero. With no earlier
write to that slot, it belongs in `storage_reads`, with no `storage_changes`.
If the second commit only reads 42, the index's write of 42 must remain.
Restoring a starting value here does not mean an EVM `REVERT`.

The builder captures the first original account values and the first original
value for each slot encountered at an index. It preserves them while applying
later present values. Fixed index-start originals and originals rebased after
each commit are both supported. Every result must be submitted in execution
order, including accessed unchanged slots, and indices must be nondecreasing.
Changed code must include its bytecode; an unchanged account read may provide
only its code hash.

`Bal`, `AccountBal`, `AccountInfoBal`, `StorageBal`, and `BalWrites` remain
output structures. Their low-level update methods require fixed index-start
originals; they cannot reconstruct a missing baseline from the first write's
final value. The limited unchanged-update guard in `BalWrites` does not provide
general aggregation of rebased writes. Use `BalBuilder` for that contract.

## Lifecycle and cost

The builder owns its output and exposes only an immutable output view. This
prevents output mutations from silently invalidating saved baselines. Advancing
the index releases the previous index's baseline entries. Extra baseline memory
is proportional to the accounts and slots visited at the current index, with
an additional map lookup per account/slot; no performance benchmark is claimed.

`BalBuilder::clone` and its serde representation retain both output and
baselines, allowing construction to continue. `into_bal` discards construction
state; the resulting `Bal` retains the existing serialized format and can be
converted to canonical Alloy output. A final BAL alone cannot resume a builder
at its last index because it does not contain the initial values.

`BalBuilder::clear` clears output and baselines. When using `BalState`, finish a
block with `take_built_bal` or `take_built_alloy_bal`, then use
`with_bal_builder` to enable a fresh builder. Taking output also resets the
index. Merely resetting the index does not begin a new block. Decreasing indices
on an existing builder panic rather than mixing two blocks.

See the [migration guide](../../MIGRATION_GUIDE.md) for the builder field type,
in-progress serde format and `const` API changes.

## Verification and Foundry integration

The local regressions cover actual `BalDatabase<InMemoryDB>` commits and final
Alloy output, both commit APIs, fixed/rebased originals, write/read and
write/restore sequences, earlier-index history, account fields, lazy code
loading, clone, serde continuation and clear/reuse. A bounded exhaustive test
compares six-commit sequences with their index endpoints. Handler tests also
execute real `SSTORE`/`SLOAD` system calls, finalize and commit them separately,
and assert each result's original/present values as well as the database and BAL.
Local selfdestruct
handling also clears slots observed by earlier submissions at the same index;
this does not claim full historical-fork storage-wipe validation.

On 2026-09-18, [revm #3919](https://github.com/bluealloy/revm/pull/3919) remained
at `3668689b6c6d33eddd6fa9f6cc4d94808bef7496` (base
`bd4f8583d42099f0e41f0cbfcffabc28e703402e`). It explicitly fixes write/read
preservation while excluding general rebased aggregation. The real database
regression for `0 -> 42 -> 0` fails with that patch alone and passes with this
builder.

[Foundry #16917](https://github.com/foundry-rs/foundry/pull/16917) remained at
`e91aba93036d3dc79f02cfc1514da644490d204e`. Its
`AnvilCacheDB::commit` feeds `BalState` before committing to `CacheDB`. Separate
system calls finalize and reload the committed database, producing rebased
originals. Two pre-block system calls share index 0, transactions use 1..N, and
the current Amsterdam post-block sequence can submit four separate system
calls at N+1 (withdrawals, consolidations, builder deposits and builder exits).
Each candidate block starts a new builder and takes its output once.

This builder supports that incremental submission contract. Foundry need not
also aggregate pre/post phases after adopting the fix. Phase aggregation that
retains earliest originals and final values remains an alternative if using a
revm version without this builder.

The Anvil end-to-end proof is still required. Override system contracts to
access a shared helper slot and assert independent expected BAL contents:

1. Pre-block write 42 then read: retain index-0 write 42.
2. Pre-block write 42 then restore 0: emit only the slot's read access.
3. Post-block write 42 then restore 0, followed by more reads: retain no net
   write, with post index N+1 tested for both empty and nonempty blocks.
4. A transaction writes 7, then post-block writes 42 and restores 7: preserve
   only the transaction's earlier write; do not also list the slot as a read.
5. Consecutive blocks with different starting values: ensure no baseline leaks
   from one block into the next.

For each fixture, first check the database value and expected
`storage_changes`/`storage_reads`; then compare the expected BAL's encoding and
hash with typed RPC, raw RPC and the header. Agreement between those three
outputs alone does not establish EIP compliance. This local work does not run
Anvil or prove that the default Ethereum system contracts trigger the overlap.
The separate Foundry hardfork-selection issue referenced as #16907 is outside
this aggregation fix.
