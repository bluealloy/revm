# v1.2.0
date 20.1.2022

Changes:
* Bump revm_precompile and added new feature for k256 lib.

# v1.1.0
date: 14.1.2022

There is bug introduced in last release with gas blcok optimization, it will crash revm if anywhere in contract is unknown OpCode. And now returning log after execution (ups) included them in eth/tests verification.

Changes:
* Bug fix for unknown OpCode
* Omit edgecase high nonce test. tracer gas fix 
* Some internal cleanup

# v1.0.0
date: 18.12.2021

It feel's like that the lib is in the state that is okay to promote it to the v1 version. Other that that, a lot of optimizations are done and the inspector trait was rewritten.

Changes: 
*  web3 db
*  precalculated gas blocks. Optimization
*  PC opcode as pointer. Optimization
*  U256 div_rem optimization
*  Inspector refactored and it is now closer to Host interface.

Optimization thread: https://github.com/bluealloy/revm/issues/7


# v0.5.0
date: 17.11.2021

A lot of optimization on machine(Interpreter) part, it is now at least 3x faster. On interface side, Error enum was renamed to Return and it is simplified. Additionally if needed gas measuring can be removed with rust feature.

Changes: 
* push instruction optimized.
* mload/mstore and memory optimized
* Gas calculation optimized
* optimize i256
* switch stacks from H256 with U256
* Error's refactor to Return
* clippy/warnings/fmt cleanup
* Bump auto_impl to v0.5
* opcode renaming
* Gas measurment can be removed with rust features.

# v0.4.1
date: 02.11.2021

Change in interface and how you can call evm. There is now multiple Database traits for use and inspector is taken on transact call as reference.

* 20ac70b - Database traits made useful.
* 46b5bcd - EVM Interface changed. Inspector called separately.


# v0.3.1
date: 27.10.2021

remove some warnings for unused imports and done cargo fmt.
# v0.3.0
date: 27.10.2021

Interface revamped and now looks a lot better.

Log:
* 1b1ebd8 - [revm] Interface. Inspector added, Env cleanup. revm-test passes (9 hours ago) <rakita>
* 351d4e0 - BIG interface change (11 hours ago) <rakita>
* a723827 - no_sdt to no_std (2 days ago) <rakita>
* a449bed - [precompiles] spelling, small cleanup (2 days ago) <rakita>


# v0.2.2

Same as v0.2.1 but added readme.
# v0.2.1
date: 25.10.2021

Big refactor, cleanup changes, and updating tests. EIP-3607 added.

Log:
* a6e01de - BIG reorg. workspace added. revm-precompile lib (20 minutes ago) <rakita>
* e50f6d3 - Move merkle trie from revm to eth/tests crate (4 hours ago) <rakita>
* 633ffd4 - Bump tests to v10.1 (28 hours ago) <rakita>
* 14b3de1 - Payment overflow check (30 hours ago) <rakita>
* 6e964ba - EIP-3607: Reject transactions from senders with deployed code (30 hours ago) <rakita>


# v0.2.0
date: 23.10.2021:

Published v0.2.0, first initial version of code. London supported and all eth state test are 100% passing or Istanbul/Berlin/London.


### 17.10.2021:
-For past few weeks working on this structure and project in general become really good and I like it. For me it surved as good distraction for past few weeks and i think i am going to get drained if i continue working on it, so i am taking break and i intend to come back after few months and finish it.
- For status:
    * machine/spec/opcodes/precompiles(without modexp) feels good and I probably dont need to touch them.
    * inspector: is what i wanted, full control on insides of EVM so that we can control it and modify it. will probably needs to add some small tweaks to interface but nothing major.
    * subroutines: Feels okay but it needs more scrutiny just to be sure that all corner cases are covered.
    * Test that are failing (~20) are mostly related to EIP-158: State clearing. For EIP-158 I will time to do it properly.
    * There is probably benefit of replaing HashMap hasher with something simpler, but this is research for another time.
## Project structure: