# Test for EIP-7907

* Does access list warms the code?
* extcodecopy check if len == 0 check is done before loading extra code size.

# v82 (revm v28.0.0) from v81 tag (revm v27.0.1)

* Changes releted to EIP-7907
    * This affects journal and how code is loaded, some functions now do not load the code.
    

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