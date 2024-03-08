# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v2.1.0...revm-primitives-v3.0.0) - 2024-03-08

### Added
- *(primitives)* derive Default for SpecId ([#1171](https://github.com/bluealloy/revm/pull/1171))
- implement DatabaseCommit for DatabaseComponents ([#1163](https://github.com/bluealloy/revm/pull/1163))
- add PrecompileError::Other ([#1165](https://github.com/bluealloy/revm/pull/1165))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- Add boxed precompile trait ([#1131](https://github.com/bluealloy/revm/pull/1131))

### Other
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/bluealloy/revm/pull/1132))
- add helper function for spec id ([#1130](https://github.com/bluealloy/revm/pull/1130))

## [2.1.0](https://github.com/bluealloy/revm/compare/revm-primitives-v2.0.1...revm-primitives-v2.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))

### Other
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))

## [2.0.1](https://github.com/bluealloy/revm/compare/revm-primitives-v2.0.0...revm-primitives-v2.0.1) - 2024-02-17

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))
- optimism compilation ([#1091](https://github.com/bluealloy/revm/pull/1091))
- Set is_optimism to true if default config ([#1087](https://github.com/bluealloy/revm/pull/1087))

### Other
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-primitives-v1.3.0...revm-primitives-v2.0.0) - 2024-02-07

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/bluealloy/revm/pull/990))
- Convert optimism panic into graceful error ([#982](https://github.com/bluealloy/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- transition account balance delta ([#843](https://github.com/bluealloy/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- add is_empty_code_hash fn ([#826](https://github.com/bluealloy/revm/pull/826))
- *(revme)* format kzg setup ([#818](https://github.com/bluealloy/revm/pull/818))
- add more `auto_impl`s to revm traits ([#799](https://github.com/bluealloy/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- add changed storage slots iter ([#801](https://github.com/bluealloy/revm/pull/801))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))

### Fixed
- use maximum possible data fee for 4844 balance checks ([#981](https://github.com/bluealloy/revm/pull/981))
- make revm-primitives no-std with c-kzg feature ([#933](https://github.com/bluealloy/revm/pull/933))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/bluealloy/revm/pull/824))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/bluealloy/revm/pull/795))
- *(primitives)* Error Primitive Display + Error Implementations ([#770](https://github.com/bluealloy/revm/pull/770))

### Other
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/bluealloy/revm/pull/1059))
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/bluealloy/revm/pull/1041))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/bluealloy/revm/pull/1026))
- add some docs to StorageSlot ([#1019](https://github.com/bluealloy/revm/pull/1019))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(deps)* bump bitflags from 2.4.1 to 2.4.2 ([#983](https://github.com/bluealloy/revm/pull/983))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(deps)* alloy 0.6 ([#963](https://github.com/bluealloy/revm/pull/963))
- *(primitives)* improve readability of Env.validate_tx() ([#924](https://github.com/bluealloy/revm/pull/924))
- *(primitives)* optimize AccountInfo.is_empty() ([#922](https://github.com/bluealloy/revm/pull/922))
- *(deps)* bump once_cell from 1.18.0 to 1.19.0 ([#908](https://github.com/bluealloy/revm/pull/908))
- fix case for CreateInitCodeSizeLimit error ([#896](https://github.com/bluealloy/revm/pull/896))
- Add docs to `optimism` `InvalidTransaction` errors. ([#884](https://github.com/bluealloy/revm/pull/884))
- *(deps)* run `cargo update`, bump alloy-primitives ([#880](https://github.com/bluealloy/revm/pull/880))
- *(primitives)* don't duplicate the SpecId::enabled() logic ([#869](https://github.com/bluealloy/revm/pull/869))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- cargo fmt and removed extra newlines ([#860](https://github.com/bluealloy/revm/pull/860))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/bluealloy/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/bluealloy/revm/pull/859))
- bump c-kzg to v0.4.0 ([#849](https://github.com/bluealloy/revm/pull/849))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/bluealloy/revm/pull/844))
- decode KZG points directly into the buffers ([#840](https://github.com/bluealloy/revm/pull/840))
- *(deps)* make derive-more optional ([#827](https://github.com/bluealloy/revm/pull/827))
- *(eip4844)* update kzg trusted setup ([#822](https://github.com/bluealloy/revm/pull/822))
- *(deps)* bump bitflags from 2.4.0 to 2.4.1 ([#803](https://github.com/bluealloy/revm/pull/803))
- include alloy-primitives std feature ([#788](https://github.com/bluealloy/revm/pull/788))
- *(primitives)* OptimismFields Docs Cleanup ([#778](https://github.com/bluealloy/revm/pull/778))
- Remove dead state trait function ([#779](https://github.com/bluealloy/revm/pull/779))
- Verify account is empty checks ([#780](https://github.com/bluealloy/revm/pull/780))
- *(primitives)* Clean up the account typs with docs ([#776](https://github.com/bluealloy/revm/pull/776))
- Use upstream create and create2 implementations ([#775](https://github.com/bluealloy/revm/pull/775))
- Implement display and error for precompile error ([#777](https://github.com/bluealloy/revm/pull/777))
- reorder env members ([#771](https://github.com/bluealloy/revm/pull/771))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v1.3.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>
* 83d27b0 - fix: use u128 for calc_blob_gasprice (#764) (16 hours ago) <Dan Cline>
* 1f86e45 - chore(deps): bump proptest from 1.2.0 to 1.3.1 (#763) (21 hours ago) <dependabot[bot]

# v1.2.0
date 28.09.2023

Summary of biggest changes:
* Some check for Env validity moved from revm to primitives crate.
* Cancun spec introduced.
* no_std added to primitives.
* introduce initcode size limit check taking config into account.
* deprecate `RefDBWrapper` for more generic `WrapDatabaseRef`.
* Implement `Error` for EVMError.
* Removal of hash from Bytecode.
* ChainId converted from U256 to u64.
* CfgEnv marked as `non_exhaustive` to accommodate future changes.
* Introduce `InvalidHeader` error that contains `prevrandao` and `blob gas` not set errors.
* c-kzg added as dependency as it is needed for `KzgSetting` that is sed inside EnvCfg.

Full git log:
* ea0d8d8 - fix: use u128 for calc data fee result (#757) (49 minutes ago) <Dan Cline>
* 4f916be - chore: bump c-kzg to create lib (#758) (5 hours ago) <rakita>
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 8a85d19 - fix: balance check disabled (#751) (25 hours ago) <Wodann>
* 8206193 - feat: add "kzg" as a separate feature (#746) (80 minutes ago) <DaniPopes>
* 26af13e - EIP-7516: BLOBBASEFEE opcode (#721) (5 days ago) <rakita>
* f72eaa0 - chore: error type for block header (#731) (5 days ago) <hack3r-0m>
* 1f31756 - document when InvalidTransaction errors are thrown (#722) (6 days ago) <Alessandro Mazza>
* cb39117 - fix(eip4844): Pass eth tests, additional conditions added. (#735) (6 days ago) <rakita>
* 70cf969 - chore: rm nonexhaustive for error (#726) (8 days ago) <Matthias Seitz>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* d615514 - chore: clippy incorrect_clone_impl_on_copy_type (#720) (12 days ago) <rakita>
* 5d68dd5 - chore(deps): bump bytes from 1.4.0 to 1.5.0 (#707) (2 weeks ago) <dependabot[bot]>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 616cc7e - chore(cfg): convert chain_id from u256 to u64 (#693) (3 weeks ago) <Lorenzo Feroleto>
* 7e7cb02 - Small doc comment fix (#698) (3 weeks ago) <refcell.eth>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* b0ee6d4 - feat: derive PartialEq, Eq for Env (#689) (3 weeks ago) <Dan Cline>
* 86d25c6 - chore: remove unused new_raw_with_hash (#676) (4 weeks ago) <Dan Cline>
* 175aaec - Removed the last dependencies breaking no-std build. (#669) (4 weeks ago) <Lucas Clemente Vella>
* f2929ad - chore(deps): bump proptest-derive from 0.3.0 to 0.4.0 (#652) (4 weeks ago) <dependabot[bot]>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 43d535c - style: bundle state (#637) (5 weeks ago) <Roman Krasiuk>
* 321152a - book workflow (#537) (5 weeks ago) <Waylon Jepsen>
* 0028193 - feat: Optional coinbase tip (#625) (5 weeks ago) <clabby>
* 3907fdf - chore: mark CfgEnv as non_exhaustive (#623) (5 weeks ago) <rakita>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* 449d6b9 - chore: export some `unreachable_pub` items (#598) (6 weeks ago) <DaniPopes>
* fc2107c - chore: Revert test, not change storage check , renaming of original slot value (#601) (6 weeks ago) <rakita>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* 5cdaa97 - chore: avoid unnecessary allocations (#581) (6 weeks ago) <DaniPopes>
* e9b6859 - chore(deps): bump bitflags from 2.3.3 to 2.4.0 (#596) (6 weeks ago) <dependabot[bot]>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 157ef36 - feat: introduce initcode size limit check taking config into account (#587) (7 weeks ago) <evalir>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* 781c8cc - feat: Implement `Error` for EVMError (#559) (9 weeks ago) <Yiannis Marangos>
* 5ce9dc9 - chore: clippy and fmt (#568) (9 weeks ago) <rakita>
* c153428 - feat(cancun): EIP-5656: MCOPY - Memory copying instruction (#528) (3 months ago) <Waylon Jepsen>
* 1839b3f - chore(deps): bump hashbrown from 0.13.2 to 0.14.0 (#519) (3 months ago) <dependabot[bot]>
* 63f9460 - chore(deps): bump auto_impl from 1.0.1 to 1.1.0 (#478) (3 months ago) <dependabot[bot]>
* b224874 - chore: add util functions for getting output data (#509) (4 months ago) <Matthias Seitz>
* e0ec1cc - chore: fix typo (#488) (4 months ago) <adria0.eth>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* d193418 - chore: Bundle inspector crate/call calls (#480) (5 months ago) <rakita>
* 75a6136 - feat: Introduce account status as bitflag inside JournalState (#477) (5 months ago) <rakita>


# v1.1.2
date: 03.05.2023

small release:
* ccefbca - chore(deps): bump ruint from 1.7.0 to 1.8.0 (#465) (50 minutes ago) <dependabot[bot]>
* d7adfd5 - Fix typo in primitives/src/state.rs (#474) (50 minutes ago) <Udoagwa Franklin>
* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>
# v1.1.1
date: 14.04.2023

One change:
* 4915bd1 - chore: add into_logs (#453) (3 days ago)


# v1.1.0
date: 04.04.2023

Mosty utility functions, addional checks and convenience changes.
Old bytecode that supported gas block was replaced with jumpmap only bitvec.

Changelog: 
* 992a11c - (HEAD -> v/310, origin/lib_versions) bump all (81 minutes ago) <rakita>
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 2d5b710 - Comment Fix (#430) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* 3d8ca66 - feat: add Output::into_data (#420) (3 weeks ago) <Matthias Seitz>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* cff1070 - Update readmdoc of `perf_analyse_created_bytecodes` (#404) (4 weeks ago) <rakita>
* 7bb73da - feat: Add check for chainID (#393) (4 weeks ago) <chirag-bgh>
* 3a17ca8 - feat: add b256<->u256 from impls (#398) (4 weeks ago) <Matthias Seitz>
* 3789509 - feat: add API to retrieve unpadded bytecode (#397) (5 weeks ago) <Wodann>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* 5efd9d1 - impl NonceTooHigh/ NonceTooLow checks (#383) (6 weeks ago) <gd>
* 188dacf - improvement: derive Debug for DatabaseComponentError (#377) (7 weeks ago) <Wodann>
* 0401cfd - Add B160/B256 From primitive_types traits (#380) (7 weeks ago) <Francesco Cinà>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* 5788340 - chore(deps): bump bytes from 1.3.0 to 1.4.0 (#355) (7 weeks ago) <dependabot[bot]>
* b4c62e9 - chore: rename Then to Than (#368) (7 weeks ago) <Matthias Seitz>
* 1c3e9e3 - improvement: use alloc & core for Arc impl (#367) (8 weeks ago) <Wodann>
* 3158ce9 - feat: implement Debug for DatabaseComponentError if supported (#363) (8 weeks ago) <Wodann>


* d9727c2 - improvement: add error details to InvalidTransaction::LackOfFundForGasLimit (#364) (8 weeks ago) <Wodann>
* 5d6ecd0 - improvement: implement BlockHash for Arc<BlockHashRef> (#361) (8 weeks ago) <Wodann>
* ae9baba - improvement: implement State for Arc<StateRef> (#360) (8 weeks ago) <Wodann>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>

# v1.0.0
date: 29.01.2023

Interpreter was extracted from main revm crate at the revm v3.0.0 version.