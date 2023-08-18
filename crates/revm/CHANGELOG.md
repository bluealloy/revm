# v3.3.0
date: 03.05.2023

Consensus bug:
* cde2f2d - fix: revert of selfdestruct with same target address (#475) (2 hours ago) <Roman Krasiuk>

Other small changes:
* bd0fad8 - (HEAD -> reles, origin/main, origin/HEAD) chore(deps): bump tokio from 1.27.0 to 1.28.0 (#470) (52 minutes ago) <dependabot[bot]>
* ccefbca - chore(deps): bump ruint from 1.7.0 to 1.8.0 (#465) (52 minutes ago) <dependabot[bot]>
* 7c2e0f5 - chore(deps): bump anyhow from 1.0.70 to 1.0.71 (#471) (53 minutes ago) <dependabot[bot]>
* d7adfd5 - Fix typo in primitives/src/state.rs (#474) (53 minutes ago) <Udoagwa Franklin>
* d0cd897 - add example to revm crate (#468) (8 days ago) <Sambhav>
* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v3.2.0
date: 19.04.2023

consensus bug:
* fix: touched account on creation (#463) (2 hours ago) <Roman Krasiuk>

# v3.1.1
date: 14.04.2023

bump revm dependency versions.

# v3.1.0
date: 04.04.2022

Main changes can be summarizes in:
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
    * removal of gas block allowed us to have more compact analysis data. Gas block from beginning didn't have big impact on performance but introduced not intuitive gas calculations that was
    source of some bugs. 
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
    * revm is Shanghai ready
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
    * There wasone  bug inside CacheDB that was here for a long time, and would happen only if
    selfdestruct/create2 is called in multiple transaction on same account on same cache data.
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>


Changelogs:
* 9edb8f4 - (origin/main, origin/HEAD) Improve EthersDB::new (#440) (5 days ago) <lazymio>
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 6b09caf - chore(deps): bump serde_json from 1.0.94 to 1.0.95 (#434) (6 days ago) <dependabot[bot]>
* 77f1735 - chore(deps): bump walkdir from 2.3.2 to 2.3.3 (#426) (8 days ago) <dependabot[bot]>
* ed981c3 - chore(deps): bump serde from 1.0.157 to 1.0.158 (#425) (8 days ago) <dependabot[bot]>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* 2d5b710 - Comment Fix (#430) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* d935525 - chore(deps): bump secp256k1 from 0.26.0 to 0.27.0 (#429) (2 weeks ago) <dependabot[bot]>
* a85ff79 - Update README.md (#424) (2 weeks ago) <Waylon Jepsen>
* 9645015 - chore(deps): bump thiserror from 1.0.38 to 1.0.40 (#421) (2 weeks ago) <dependabot[bot]>
* aa6519f - chore(deps): bump enumn from 0.1.6 to 0.1.8 (#422) (2 weeks ago) <dependabot[bot]>
* d63146f - chore(deps): bump futures from 0.3.26 to 0.3.27 (#416) (2 weeks ago) <dependabot[bot]>
* 52fe7c4 - chore(deps): bump serde_json from 1.0.93 to 1.0.94 (#401) (2 weeks ago) <dependabot[bot]>
* b98d9c9 - chore(deps): bump serde from 1.0.152 to 1.0.157 (#423) (2 weeks ago) <dependabot[bot]>
* 3d8ca66 - feat: add Output::into_data (#420) (3 weeks ago) <Matthias Seitz>
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* b1208fe - feat: add contract+target to selfdestruct hook (#410) (4 weeks ago) <Matthias Seitz>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* f2656b7 - chore: add primitive SpecId to precompile SpecId conversion (#408) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* cff1070 - Update readmdoc of `perf_analyse_created_bytecodes` (#404) (4 weeks ago) <rakita>
* fbc62a3 - chore: fix typo StorageChange (#403) (4 weeks ago) <Matthias Seitz>
* 7bb73da - feat: Add check for chainID (#393) (4 weeks ago) <chirag-bgh>
* 3a17ca8 - feat: add b256<->u256 from impls (#398) (4 weeks ago) <Matthias Seitz>
* 3789509 - feat: add API to retrieve unpadded bytecode (#397) (5 weeks ago) <Wodann>
* 5ab154a - chore(deps): bump tokio from 1.25.0 to 1.26.0 (#395) (5 weeks ago) <dependabot[bot]>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* 8dc024a - Add copyright start year (#387) (5 weeks ago) <Marius Kjærstad>
* 4d2f074 - feat: add EVM::with_env (#385) (6 weeks ago) <Matthias Seitz>
* 5efd9d1 - impl NonceTooHigh/ NonceTooLow checks (#383) (6 weeks ago) <gd>
* 8e6f4f2 - chore: fix compilation if serde not enabled (#381) (7 weeks ago) <rakita>
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>
* ec582a8 - chore(deps): bump once_cell from 1.17.0 to 1.17.1 (#378) (7 weeks ago) <dependabot[bot]>
* 188dacf - improvement: derive Debug for DatabaseComponentError (#377) (7 weeks ago) <Wodann>
* 0401cfd - Add B160/B256 From primitive_types traits (#380) (7 weeks ago) <Francesco Cinà>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* d5ebdb0 - chore(deps): bump tokio from 1.24.2 to 1.25.0 (#352) (7 weeks ago) <dependabot[bot]>
* ebaccca - chore(deps): bump futures from 0.3.25 to 0.3.26 (#353) (7 weeks ago) <dependabot[bot]>
* 5788340 - chore(deps): bump bytes from 1.3.0 to 1.4.0 (#355) (7 weeks ago) <dependabot[bot]>
* d3fba88 - chore(deps): bump serde_json from 1.0.92 to 1.0.93 (#365) (7 weeks ago) <dependabot[bot]>
* e22c3f3 - fix: call create_end for all code paths (#362) (7 weeks ago) <Wodann>
* b4c62e9 - chore: rename Then to Than (#368) (7 weeks ago) <Matthias Seitz>
* 1c3e9e3 - improvement: use alloc & core for Arc impl (#367) (8 weeks ago) <Wodann>
* 3158ce9 - feat: implement Debug for DatabaseComponentError if supported (#363) (8 weeks ago) <Wodann>
* d9727c2 - improvement: add error details to InvalidTransaction::LackOfFundForGasLimit (#364) (8 weeks ago) <Wodann>
* 6b170b4 - Use gas price in place of effective gas price for initial balance check (#359) (8 weeks ago) <gd>
* 5d6ecd0 - improvement: implement BlockHash for Arc<BlockHashRef> (#361) (8 weeks ago) <Wodann>
* ae9baba - improvement: implement State for Arc<StateRef> (#360) (8 weeks ago) <Wodann>
* 2e4e800 - chore(deps): bump serde_json from 1.0.91 to 1.0.92 (#357) (8 weeks ago) <dependabot[bot]>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>
* 10187ed - data change (9 weeks ago) <rakita>

# v3.0.0
date 29.01.2022

This is big release that has core changes that breaks compatibility. In summary:
*  Project is refactored into `revm-primitives`,`revm-precompile`,`revm-interpreter` and `revm` to have more flexibility and separation of concerns. And include paths in revm reflect that. So try to find include as `revm::primitives` or `revm::interpreter`
* Parity `primitive-types` was replaced with `ruint` for big numbers and subset of macros are used for native `B160`/`B256` types. 
* Interpreter instructions are unified and now all of them have same signature.
* web3 db was replaces with ethers alternative.
* revmjs lib was removed from crates.
* `revm_precompiles` was renamed to `revm-precompile.`

* Return types are made to have more insight of what have happened inside revm.
* Snailtracer benchmark got around 20% faster.

Github Changelog:
* dc9818f - (HEAD -> o/bump, origin/bump_v20) Bump v20 (13 hours ago) <rakita>
* 75ef0f1 - (origin/main, origin/HEAD) feat: Staticcall internal return (#349) (13 hours ago) <rakita>
* 0194b37 - (t) fix bug introduced in last commit (13 hours ago) <rakita>
* 7b00f32 - Cleanup imports (#348) (14 hours ago) <rakita>
* c14d7ea - fix: enable the examples to run with the current revm (#347) (16 hours ago) <flyq>
* 329fd94 - Wrap all calls to interpreter.gas.erase_cost with checks if USE_GAS is enabled (#346) (2 days ago) <christn>
* 72355f4 - improvement: add logs & return value to revert (#343) (3 days ago) <Wodann>
* 142a1c9 - expose hashbrown::HashMap in primitives (#345) (3 days ago) <Andy Thomson>
* ba393d7 - fix: disable balance check (#342) (4 days ago) <Wodann>
* 876fad1 - refactor: simplify DatabaseComponentError (#339) (6 days ago) <Wodann>
* 81534ad - chore: includes to libs (#338) (7 days ago) <rakita>
* e2f4d32 - Creating revm-primitives, revm better errors and db components  (#334) (10 days ago) <rakita>
* de83db6 - fix: feature flags (#330) (2 weeks ago) <Wodann>
* b60269c - `revm`: mark `with-serde` feature as deprecated (#328) (2 weeks ago) <Enrique Ortiz>
* 63bf475 - make load_account pub (#325) (3 weeks ago) <rakita>
* 0ef0197 - Cleanup, move hot fields toggether in Interpreter (#321) (3 weeks ago) <rakita>
* 81942d6 - enable proptest with arbitrary feature (#323) (3 weeks ago) <joshieDo>
* 2be3798 - feat: revm-interpreter created (#320) (3 weeks ago) <rakita>
* 7e98fef - fix: feature flag compiler errors (#256) (5 weeks ago) <Wodann>
* 488ef8a - Add example for fork + ref_transact impl (#296) (6 weeks ago) <0xDmtri>
* 56e6c22 - feat: allow disabling of balance checks (#297) (6 weeks ago) <Wodann>
* 8661467 - feat: Export CustomPrinter insector from revm (#300) (6 weeks ago) <rakita>
* 222b8e9 - feature: substitute web3db to ethersdb (#293) (6 weeks ago) <0xDmtri>
* fd01083 - feature(revm): Return `bytes` in Create calls (#289) (7 weeks ago) <Nicolas Gotchac>
* 2fb0933 - docs: Correct typo (#282) (7 weeks ago) <Przemyslaw Rzad>
* 90fe01e - feat(interpreter): Unify instruction fn signature (#283) (7 weeks ago) <rakita>
* 54e0333 - bug: Integer overflow while calculating the remaining gas in GasInspector (#287) (8 weeks ago) <rakita>
* acdbaac - native bits (#278) (8 weeks ago) <rakita>
* 69e302b - feat(revm): Add prevrandao field to EnvBlock (#271) (2 months ago) <rakita>
* d1703cd - Export StorageSlot (#265) (3 months ago) <Francesco Cinà>
* 560bb03 - Fix: typos (#263) (3 months ago) <HAPPY>
* 369244e - feat(refactor): make keccak in one place. (#247) (3 months ago) <rakita>
* c96c878 - feat: Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239) (3 months ago) <Alexey Shekhirin>


# v2.3.1
date: 22.11.2022

Bump dependency versions.


# v2.3.0
date: 16.11.2022
Very small release. Exposes one field and added prevrandao to remove footgun of forgeting to set difficulty.

* 927d16c - disable gas refunds with env flag (#267) (14 minutes ago) <gd>
* 47a8310 - Add prevrandao field to EnvBlock (3 minutes ago) <rakita>
* 2c45b04 - Export StorageSlot (#265) (23 minutes ago) <Francesco Cinà>

# v2.2.0
date: 12.11.2022

Small release that contains consensus bug fix. Additionaly added few small feature flags needed for hardhat, opcode utility function and removal of web3db block number check. 

* dc3414a - Added OEF spec for tests. Skip HighGasPrice (4 minutes ago) <rakita>
* f462f9d - Bugfix: if returndatacopy is len 0 return after initial cost (#259) (4 minutes ago) <gd>
* ea2f2a2 - fix web3db sanity check (#245) (12 days ago) <Wulder>
* 9f8cdbd - feat: allow block gas limit to be toggled off (#238) (3 weeks ago) <Wodann>
* efd9afc - feat: allow eip3607 to be toggled off (#237) (3 weeks ago) <Wodann>
* 88c72a7 - fix: return out of gas code for precompiled contracts (#234) (3 weeks ago) <Wodann>
* 30462a3 - Fix: typos (#232) (3 weeks ago) <omahs>
* 9f513c1 - Borrow self and add derive traits for OpCode (#231) (4 weeks ago) <Franfran>

# v2.1.0
date: 25.09.2022

GasInspector added by Alexey Shekhirin and some helper functions.
Changes:

* ca14d61 - gas inspector (#222) (7 days ago) <Alexey Shekhirin>
* 1e25c99 - chore: expose original value on storageslot (#216) (13 days ago) <Matthias Seitz>
* aa39d64 - feat: add Memory::shrink_to_fit (#215) (13 days ago) <Matthias Seitz

# v2.0.0
date: 10.09.2022

Release with `Database` interface changed, execution result, consensus bug fixes and support for all past forks. Additional optimizations on evm initialization.

Main changes:
* Add support for old forks. (#191) (9 days ago)
* revm/evm: Return `ExecutionResult`, which includes `gas_refunded` (#169) (4 weeks ago) <Nicolas Gotchac>
* JournaledState (#175)
    * Optimize handling of precompiles. Initialization and account loading.
    * Fixes SELFDESTRUCT bug.
* Optimize calldataload. Some cleanup (#168)
* Handle HighNonce tests (#176)
* feat: expose hash on `BytecodeLocked` (#189) (12 days ago) <Bjerg>
* revm: Update account storage methods in CacheDB (#171) (4 weeks ago) <Nicolas Gotchac>
* reexport revm_precompiles as precompiles (#197) (6 days ago) <Matthias Seitz>
* chore(ci): use ethtests profile for CI tests (#188) (2 weeks ago) <Alexey Shekhirin>
* Bump dependencies version
* current_opcode fn and rename program_counter to instruction_pointer (#211)
* Cfg choose create analysis, option on bytecode size limit (#210)
* Cleanup remove U256 and use u64 for gas calculation (#213)

Consensus bugs:
* SELFDESTRUCT was not handled correctly. It would remove account/storage but it should just mark it for removal. This bug was here from earlier version of revm. (#175)
* fix: set gas_block to empty bytecode (#172). Introduced in v1.8.0 with bytecode format.

# v1.9.0
date: 09.08.2022

Small release. Optimizations

* Cache bytecode hash
* Move override_spec config from Inspector to cfg

# v1.8.0
date: 01.08.2022

Medium release, good performance boost. Database trait has changed to support Bytecode.

* Introduce Bytecode format (#156)
* Update readme files.
* Merge eth/tests supported.

# v1.7.0
date: 11.06.2022

small release:
* Make CacheDB field pub and add few utility functions
* Rename Byzantine to Byzantium

# v1.6.0
date: 02.06.2022

Most changes are relayed to CacheDB and how it saved accounts.

* Introduce account `Touched/Cleared/None` state in CacheDB
* Add missing inspectors `call_end` calls
* bump dependencies and few standard derives.

# v1.5.0
date: 09.06.2022

Consensus error related to gas block optimization and `sstore` min stipend. Solution is to make `sstore` instruction as `gas_block_end` as to not spend future instruction gas when checking min stipend condition introduced in EIP-2200.

* Consensus error with gas block for SSTORE stipend check (#124)
* enable EIP2200 in Istanbul (#125)

# v1.4.1
date: 06.06.2022

Small release:
* chore: export evm_inner (#122)

# v1.4.0
date: 03.06.2022

Small release:
* fix: BLOCKHASH should return 0 if number not in last 256 blocks (#112)
* feat: add getters for cachedb (#119)
* bump some lib versions.

# v1.3.1
date: 11.4.2022

Small fixes release.
* Empty keccak constant and remove access_list.clone (#111)
* chore: typo fixes
* fix is_static for Inspector initialize_interp

# v1.3.0
date: 30.4.2022

There are a lot of big changes that are included in this release as revm was integrated inside foundry.

* A lot of changed on Inspector, added new calls and flushed out how it should be called. Big effort mostly driven by Oliver Nordbjerg
* Big internal refactor and renaming: Machine->Inspector, call/create info are now in structs.
* feat: add serde support to model types. Thank you Matthias Seitz
* Added rust feature that sets memory limit on interpreter that is configurable with env.cfg. by Oliver Nordbjerg.
* Library bumped to higher version.

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