
# v107 tag (revm v38.0.0)

* `Handler::first_frame_input` ([#3578](https://github.com/bluealloy/revm/pull/3578)): `init_and_floor_gas: &InitialAndFloorGas` param replaced by `reservoir: u64`. Compute via `InitialAndFloorGas::initial_gas_and_reservoir(tx_gas_limit, tx_gas_limit_cap, is_eip8037) -> (gas_limit, reservoir)`.
  * `create_init_frame` and `CreateInputs::new` gained a trailing `reservoir: u64` param.
* `Handler::validate_against_state_and_deduct_caller` ([#3577](https://github.com/bluealloy/revm/pull/3577)): gained `&mut InitialAndFloorGas` param.
* `PrecompileOutput::gas_refunded: i64` field re-added ([#3574](https://github.com/bluealloy/revm/pull/3574)). Breaks struct literal constructors.

# v106 tag (revm v37.0.0)

* EIP-8037 state gas support ([#3406](https://github.com/bluealloy/revm/pull/3406)). Gas is now split into regular gas and state gas tracked via a reservoir. State gas draws from the reservoir first and spills into regular gas when exhausted. This affects gas accounting across the entire stack.
  * `Gas::spent()` and `record_cost()` deprecated. Use `total_gas_spent()`, `record_regular_cost()`, `record_state_cost()`.
  * New `Gas::new_with_regular_gas_and_reservoir(limit, reservoir)` constructor for creating child frame gas with reservoir.
  * `ResultGas::new()` deprecated (takes 3 params now). Use `ResultGas::new_with_state_gas(total_gas_spent, refunded, floor_gas, state_gas_spent)`.
  * `ResultGas::spent()`/`used()` deprecated. Use `total_gas_spent()`/`tx_gas_used()`. New accessors: `state_gas_spent()`, `block_regular_gas_used()`, `block_state_gas_used()`.
  * `ExecutionResult::gas_used()` deprecated, use `gas().tx_gas_used()`.
  * `InitialAndFloorGas::initial_gas` field renamed to `initial_total_gas`. New fields added: `initial_state_gas` (state gas portion) and `eip7702_reservoir_refund` (refund for existing EIP-7702 authorities).
  * `CallInputs` gained `reservoir: u64` field for propagating state gas from parent to child frames.
  * `Interpreter::clear` takes 7 params (was 6), `EthFrame::clear` takes 11 (was 10) — both gained reservoir parameter.
  * `Host` and `Cfg` traits gained required method `is_amsterdam_eip8037_enabled() -> bool`. Must be implemented on any custom types.
  * `Handler::pre_execution` and `apply_eip7702_auth_list` gained `&mut InitialAndFloorGas` param to write back state gas and refund info.
  * `Handler::first_frame_input` gained `&InitialAndFloorGas` param to compute the reservoir for the first frame.
  * `validate_initial_tx_gas` takes 5 params (was 3) — added `is_amsterdam_eip8037_enabled` and `tx_gas_limit_cap`.
  * `create_init_frame` takes 2 params (was 3), gained `CTX: ContextTr` generic. Reservoir is set by `first_frame_input` after creation.
* `PrecompileError` restructured ([#3496](https://github.com/bluealloy/revm/pull/3496), [#3502](https://github.com/bluealloy/revm/pull/3502)). All specific error variants removed (`OutOfGas`, `Blake2*`, `Bn254*`, `Bls12381*`, `Kzg*`, `Secp256k1*`, `Other`, etc.). `PrecompileError` is now only for fatal/unrecoverable errors with two variants: `Fatal(String)` and `FatalAny(AnyError)`. Non-fatal failures (OOG, invalid input) are now expressed via `PrecompileStatus::Halt` in `PrecompileOutput`.
  * `PrecompileError::other()`, `other_static()`, `is_oog()` removed.
* `PrecompileOutput` fields changed: `gas_refunded` and `reverted` removed, new fields `status: PrecompileStatus`, `state_gas_used`, `reservoir`.
  * `PrecompileOutput::new(gas_used, bytes, reservoir)` — added reservoir param.
  * `PrecompileOutput::new_reverted()`/`reverted()` removed, use `PrecompileOutput::revert()` instead.
  * `PrecompileFn` signature changed from `fn(&[u8], u64)` to `fn(&[u8], u64, u64)` — added reservoir param.
  * `Precompile::execute()` also gained `reservoir: u64` param.
* `EVMError::CustomAny(AnyError)` variant added ([#3502](https://github.com/bluealloy/revm/pull/3502)). Update exhaustive matches. `EVMError` also lost `UnwindSafe`/`RefUnwindSafe` auto traits.
* `SELFDESTRUCT_LOG_TOPIC` constant removed from `revm-primitives` ([#3438](https://github.com/bluealloy/revm/pull/3438)).
* `StateBuilder::with_background_transition_merge` removed (was a no-op) ([#3510](https://github.com/bluealloy/revm/pull/3510)). Simply remove any calls.
* `CfgEnv::set_spec` deprecated ([#3550](https://github.com/bluealloy/revm/pull/3550)). Use the `spec` field directly.
* `MemoryGas::record_new_len` and `memory_gas` function removed ([#3534](https://github.com/bluealloy/revm/pull/3534)).
* Inspector `frame_start` and `frame_end` methods added with default impls ([#3518](https://github.com/bluealloy/revm/pull/3518)). Hooks into frame lifecycle; no action needed unless you want to use them.

# v104 tag (revm v34.1.0)

## Bytecode flattened from enum to struct ([#3375](https://github.com/bluealloy/revm/pull/3375))

`Bytecode` is no longer an enum with `LegacyRaw`, `LegacyAnalyzed`, `Eip7702` variants. It is now an opaque struct wrapping `Arc<BytecodeInner>` (size reduced from 16 to 8 bytes).

* Pattern matching on `Bytecode` variants is no longer possible. Use accessor methods instead:
  * `bytecode.kind()` returns `BytecodeKind` (`LegacyAnalyzed` or `Eip7702`).
  * `bytecode.is_legacy()`, `bytecode.is_eip7702()` for type checks.
  * `bytecode.eip7702_address()` instead of matching `Bytecode::Eip7702(...)`.
  * `bytecode.legacy_jump_table()` instead of matching `Bytecode::LegacyAnalyzed(...)`.
* Constructors:
  * `Bytecode::new_legacy(raw_bytes)` — replaces `Bytecode::LegacyRaw(...)` (analysis happens on creation).
  * `Bytecode::new_analyzed(bytecode, original_len, jump_table)` — replaces `Bytecode::LegacyAnalyzed(Arc::new(...))`.
  * `Bytecode::new_eip7702(address)` — replaces `Bytecode::Eip7702(Arc::new(...))`.
  * `Bytecode::new_raw(bytes)` — auto-detects EIP-7702 delegation.
* `LegacyAnalyzedBytecode`, `LegacyRawBytecode`, `Eip7702Bytecode` structs are no longer publicly exported.
* `JumpTable::from_bytes` and `JumpTable::from_bytes_arc` removed (Arc is now internal to Bytecode).
* The `legacy` module is removed from public API.

## `ExecutionResult`: new `ResultGas` struct replaces `gas_used`/`gas_refunded` ([#3413](https://github.com/bluealloy/revm/pull/3413))

All three `ExecutionResult` variants now use a `gas: ResultGas` field instead of separate `gas_used: u64` / `gas_refunded: u64`:

```rust
// Before:
ExecutionResult::Success { reason, gas_used, gas_refunded, logs, output }
ExecutionResult::Revert { gas_used, output }
ExecutionResult::Halt { reason, gas_used }

// After:
ExecutionResult::Success { reason, gas: ResultGas, logs, output }
ExecutionResult::Revert { gas: ResultGas, logs, output }
ExecutionResult::Halt { reason, gas: ResultGas, logs }
```

`ResultGas` provides:
* `used()` — equivalent to old `gas_used` (accounts for EIP-7623 floor).
* `remaining()` — `limit - spent`.
* `inner_refunded()` — raw refund value.
* `final_refunded()` — 0 when floor gas is active, otherwise equals refunded.
* `spent_sub_refunded()` — `spent - refunded`.
* Construct with `ResultGas::new(limit, spent, refunded, floor_gas, intrinsic_gas)`.

The convenience method `ExecutionResult::gas_used()` still works (delegates to `gas().used()`).

## `ExecutionResult::Revert` and `Halt` now carry `logs` ([#3424](https://github.com/bluealloy/revm/pull/3424))

This is only relevant for revm variant ( Tempo ), ethereum does not contains logs on Halt or Revert.
Both `Revert` and `Halt` variants gained a `logs: Vec<Log>` field containing logs emitted before the revert/halt:

```rust
// Before:
ExecutionResult::Revert { gas, output }
ExecutionResult::Halt { reason, gas }

// After:
ExecutionResult::Revert { gas, logs, output }
ExecutionResult::Halt { reason, gas, logs }
```

* `logs()` and `into_logs()` now return logs from all variants, not just `Success`.

## EIP-161 state clear moved into journal finalize ([#3444](https://github.com/bluealloy/revm/pull/3444))

Pre-EIP-161 normalization is now handled by `JournalInner::finalize()` instead of the database layer.

* `CacheState::has_state_clear` field **removed**.
* `CacheState::set_state_clear_flag()` **removed**.
* `State::set_state_clear_flag()` **removed**.
* `StateBuilder::without_state_clear()` **removed**.
* `CacheState::new()` now takes **no parameters** (was `new(has_state_clear: bool)`).
* `CacheAccount::touch_create_pre_eip161()` **removed**.
* `AccountStatus::on_touched_created_pre_eip161()` **removed**.

```rust
// Before:
CacheState::new(true)
state_builder.without_state_clear()

// After:
CacheState::new()
// (no replacement needed — journal handles it)
```

## `JournalInner.spec` replaced by `JournalInner.cfg` ([#3395](https://github.com/bluealloy/revm/pull/3395))

`JournalInner`'s `spec` field is replaced by a `cfg: JournalCfg` struct that bundles spec with EIP-7708 config:
* Access spec via `journal.inner.cfg.spec` instead of `journal.inner.spec`.
* `JournalInner` also gained a `selfdestructed_addresses` field.
* `JournalCheckpoint` gained a `selfdestructed_i: usize` field.

## `Handler::execution_result` signature changed ([#3413](https://github.com/bluealloy/revm/pull/3413))

```rust
// Before:
fn execution_result(&mut self, evm, result) -> Result<ExecutionResult, Error>

// After:
fn execution_result(&mut self, evm, result, result_gas: ResultGas) -> Result<ExecutionResult, Error>
```

Also, `post_execution::output()` now takes a `ResultGas` parameter instead of computing gas internally.

## `EthPrecompiles` and `EthInstructions` no longer implement `Default` ([#3434](https://github.com/bluealloy/revm/pull/3434))

```rust
// Before:
EthPrecompiles::default()
EthInstructions::default()
EthInstructions::new_mainnet()  // deprecated

// After:
EthPrecompiles::new(spec)
EthInstructions::new_mainnet_with_spec(spec)
```

## Fixed-bytes hashmaps from alloy-core ([#3358](https://github.com/bluealloy/revm/pull/3358))

`HashMap<Address, _>` / `HashSet<Address>` replaced with `AddressMap<_>` / `AddressSet` throughout the codebase. Similarly `HashMap<B256, _>` → `B256Map<_>` and `HashMap<U256, _>` → `U256Map<_>`.

Affected trait signatures:
* `DatabaseCommit::commit()`: `HashMap<Address, Account>` → `AddressMap<Account>`.
* `JournalTr::warm_access_list()`: `HashMap<Address, HashSet<StorageKey>>` → `AddressMap<HashSet<StorageKey>>`.
* `JournalTr::warm_precompiles()`: `HashSet<Address>` → `AddressSet`.
* `JournalTr::precompile_addresses()`: returns `&AddressSet`.

Import these types from `revm::primitives` (re-exported from alloy-core).

## `BlockHashCache` replaces `BTreeMap` for block hashes ([#3299](https://github.com/bluealloy/revm/pull/3299))

`State::block_hashes` changed from `BTreeMap<u64, B256>` to `BlockHashCache` (O(1) ring buffer, 256 entries).

```rust
// Before:
state.block_hashes.get(&block_num)
StateBuilder::default().with_block_hashes(btree_map)

// After:
state.block_hashes.get(block_num)
StateBuilder::default().with_block_hashes(block_hash_cache)
```

## `apply_auth_list` gains `refund_per_auth` parameter ([#3366](https://github.com/bluealloy/revm/pull/3366))

```rust
// Before:
apply_auth_list(context, auth_list, journal)

// After:
apply_auth_list(context, auth_list, journal, refund_per_auth)
```

Use `gas_params.tx_eip7702_auth_refund()` for the default value (12500).

## New EIP features (Amsterdam hardfork)

### EIP-7843: SLOTNUM opcode ([#3340](https://github.com/bluealloy/revm/pull/3340))
* New opcode `SLOTNUM` (`0x4B`) gated behind `AMSTERDAM`.
* `BlockEnv` has a new `slot_num: u64` field (default `0`). Include when constructing `BlockEnv` literals.
* `Host` trait gained `fn slot_num(&self) -> U256`.
* `Block` trait gained `fn slot_num(&self) -> u64` with default returning `0`.

### EIP-7708: ETH transfers emit logs ([#3334](https://github.com/bluealloy/revm/pull/3334), [#3395](https://github.com/bluealloy/revm/pull/3395))
* When Amsterdam is active and value is non-zero, ETH transfers (CALL, CREATE, SELFDESTRUCT, tx value) emit EIP-7708 logs.
* `Cfg` trait gained `is_eip7708_disabled()` and `is_eip7708_delayed_burn_disabled()`.
* `JournalTr` trait gained `set_eip7708_config()`.
* Tests that check exact log counts on Amsterdam+ specs need updating.

### EIP-8024: DUPN, SWAPN, EXCHANGE opcodes ([#3223](https://github.com/bluealloy/revm/pull/3223))
* New opcodes at `0xE6`–`0xE8` gated behind `AMSTERDAM`, each with 1-byte immediates.
* New `InstructionResult::InvalidImmediateEncoding` variant — update exhaustive matches.

## Deprecated (still functional, warnings emitted)

* `JournalTr::caller_accounting_journal_entry()` and `nonce_bump_journal_entry()` ([#3367](https://github.com/bluealloy/revm/pull/3367)).
* `EthInstructions::new_mainnet()` — use `new_mainnet_with_spec(spec)` ([#3434](https://github.com/bluealloy/revm/pull/3434)).
* `ItemOrResult::map_frame` — use `map_item` ([#3320](https://github.com/bluealloy/revm/pull/3320)).

## Other notable changes

* Default hardfork updated to Osaka (Ethereum) and Jovian (Optimism) ([#3326](https://github.com/bluealloy/revm/pull/3326)).
* `ContextError` handling extracted into `take_error` helper ([#3312](https://github.com/bluealloy/revm/pull/3312)).
* `BlockHashCache` incorrectly returning zero for block 0 fixed ([#3319](https://github.com/bluealloy/revm/pull/3319)).
* `ResultGas::final_refunded()` corrected when floor gas is active ([#3450](https://github.com/bluealloy/revm/pull/3450)).
* EIP-161 state clear fix for empty Loaded/Changed accounts ([#3421](https://github.com/bluealloy/revm/pull/3421)).
* `CacheState::clear()` and `TransitionState::clear()` added ([#3390](https://github.com/bluealloy/revm/pull/3390)).
* `calc_linear_cost_u32` renamed to `calc_linear_cost` ([#3318](https://github.com/bluealloy/revm/pull/3318)).

# v103 tag (revm v34.0.0)

* BAL (EIP-7928) support added to Database implementations.
* `GasParams` is new struct where you can set dynamic opcode gas params. Initialized and can be found in cfg.
  * Gas calculation functions moved from `revm-interpreter` to be part of gas params.
  * Gas constants moved from `revm_interpreter`::gas to `revm_context_interface::cfg::gas`
* `CreateInputs` struct fields made private with accessor pattern.
  * Use `CreateInputs::created_address()` getter (now cached).
* `Host::selfdestruct` signature changed to support OOG on cold load for target account.
* Inspector `log` function renamed:
  * `Inspector::log` renamed to `log` and `log_full`.
  * `log_full` default impl calls `log`.
  * `log_full` has `Interpreter` input while `log` does not.
  * `log` is called in places where Interpreter is not found.
* `PrecompileError::Other` now contains `Cow<'static, str>` instead of `&'static str`.
  * Allows setting both `&'static str` (no perf penalty) and `String` if needed.
* `JournaledAccount` struct added for tracking account changes.
  * `JournalTr` functions that fetch account now return a ref.
  * New function `load_account_mut` returns `JournaledAccount`.
* `JournalTr::load_account_code` deprecated, renamed to `JournalTr::load_account_with_code`.
* `JournalTr::warm_account_and_storage` and `JournalTr::warm_account` removed.
  * Access list is now separate from the main Journal EvmState.
  * Use `JournalTr::warm_access_list` to import access list.
* Declarative macros `tri!`, `gas_or_fail!`, `otry!` removed from `revm-interpreter`.
* `MemoryGas` API signature changes.
* Removed deprecated methods including `into_plain_state`, `regenerate_hash`.
* `State.bal_state` field added (breaks struct literal constructors).
* `DatabaseCommitExt::drain_balances` and `increment_balances` added.
* First precompile error now bubbles up with detailed error messages.
  * New `PrecompileError` variants added.

# 102 tag ( revm v33.1.0)

No breaking changes

# 101 tag ( revm v33.1.0)

No breaking changes

# v100 tag (revm v33.0.0)

* Additionally to v99 version:
  * `Host::selfdestruct` function got changed to support oog on cold load for target account. 

# v99 tag ( revm v32.0.0 )

(yanked version)

* Added support for transmitting Logs set from precompiles.
   * `Inspector::log` function got renamed to `log` and `log_full`
   * `log_full` default impl will call `log`
   * difference is that `log_full` has `Interpreter` input while `log` does not
     and `log` will be called in places where Interpreter is not found.
* `PrecompileError` now contains `Other` as `Cow<'static, str>`
   * It allows setting both `&'static str` that is without perf penalty and `String` if needed.

# v98 tag

No breaking changes.

# v97 tag

No breaking changes.

# v96 tag

No breaking changes.

# v95 tag ( revm v31.0.0)

* Cfg added `memory_limit()` function
* `JournaledAccount` have been added that wraps account changes, touching and creating journal entry.
  * Past function that fetches account now return a ref and new function `load_account_mut` now return `JournaledAccount`
  * `JournalEntry` type is added to `JournalTr` so JournaledAccount can create it.
* `JournalTr::load_account_code` is deprecated/renamed to `JournalTr::load_account_with_code`
* `JournalTr::warm_account_and_storage` and `JournalTr::warm_account` are removed as access list is now separate from the main
  Journal EvmState. Function that imports access list to the Journal is `JournalTr::warm_access_list`

# v94 tag ( op-revm )

No breaking changes

# v93 tag ( op-revm )

No breaking changes

# v93 tag ( revm v30.1.0)

No breaking changes

# v92 tag ( revm v30.1.2)

No breaking changes

# v91 tag ( revm v30.1.1)

No breaking changes

# v90 tag ( revm v30.1.0)

* Removal of some deprecated functions. `into_plain_state`, `regenerate_hash` were deprecated few releases ago.

# v89 tag ( op-revm)

No breaking changes

# v88 tag (revm v30.0.0)

* `ContextTr`, `EvmTr` gained `all_mut()` and `all()` functions.
  * `InspectorEvmTr` got `all_inspector` and `all_mut_inspector` functions.
  * For custom Evm, you only need to implement `all_*` functions.
* `InspectorFrame` got changed to allow CustomFrame to be added.
  * If there is no need for inspection `fn eth_frame` can return None.
* `kzg-rs` feature and library removed. Default KZG implementation is now c-kzg.
  * for no-std variant, arkworks lib is used.
* Opcodes that load account/storage will not load item if there is not enough gas for cold load.
  * This is in preparation for BAL.
  * Journal functions for loading now have skip_cold_load bool.
* `libsecp256k1` parity library is deprecated and removed.
* `JumpTable` internal representation changed from `BitVec` to `Bytes`. No changes in API.
* `SpecId` enum gained new `Amsterdam` variant and `OpSpecId` gained `Jovian` variant.
* `SELFDESTRUCT` constant renamed to `SELFDESTRUCT_REFUND`.
* `FrameStack::push` and `FrameStack::end_init` marked as `unsafe` as it can cause UB.
* First precompile error is now bubble up with detailed error messages. New `PrecompileError` variants added.
* Batch execution errors now include transaction index.
* `CallInput` now contains bytecode that is going to be executed (Previously it had address).
  * This allows skipping double bytecode fetching.
* `InvalidTransaction` enum gained `Str(Cow<'static, str>)` variant for custom error messages.
* `calc_excess_blob_gas` and `calc_excess_blob_gas` removes as they are unused and not accurate for Prague.

# v86 tag (revm v29.0.0)

* `PrecompileWithAddress` is renamed to `Precompile` and it became a struct.
  * `Precompile` contains`PrecompileId`, `Address` and function.
  * The reason is adding `PrecompileId` as it is needed for fusaka hardfork

# v85 tag (revm v28.0.1) from v84 tag (revm v28.0.0)

Forward compatible version.

# v84 tag (revm v28.0.0) from v83 tag (revm v27.1.0)

* `SystemCallEvm` functions got renamed and old ones are deprecated. Renaming is done to align it with other API calls.
   * `transact_system_call_finalize` is now `system_call`.
   * `transact_system_call` is now `system_call_one`.
* `ExtBytecode::regenerate_hash` got deprecated in support for `get_or_calculate_hash` or `calculate_hash`.
* Precompiles:
  * Bn128 renamed to Bn254. https://github.com/ethereum/EIPs/pull/10029#issue-3240867404
* `InstructionResult` now starts from 1 (previous 0) for perf purposes.
* In `JournalInner` previous `precompiles`, `warm_coinbase_address` and `warm_preloaded_addresses` pub fields are now moved to `warm_addresses` to encapsulate addresses that are warm by default. All access list account are all loaded from database.


# v83 tag (revm v27.1.0) from v82 tag (revm v27.0.3)

* `ContextTr` gained `Host` supertrait.
  * Previously Host was implemented for any T that has ContextTr, this restricts specializations.
  https://github.com/bluealloy/revm/issues/2732
  * `Host` is moved to `revm-context-interface`
  * If you custom struct that implement `ContextTr` you would need to manually implement `Host` trait, in most cases no action needed.
* In `revm-interpreter`, fn `cast_slice_to_u256` was removed and `push_slice` fn is added to `StackTrait`.
* `PrecompileOutput` now contains revert flag.
  * It is safe to put to false.
* In `kzg` and `blake2` modules few internal functions were made private or removed.

# v80 tag (revm v27.0.0) -> v81 tag ( revm v27.0.1)

* Inspector fn `step_end` is now called even if Inspector `step` sets the action. Previously this was not the  case.
    * https://github.com/bluealloy/revm/pull/2687
    * this additionally fixes panic bug where `bytecode.opcode()` would panic in `step_end`

# v70 tag (revm v22.0.2) -> v71 tag ( revm v23.0.0)

* Removal of `EvmData`.
    * It got flattened and ctx/inspector fields moved directly to Evm, additional layering didn't have purpose.
* Merging of `Handler`'s `validate_tx_against_state` and `deduct_caller` into one function `validate_against_state_and_deduct_caller`
    * If you dont override those functions there is no action. If you do please look at `pre_execution::validate_against_state_and_deduct_caller`
    function or `OpHandler` for examples of migration.
* Breaking changed for EOF to support eof-devnet1. 
* `SharedMemory` is not longer Rc<RefCell<>> and internally uses Rc<RefCell<Vec<u8>>> buffer.
    * No action if you dont use it inside Interpreter.
* In `JournalExt` fn `last_journal_mut()` is renamed to `journal_mut()`
* EOF is disabled from Osaka and not accessible.

# v68 tag (revm v21.0.0) -> v70 tag ( revm v22.0.2)

No breaking changes

# v67 tag (revm v21.0.0) -> v68 tag ( revm v22.0.0)

* No code breaking changes
* alloy-primitives bumped to v1.0.0 and we had a major bump because of it.
