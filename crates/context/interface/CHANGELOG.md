# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [11.1.2](https://github.com/bluealloy/revm/compare/revm-context-interface-v11.1.1...revm-context-interface-v11.1.2) - 2025-10-15

### Other

- updated the following local packages: revm-state, revm-database-interface

## [11.1.1](https://github.com/bluealloy/revm/compare/revm-context-interface-v11.1.0...revm-context-interface-v11.1.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-state, revm-database-interface

## [11.1.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v11.0.0...revm-context-interface-v11.1.0) - 2025-10-09

### Other

- updated the following local packages: revm-database-interface

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v10.2.0...revm-context-interface-v11.0.0) - 2025-10-07

### Added

- Support bubbling up first precompile error messages  ([#2905](https://github.com/bluealloy/revm/pull/2905))
- add transaction index to batch execution error handling ([#3000](https://github.com/bluealloy/revm/pull/3000))
- Add Str(Cow<'static, str>) to InvalidTransaction error enum ([#2998](https://github.com/bluealloy/revm/pull/2998))
- allow EIP-7623 to be disabled ([#2985](https://github.com/bluealloy/revm/pull/2985))
- Introduced `all_mut` and `all` functions to ContextTr ([#2992](https://github.com/bluealloy/revm/pull/2992))
- send bytecode with call input ([#2963](https://github.com/bluealloy/revm/pull/2963))
- *(op-revm)* Add an option to disable "fee-charge" on `op-revm` ([#2980](https://github.com/bluealloy/revm/pull/2980))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))
- FrameStack mark push/end_init as unsafe ([#2929](https://github.com/bluealloy/revm/pull/2929))
- skip cold load on oog ([#2903](https://github.com/bluealloy/revm/pull/2903))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- helper function gas_balance_spending ([#3030](https://github.com/bluealloy/revm/pull/3030))
- remove unreachable zero-denominator check in fake_exponential ([#3039](https://github.com/bluealloy/revm/pull/3039))
- add ensure_enough_balance helper ([#3033](https://github.com/bluealloy/revm/pull/3033))
- add default impl for tx_local_mut and tx_journal_mut ([#3029](https://github.com/bluealloy/revm/pull/3029))
- prealloc few frames ([#2965](https://github.com/bluealloy/revm/pull/2965))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- *(cleanup)* Remove EIP-7918 related functions and EIP file  ([#2925](https://github.com/bluealloy/revm/pull/2925))
- cargo update ([#2930](https://github.com/bluealloy/revm/pull/2930))

## [10.2.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v10.1.0...revm-context-interface-v10.2.0) - 2025-09-23

### Added

- *(op-revm)* Add an option to disable "fee-charge" on `op-revm` ([#2980](https://github.com/bluealloy/revm/pull/2980))

## [10.1.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v10.0.1...revm-context-interface-v10.1.0) - 2025-08-23

### Added

- add generic state to ResultAndState ([#2897](https://github.com/bluealloy/revm/pull/2897))

## [10.0.1](https://github.com/bluealloy/revm/compare/revm-context-interface-v10.0.0...revm-context-interface-v10.0.1) - 2025-08-12

### Other

- Aggregate changes from PRs #2866, #2867, and #2874 ([#2876](https://github.com/bluealloy/revm/pull/2876))
- make ci happy ([#2863](https://github.com/bluealloy/revm/pull/2863))

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v9.0.0...revm-context-interface-v10.0.0) - 2025-08-06

### Added

- short address for journal cold/warm check ([#2849](https://github.com/bluealloy/revm/pull/2849))

### Fixed

- correct various typos in documentation and comments ([#2855](https://github.com/bluealloy/revm/pull/2855))
- swapped comments for db and db_mut methods in JournalTr trait ([#2774](https://github.com/bluealloy/revm/pull/2774))

### Other

- rm redundant lifetime constraints ([#2850](https://github.com/bluealloy/revm/pull/2850))
- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v8.0.1...revm-context-interface-v9.0.0) - 2025-07-23

### Fixed

- fully deprecate serde-json ([#2767](https://github.com/bluealloy/revm/pull/2767))

### Other

- un-Box frames ([#2761](https://github.com/bluealloy/revm/pull/2761))
- discard generic host implementation ([#2738](https://github.com/bluealloy/revm/pull/2738))

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-context-interface-v8.0.0...revm-context-interface-v8.0.1) - 2025-07-03

### Other

- updated the following local packages: revm-state, revm-database-interface

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v7.0.1...revm-context-interface-v8.0.0) - 2025-06-30

### Added

- implement `Transaction` for `Either` ([#2662](https://github.com/bluealloy/revm/pull/2662))
- optional_eip3541 ([#2661](https://github.com/bluealloy/revm/pull/2661))

### Other

- fix copy-pasted inner doc comments ([#2663](https://github.com/bluealloy/revm/pull/2663))

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-context-interface-v7.0.0...revm-context-interface-v7.0.1) - 2025-06-20

### Fixed

- call stack_frame.clear() at end ([#2656](https://github.com/bluealloy/revm/pull/2656))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v6.0.0...revm-context-interface-v7.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/bluealloy/revm/pull/2611)) ([#2642](https://github.com/bluealloy/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- change blob_max_count to max_blobs_per_tx ([#2608](https://github.com/bluealloy/revm/pull/2608))
- add optional priority fee check configuration ([#2588](https://github.com/bluealloy/revm/pull/2588))

### Other

- re-use frame allocation ([#2636](https://github.com/bluealloy/revm/pull/2636))
- store coinbase address separately to avoid cloning warm addresses in the common case ([#2634](https://github.com/bluealloy/revm/pull/2634))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v5.0.0...revm-context-interface-v6.0.0) - 2025-06-06

### Added

- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/bluealloy/revm/pull/2575))
- Config blob basefee fraction ([#2551](https://github.com/bluealloy/revm/pull/2551))
- expand timestamp/block_number to u256 ([#2546](https://github.com/bluealloy/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- support functions for eip7918 ([#2579](https://github.com/bluealloy/revm/pull/2579))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/bluealloy/revm/pull/2568))
- *(docs)* context crate lints ([#2565](https://github.com/bluealloy/revm/pull/2565))
- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/bluealloy/revm/pull/2560))
- *(cfg)* add tx_chain_id_check fields. Optimize effective gas cost calc ([#2557](https://github.com/bluealloy/revm/pull/2557))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v4.1.0...revm-context-interface-v5.0.0) - 2025-05-22

### Added

- make blob max number optional ([#2532](https://github.com/bluealloy/revm/pull/2532))

### Other

- add TxEnvBuilder::build_fill ([#2536](https://github.com/bluealloy/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [4.1.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v4.0.0...revm-context-interface-v4.1.0) - 2025-05-07

Dependency bump

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v3.0.0...revm-context-interface-v4.0.0) - 2025-05-07

### Added

- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/bluealloy/revm/pull/2462))
- Add a custom address to the CreateScheme. ([#2464](https://github.com/bluealloy/revm/pull/2464))
- *(Handler)* merge state validation with deduct_caller ([#2460](https://github.com/bluealloy/revm/pull/2460))
- add chain_ref method to ContextTr trait ([#2450](https://github.com/bluealloy/revm/pull/2450))
- *(tx)* Add Either RecoveredAuthorization ([#2448](https://github.com/bluealloy/revm/pull/2448))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/bluealloy/revm/pull/2382))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v2.0.0...revm-context-interface-v3.0.0) - 2025-04-09

### Fixed

- Effective gas price should check tx type ([#2375](https://github.com/bluealloy/revm/pull/2375))

### Other

- make blob params u64 ([#2385](https://github.com/bluealloy/revm/pull/2385))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0...revm-context-interface-v2.0.0) - 2025-03-28

### Added

- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.5...revm-context-interface-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.5...revm-context-interface-v1.0.0-alpha.6) - 2025-03-21

### Added

- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/bluealloy/revm/pull/2233))
- allow reuse of API for calculating initial tx gas for tx ([#2215](https://github.com/bluealloy/revm/pull/2215))

### Other

- use AccessListItem associated type instead of AccessList ([#2214](https://github.com/bluealloy/revm/pull/2214))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.4...revm-context-interface-v1.0.0-alpha.5) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.3...revm-context-interface-v1.0.0-alpha.4) - 2025-03-12

### Added

- add custom error to context ([#2197](https://github.com/bluealloy/revm/pull/2197))
- Add tx/block to EvmExecution trait ([#2195](https://github.com/bluealloy/revm/pull/2195))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.2...revm-context-interface-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- Add comments to handler methods ([#2188](https://github.com/bluealloy/revm/pull/2188))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-context-interface-v1.0.0-alpha.1...revm-context-interface-v1.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))
- Standalone Host, remove default fn from context ([#2147](https://github.com/bluealloy/revm/pull/2147))
- implement AccessListTr for Vec ([#2136](https://github.com/bluealloy/revm/pull/2136))
- allow host to be implemented on custom context ([#2112](https://github.com/bluealloy/revm/pull/2112))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/bluealloy/revm/pull/2132))
- fix eofcreate error typo ([#2120](https://github.com/bluealloy/revm/pull/2120))
- Add docs to revm-bytecode crate ([#2108](https://github.com/bluealloy/revm/pull/2108))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-context-interface-v1.0.0-alpha.1) - 2025-02-16

### Added

- Introduce Auth and AccessList traits (#2079)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- align Block trait (#1957)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- Clear journal (#1927)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- API cleanup (#2067)
- relax halt reason bounds (#2041)
- simplify some generics (#2032)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- tie journal database with database getter (#1923)
- Move CfgEnv from context-interface to context crate (#1910)
- Bump new logo (#1735)
- *(README)* add rbuilder to used-by (#1585)
- added simular to used-by (#1521)
- add Trin to used by list (#1393)
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))
- Add Hardhat to the "Used by" list ([#1164](https://github.com/bluealloy/revm/pull/1164))
- Add VERBS to used by list ([#1141](https://github.com/bluealloy/revm/pull/1141))
- license date and revm docs (#1080)
- *(docs)* Update the benchmark docs to point to revm package (#906)
- *(docs)* Update top-level benchmark docs (#894)
- clang requirement (#784)
- Readme Updates (#756)
- Logo (#743)
- book workflow ([#537](https://github.com/bluealloy/revm/pull/537))
- add example to revm crate ([#468](https://github.com/bluealloy/revm/pull/468))
- Update README.md ([#424](https://github.com/bluealloy/revm/pull/424))
- add no_std to primitives ([#366](https://github.com/bluealloy/revm/pull/366))
- revm-precompiles to revm-precompile
- Bump v20, changelog ([#350](https://github.com/bluealloy/revm/pull/350))
- typos (#232)
- Add support for old forks. ([#191](https://github.com/bluealloy/revm/pull/191))
- revm bump 1.8. update libs. snailtracer rename ([#159](https://github.com/bluealloy/revm/pull/159))
- typo fixes
- fix readme typo
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/bluealloy/revm/pull/52))
- readme. debuger update
- Bump revm v0.3.0. README updated
- readme
- Add time elapsed for tests
- readme updated
- Include Basefee into cost calc. readme change
- Initialize precompile accounts
- Status update. Taking a break
- Merkle calc. Tweaks and debugging for eip158
- Replace aurora bn lib with parity's. All Bn128Add/Mul/Pair tests passes
- TEMP
- one tab removed
- readme
- README Example simplified
- Gas calculation for Call/Create. Example Added
- readme usage
- README changes
- Static gas cost added
- Subroutine changelogs and reverts
- Readme postulates
- Spelling
- Restructure project
- First iteration. Machine is looking okay
