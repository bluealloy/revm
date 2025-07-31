# v83 tag (revm v28.0.0) from v82 tag (revm v27.1.0)

* `SystemCallEvm` functions got renamed and old ones are deprecated. Renaming is done to align it with other API calls.
   * `transact_system_call_finalize` is now `system_call`.
   * `transact_system_call` is now `system_call_one`.

# v82 tag (revm v27.1.0) from v81 tag (revm v27.0.3)

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