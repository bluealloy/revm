# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [25.0.2](https://github.com/bluealloy/revm/compare/revm-interpreter-v25.0.1...revm-interpreter-v25.0.2) - 2025-08-23

### Fixed

- *(interpreter)* correct CreateContractStartingWithEF halt mapping ([#2890](https://github.com/bluealloy/revm/pull/2890))

## [25.0.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v25.0.0...revm-interpreter-v25.0.1) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context-interface

## [25.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v24.0.0...revm-interpreter-v25.0.0) - 2025-08-06

### Added

- short address for journal cold/warm check ([#2849](https://github.com/bluealloy/revm/pull/2849))
- gastable, record static gas in Interpreter loop ([#2822](https://github.com/bluealloy/revm/pull/2822))

### Fixed

- map new once and for all (+ci) ([#2852](https://github.com/bluealloy/revm/pull/2852))

### Other

- *(deps)* bump ruint ([#2811](https://github.com/bluealloy/revm/pull/2811))
- specialize halt, making instruction code very slightly smaller ([#2840](https://github.com/bluealloy/revm/pull/2840))
- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add debug assertions to set_action ([#2832](https://github.com/bluealloy/revm/pull/2832))
- improve ExtBytecode hash handling ([#2826](https://github.com/bluealloy/revm/pull/2826))
- fix inspector, cleanup loop ([#2797](https://github.com/bluealloy/revm/pull/2797))
- start InstructionResult at 1 ([#2802](https://github.com/bluealloy/revm/pull/2802))
- fix typos ([#2800](https://github.com/bluealloy/revm/pull/2800))
- improve inspector loop ([#2776](https://github.com/bluealloy/revm/pull/2776))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
- collapse debug info for interpreter macros ([#2780](https://github.com/bluealloy/revm/pull/2780))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [27.0.2](https://github.com/bluealloy/revm/compare/revm-interpreter-v27.0.1...revm-interpreter-v27.0.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-context-interface

## [27.0.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v27.0.0...revm-interpreter-v27.0.1) - 2025-10-15

### Fixed

- support legacy JumpTable serde format ([#3098](https://github.com/bluealloy/revm/pull/3098))

### Other

- make CallInput::bytes accept immutable ContextTr ([#3082](https://github.com/bluealloy/revm/pull/3082))

## [27.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v26.0.0...revm-interpreter-v27.0.0) - 2025-10-09

### Other

- remove deprecated methods ([#3050](https://github.com/bluealloy/revm/pull/3050))

## [26.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v25.0.3...revm-interpreter-v26.0.0) - 2025-10-07

### Added

- Support bubbling up first precompile error messages  ([#2905](https://github.com/bluealloy/revm/pull/2905))
- send bytecode with call input ([#2963](https://github.com/bluealloy/revm/pull/2963))

### Fixed

- remove redundant U256::from on Host getters in instructions ([#3053](https://github.com/bluealloy/revm/pull/3053))
- *(interpreter)* remove redundant stack underflow check in LOG instruction ([#3028](https://github.com/bluealloy/revm/pull/3028))
- unsafe stack capacity invariant and serde deserialization assumptions ([#3025](https://github.com/bluealloy/revm/pull/3025))
- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))
- skip cold load on oog ([#2903](https://github.com/bluealloy/revm/pull/2903))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- expose stack data ([#3047](https://github.com/bluealloy/revm/pull/3047))
- use offset_from_unsigned ([#2999](https://github.com/bluealloy/revm/pull/2999))
- rm eof comments ([#2987](https://github.com/bluealloy/revm/pull/2987))
- comments on EIP-2929/2930 constants ([#2969](https://github.com/bluealloy/revm/pull/2969))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- remove duplicate comment for TOTAL_COST_FLOOR_PER_TOKEN constant ([#2950](https://github.com/bluealloy/revm/pull/2950))
- clean static_selfdestruct_cost ([#2944](https://github.com/bluealloy/revm/pull/2944))
- rename SELFDESTRUCT to SELFDESTRUCT_REFUND ([#2937](https://github.com/bluealloy/revm/pull/2937))

## [25.0.3](https://github.com/bluealloy/revm/compare/revm-interpreter-v25.0.2...revm-interpreter-v25.0.3) - 2025-09-23

### Other

- updated the following local packages: revm-context-interface

## [24.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v23.0.2...revm-interpreter-v24.0.0) - 2025-07-23

### Added

- *(interpreter)* update CLZ cost ([#2739](https://github.com/bluealloy/revm/pull/2739))

### Fixed

- features and check in ci ([#2766](https://github.com/bluealloy/revm/pull/2766))

### Other

- un-Box frames ([#2761](https://github.com/bluealloy/revm/pull/2761))
- interpreter improvements ([#2760](https://github.com/bluealloy/revm/pull/2760))
- evaluate instruction table initializer at compile time ([#2762](https://github.com/bluealloy/revm/pull/2762))
- discard generic host implementation ([#2738](https://github.com/bluealloy/revm/pull/2738))
- add release safety section for SharedMemory fn ([#2718](https://github.com/bluealloy/revm/pull/2718))
- *(interpreter)* update docs for slice_mut and slice_range ([#2714](https://github.com/bluealloy/revm/pull/2714))

## [23.0.2](https://github.com/bluealloy/revm/compare/revm-interpreter-v23.0.1...revm-interpreter-v23.0.2) - 2025-07-14

### Other

- simplify gas calculations by introducing a used() method ([#2703](https://github.com/bluealloy/revm/pull/2703))

## [23.0.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v23.0.0...revm-interpreter-v23.0.1) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode, revm-context-interface

## [22.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v22.0.1...revm-interpreter-v22.1.0) - 2025-06-30

### Added

- blake2 avx2 ([#2670](https://github.com/bluealloy/revm/pull/2670))

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))

## [22.0.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v22.0.0...revm-interpreter-v22.0.1) - 2025-06-20

### Other

- updated the following local packages: revm-context-interface

## [22.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v21.0.0...revm-interpreter-v22.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/bluealloy/revm/pull/2611)) ([#2642](https://github.com/bluealloy/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- add clz opcode ([#2598](https://github.com/bluealloy/revm/pull/2598))
- added instruction_result fn in LoopControl trait  ([#2595](https://github.com/bluealloy/revm/pull/2595))

### Other

- lints handler inspector interpreter ([#2646](https://github.com/bluealloy/revm/pull/2646))
- bump all deps ([#2647](https://github.com/bluealloy/revm/pull/2647))
- re-use frame allocation ([#2636](https://github.com/bluealloy/revm/pull/2636))
- make CallInput default 0..0 ([#2621](https://github.com/bluealloy/revm/pull/2621))

## [21.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v20.0.0...revm-interpreter-v21.0.0) - 2025-06-06

### Added

- expand timestamp/block_number to u256 ([#2546](https://github.com/bluealloy/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/bluealloy/revm/pull/2560))
- simplify Interpreter loop ([#2544](https://github.com/bluealloy/revm/pull/2544))
- Add InstructionContext instead of Host and Interpreter ([#2548](https://github.com/bluealloy/revm/pull/2548))

## [20.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v19.1.0...revm-interpreter-v20.0.0) - 2025-05-22

### Added

- expose Gas::memory field ([#2512](https://github.com/bluealloy/revm/pull/2512))
- added CallInput::bytes<CTX>(ctx: &CTX) -> Bytes {} function ([#2507](https://github.com/bluealloy/revm/pull/2507))

### Other

- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- fix clippy ([#2523](https://github.com/bluealloy/revm/pull/2523))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [19.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v19.0.0...revm-interpreter-v19.1.0) - 2025-05-07

Dependency bump

## [19.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v18.0.0...revm-interpreter-v19.0.0) - 2025-05-07

### Added

- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/bluealloy/revm/pull/2462))
- Add a custom address to the CreateScheme. ([#2464](https://github.com/bluealloy/revm/pull/2464))
- replace input Bytes and refactored code where required ([#2453](https://github.com/bluealloy/revm/pull/2453))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/bluealloy/revm/pull/2382))

### Fixed

- *(inspector)* fix call return with Some ([#2469](https://github.com/bluealloy/revm/pull/2469))
- skip account list for legacy ([#2400](https://github.com/bluealloy/revm/pull/2400))

### Other

- Add Bytecode address to Interpreter ([#2479](https://github.com/bluealloy/revm/pull/2479))
- make ReturnDataImpl and LoopControlImpl public ([#2470](https://github.com/bluealloy/revm/pull/2470))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- Replace InterpreterAction with InterpreterTypes::Output ([#2424](https://github.com/bluealloy/revm/pull/2424))
- simplify reading signed integers ([#2456](https://github.com/bluealloy/revm/pull/2456))
- *(revm-interpreter)* remove unused deps ([#2447](https://github.com/bluealloy/revm/pull/2447))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [18.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v17.0.0...revm-interpreter-v18.0.0) - 2025-04-09

### Added

- *(tests)* Add dupn, swapn and exhange tests ([#2343](https://github.com/bluealloy/revm/pull/2343))
- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))

### Other

- *(test)* uncommented bitwise.rs and system.rs tests ([#2370](https://github.com/bluealloy/revm/pull/2370))
- *(tests)* fix program counter for eof jump instructions ([#2368](https://github.com/bluealloy/revm/pull/2368))
- fix tests in data.rs file ([#2365](https://github.com/bluealloy/revm/pull/2365))
- remove redundant U256 conversions in instructions ([#2364](https://github.com/bluealloy/revm/pull/2364))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0...revm-interpreter-v17.0.0) - 2025-03-28

### Other

- remove redundant clone ([#2293](https://github.com/bluealloy/revm/pull/2293))
- Remove LATEST variant from SpecId enum ([#2299](https://github.com/bluealloy/revm/pull/2299))
- make number more readable ([#2300](https://github.com/bluealloy/revm/pull/2300))

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.7...revm-interpreter-v16.0.0) - 2025-03-24

Stable version

## [16.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.6...revm-interpreter-v16.0.0-alpha.7) - 2025-03-21

### Added

- allow reuse of API for calculating initial tx gas for tx ([#2215](https://github.com/bluealloy/revm/pull/2215))

### Other

- make clippy happy ([#2274](https://github.com/bluealloy/revm/pull/2274))
- fix clippy ([#2238](https://github.com/bluealloy/revm/pull/2238))

## [16.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.5...revm-interpreter-v16.0.0-alpha.6) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context-interface

## [16.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.4...revm-interpreter-v16.0.0-alpha.5) - 2025-03-12

### Added

- add custom error to context ([#2197](https://github.com/bluealloy/revm/pull/2197))

## [16.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.3...revm-interpreter-v16.0.0-alpha.4) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [16.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.2...revm-interpreter-v16.0.0-alpha.3) - 2025-03-10

### Fixed

- set zero if blockhash is out of range ([#2173](https://github.com/bluealloy/revm/pull/2173))

## [16.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-interpreter-v16.0.0-alpha.1...revm-interpreter-v16.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))
- Standalone Host, remove default fn from context ([#2147](https://github.com/bluealloy/revm/pull/2147))
- allow host to be implemented on custom context ([#2112](https://github.com/bluealloy/revm/pull/2112))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- add immutable gas API to LoopControl ([#2134](https://github.com/bluealloy/revm/pull/2134))
- expose popn macros ([#2113](https://github.com/bluealloy/revm/pull/2113))
- Add docs to revm-bytecode crate ([#2108](https://github.com/bluealloy/revm/pull/2108))
- fix wrong comment & remove useless struct ([#2105](https://github.com/bluealloy/revm/pull/2105))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [16.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v15.2.0...revm-interpreter-v16.0.0-alpha.1) - 2025-02-16

### Added

- Evm structure (Cached Instructions and Precompiles) (#2049)
- Add essential EIP-7756 tracing fields (#2023)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- align Block trait (#1957)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- *(interpreter)* impl Clone for Stack (#1820)
- restructuring Part6 transaction crate (#1814)
- Merge validation/analyzis with Bytecode (#1793)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Fixed

- make macro crate-agnostic (#1802)

### Other

- backport op l1 fetch perf (#2076)
- add default generics for InterpreterTypes (#2070)
- Check performance of gas with i64 [#1884](https://github.com/bluealloy/revm/pull/1884) ([#2062](https://github.com/bluealloy/revm/pull/2062))
- Bump licence year to 2025 (#2058)
- relax halt reason bounds (#2041)
- remove duplicate instructions (#2029)
- align crates versions (#1983)
- Add bytecode hash in interpreter [#1888](https://github.com/bluealloy/revm/pull/1888) ([#1952](https://github.com/bluealloy/revm/pull/1952))
- Make inspector use generics, rm associated types (#1934)
- use MemoryOOG (#1941)
- fix comments and docs into more sensible (#1920)
- Move CfgEnv from context-interface to context crate (#1910)
- implement serde for interpreter ([#1909](https://github.com/bluealloy/revm/pull/1909))
- make ExtBytecode pointer private (#1904)
- fix typos (#1868)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
- refactor -copy common code (#1799)
- add ReentrancySentryOOG for SSTORE (#1795)
- simplify SuccessOrHalt trait bound (#1768)

## [15.2.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v15.1.0...revm-interpreter-v15.2.0) - 2025-02-11

### Other

- revm v19.4.0 tag v54

## [15.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v15.0.0...revm-interpreter-v15.1.0) - 2025-01-13

### Added

- *(EIP-7623)* adjuct floor gas check order ([#1990](https://github.com/bluealloy/revm/pull/1990))

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v14.0.0...revm-interpreter-v15.0.0) - 2024-12-26

### Added

- apply latest EIP-7702 changes ([#1850](https://github.com/bluealloy/revm/pull/1850))
- *(Prague)* EIP-7623 Increase Calldata Cost ([#1744](https://github.com/bluealloy/revm/pull/1744))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v13.0.0...revm-interpreter-v14.0.0) - 2024-11-06

### Other

- bump alloy-eip7702 and remove `Parity` re-export ([#1842](https://github.com/bluealloy/revm/pull/1842))

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v12.0.0...revm-interpreter-v13.0.0) - 2024-10-23

### Other

- updated the following local packages: revm-primitives

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v11.0.0...revm-interpreter-v12.0.0) - 2024-10-17

### Other

- updated the following local packages: revm-primitives

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v10.0.3...revm-interpreter-v11.0.0) - 2024-10-17

### Other

- updated the following local packages: revm-primitives

## [10.0.3](https://github.com/bluealloy/revm/compare/revm-interpreter-v10.0.2...revm-interpreter-v10.0.3) - 2024-09-26

### Other

- updated the following local packages: revm-primitives

## [10.0.2](https://github.com/bluealloy/revm/compare/revm-interpreter-v10.0.1...revm-interpreter-v10.0.2) - 2024-09-18

### Other

- make clippy happy ([#1755](https://github.com/bluealloy/revm/pull/1755))

## [10.0.1](https://github.com/bluealloy/revm/compare/revm-interpreter-v10.0.0...revm-interpreter-v10.0.1) - 2024-08-30

### Other
- Bump new logo ([#1735](https://github.com/bluealloy/revm/pull/1735))

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v9.0.0...revm-interpreter-v10.0.0) - 2024-08-29

### Added
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/bluealloy/revm/pull/1695))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v8.1.0...revm-interpreter-v9.0.0) - 2024-08-08

### Added
- *(EOF)* add evmone test suite ([#1689](https://github.com/bluealloy/revm/pull/1689))
- check for typos in CI ([#1686](https://github.com/bluealloy/revm/pull/1686))
- *(EOF)* Add non-returning CALLF/JUMPF checks ([#1663](https://github.com/bluealloy/revm/pull/1663))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/bluealloy/revm/pull/1648))
- *(EOF)* implement std::error::Error trait for EofValidationError and EofError ([#1649](https://github.com/bluealloy/revm/pull/1649))
- *(interpreter)* derive traits on FunctionStack ([#1640](https://github.com/bluealloy/revm/pull/1640))

### Fixed
- add DATACOPY to OpCode::modifies_memory ([#1639](https://github.com/bluealloy/revm/pull/1639))
- *(EOF)* returning to non-returning jumpf, enable valition error ([#1664](https://github.com/bluealloy/revm/pull/1664))
- *(EOF)* Validate code access in stack ([#1659](https://github.com/bluealloy/revm/pull/1659))
- *(eof)* deny static context in EOFCREATE ([#1644](https://github.com/bluealloy/revm/pull/1644))

### Other
- improve `InstructionResult` documentation ([#1673](https://github.com/bluealloy/revm/pull/1673))
- Add EOF Layout Fuzz Loop to `revme bytecode` ([#1677](https://github.com/bluealloy/revm/pull/1677))
- *(eof)* Add opcodes that expand memory ([#1665](https://github.com/bluealloy/revm/pull/1665))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/bluealloy/revm/pull/1661))
- avoid cloning original_bytes ([#1646](https://github.com/bluealloy/revm/pull/1646))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/bluealloy/revm/pull/1621))
- *(eof)* avoid some allocations ([#1632](https://github.com/bluealloy/revm/pull/1632))
- bump versions bcs of primitives ([#1631](https://github.com/bluealloy/revm/pull/1631))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v7.0.0...revm-interpreter-v8.0.0) - 2024-07-16

### Added
- *(eof)* cli eof-validation ([#1622](https://github.com/bluealloy/revm/pull/1622))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/bluealloy/revm/pull/1558))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))
- *(EOF)* MIN_CALLEE_GAS light failure, static-mode check ([#1599](https://github.com/bluealloy/revm/pull/1599))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v6.0.0...revm-interpreter-v7.0.0) - 2024-07-08

### Added
- *(Precompiles)* Throw fatal error if c-kzg is disabled ([#1589](https://github.com/bluealloy/revm/pull/1589))
- add bytecode_address from CallInputs to Contract during construction. ([#1568](https://github.com/bluealloy/revm/pull/1568))
- support selfdestruct for dummyhost ([#1578](https://github.com/bluealloy/revm/pull/1578))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/bluealloy/revm/pull/1565))
- *(EOF)* disallow ExtDelegateCall to legacy bytecode ([#1572](https://github.com/bluealloy/revm/pull/1572))
- *(EOF)* Add target address expansion checks ([#1570](https://github.com/bluealloy/revm/pull/1570))

### Fixed
- *(eof)* ExtDelegateCall caller/target switch ([#1571](https://github.com/bluealloy/revm/pull/1571))

### Other
- *(README)* add rbuilder to used-by ([#1585](https://github.com/bluealloy/revm/pull/1585))
- use const blocks ([#1522](https://github.com/bluealloy/revm/pull/1522))
- fix compile for alloydb ([#1559](https://github.com/bluealloy/revm/pull/1559))
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/bluealloy/revm/pull/1505))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v5.0.0...revm-interpreter-v6.0.0) - 2024-06-20

### Added
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/bluealloy/revm/pull/1504))
- add helpers for working with instruction tables ([#1493](https://github.com/bluealloy/revm/pull/1493))
- *(EOF)* change oob behavior of RETURNDATALOAD and RETURNDATACOPY ([#1476](https://github.com/bluealloy/revm/pull/1476))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/bluealloy/revm/pull/1467))
- adjust gas-costs for EIP-2935 BLOCKHASH ([#1422](https://github.com/bluealloy/revm/pull/1422))
- add Opcode::modifies_memory back ([#1421](https://github.com/bluealloy/revm/pull/1421))
- *(EOF)* Add CALLF/JUMPF stack checks ([#1417](https://github.com/bluealloy/revm/pull/1417))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/bluealloy/revm/pull/1415))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- *(eof)* output gas for eofcreate ([#1540](https://github.com/bluealloy/revm/pull/1540))
- *(EOF)* set CallOrCreate result in EOFCREATE ([#1535](https://github.com/bluealloy/revm/pull/1535))
- *(EOF)* target needed for EOFCREATE created address ([#1536](https://github.com/bluealloy/revm/pull/1536))
- *(EOF)* ext*call return values ([#1515](https://github.com/bluealloy/revm/pull/1515))
- *(EOF)* Remove redundunt ext call gas cost ([#1513](https://github.com/bluealloy/revm/pull/1513))
- *(EOF)* add DATACOPY copy gas ([#1510](https://github.com/bluealloy/revm/pull/1510))
- *(EOF)* extstaticcall make static ([#1508](https://github.com/bluealloy/revm/pull/1508))
- *(EOF)* jumpf gas was changes ([#1507](https://github.com/bluealloy/revm/pull/1507))
- *(EOF)* panic on empty input range, and continue exec after eofcreate ([#1477](https://github.com/bluealloy/revm/pull/1477))
- *(eof)* EOFCREATE spend gas and apply 63/64 rule ([#1471](https://github.com/bluealloy/revm/pull/1471))
- *(stack)* pop with five items was not correct ([#1472](https://github.com/bluealloy/revm/pull/1472))
- *(EOF)* returncontract immediate is one byte ([#1468](https://github.com/bluealloy/revm/pull/1468))
- *(Interpreter)* wrong block number used ([#1458](https://github.com/bluealloy/revm/pull/1458))
- *(interpreter)* avoid overflow when checking if mem limit reached ([#1429](https://github.com/bluealloy/revm/pull/1429))
- blockchash for devnet-0  ([#1427](https://github.com/bluealloy/revm/pull/1427))

### Other
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- simplify Interpreter serde ([#1544](https://github.com/bluealloy/revm/pull/1544))
- *(interpreter)* use U256::arithmetic_shr in SAR ([#1525](https://github.com/bluealloy/revm/pull/1525))
- pluralize EOFCreateInput ([#1523](https://github.com/bluealloy/revm/pull/1523))
- added simular to used-by ([#1521](https://github.com/bluealloy/revm/pull/1521))
- Removed .clone() in ExecutionHandler::call, and reusing output buffer in Interpreter ([#1512](https://github.com/bluealloy/revm/pull/1512))
- *(revme)* add new line in revme EOF printer ([#1503](https://github.com/bluealloy/revm/pull/1503))
- remove old deprecated items ([#1489](https://github.com/bluealloy/revm/pull/1489))
- *(interpreter)* use max gas limit in `impl Default for Interpreter` ([#1478](https://github.com/bluealloy/revm/pull/1478))
- *(interpreter)* optimisation for BYTE, SHL, SHR and SAR ([#1418](https://github.com/bluealloy/revm/pull/1418))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))" ([#1426](https://github.com/bluealloy/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))
- *(EOF)* rename extcall opcode/names ([#1416](https://github.com/bluealloy/revm/pull/1416))
- point to gas! in Gas::record_cost ([#1413](https://github.com/bluealloy/revm/pull/1413))
- pop_address should use crate scope ([#1410](https://github.com/bluealloy/revm/pull/1410))
- Remove Host constrain from calc_call_gas ([#1409](https://github.com/bluealloy/revm/pull/1409))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v4.0.0...revm-interpreter-v5.0.0) - 2024-05-12

### Added
- implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))
- parse opcodes from strings ([#1358](https://github.com/bluealloy/revm/pull/1358))
- *(interpreter)* add helpers for spending all gas ([#1360](https://github.com/bluealloy/revm/pull/1360))
- add helper methods to CallInputs ([#1345](https://github.com/bluealloy/revm/pull/1345))
- *(revm)* make `ItemOrResult` serializable ([#1282](https://github.com/bluealloy/revm/pull/1282))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- *(interpreter)* derive Eq for InterpreterAction ([#1262](https://github.com/bluealloy/revm/pull/1262))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/bluealloy/revm/pull/1243))
- *(interpreter)* test Host object-safety, allow `dyn Host` in instructions ([#1245](https://github.com/bluealloy/revm/pull/1245))

### Fixed
- return the correct error in resize_memory ([#1359](https://github.com/bluealloy/revm/pull/1359))
- correct some stack IO ([#1302](https://github.com/bluealloy/revm/pull/1302))

### Other
- add Trin to used by list ([#1393](https://github.com/bluealloy/revm/pull/1393))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- remove unused file ([#1379](https://github.com/bluealloy/revm/pull/1379))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))
- re-use num_words in gas::cost_per_word ([#1371](https://github.com/bluealloy/revm/pull/1371))
- *(interpreter)* rewrite gas accounting for memory expansion ([#1361](https://github.com/bluealloy/revm/pull/1361))
- remove bounds check in DUP, SWAP/EXCHANGE ([#1346](https://github.com/bluealloy/revm/pull/1346))
- don't clone bytes in `Bytecode::bytes` ([#1344](https://github.com/bluealloy/revm/pull/1344))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/bluealloy/revm/pull/1307))
- *(interpreter)* rename some macros ([#1304](https://github.com/bluealloy/revm/pull/1304))
- *(interpreter)* remove EOF branch in CODE{SIZE,COPY} ([#1308](https://github.com/bluealloy/revm/pull/1308))
- fix some warnings ([#1305](https://github.com/bluealloy/revm/pull/1305))
- *(interpreter)* rename wrapping_* opcodes ([#1306](https://github.com/bluealloy/revm/pull/1306))
- Add the modifies_memory macro ([#1270](https://github.com/bluealloy/revm/pull/1270))
- *(interpreter)* use `pop_top!` where possible ([#1267](https://github.com/bluealloy/revm/pull/1267))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.4.0...revm-interpreter-v4.0.0) - 2024-04-02

### Added
- add tests for shift instructions ([#1254](https://github.com/bluealloy/revm/pull/1254))
- derive serde for OpCode, improve implementations ([#1215](https://github.com/bluealloy/revm/pull/1215))
- *(interpreter)* expose mutable access methods on stack and memory ([#1219](https://github.com/bluealloy/revm/pull/1219))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))
- move div by zero check from smod to i256_mod ([#1248](https://github.com/bluealloy/revm/pull/1248))
- *(interpreter)* unbox contract field ([#1228](https://github.com/bluealloy/revm/pull/1228))
- *(interpreter)* keep track of remaining gas rather than spent ([#1221](https://github.com/bluealloy/revm/pull/1221))
- *(interpreter)* don't run signextend with 31 too ([#1222](https://github.com/bluealloy/revm/pull/1222))

## [3.4.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.3.0...revm-interpreter-v3.4.0) - 2024-03-19

### Added
- *(interpreter)* export utility macros ([#1203](https://github.com/bluealloy/revm/pull/1203))
- add convert_boxed and insert_boxed for InstructionTable ([#1194](https://github.com/bluealloy/revm/pull/1194))
- optional nonce check ([#1195](https://github.com/bluealloy/revm/pull/1195))

### Other
- expose functionality for custom EVMs ([#1201](https://github.com/bluealloy/revm/pull/1201))
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))

## [3.3.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.2.0...revm-interpreter-v3.3.0) - 2024-03-08

### Added
- *(interpreter)* OpCode struct constants ([#1173](https://github.com/bluealloy/revm/pull/1173))


## [3.2.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.1.0...revm-interpreter-v3.2.0) - 2024-03-08

### Added
- add insert method on instruction table ([#1167](https://github.com/bluealloy/revm/pull/1167))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))

### Other
- *(interpreter)* use already-computed sign in SAR ([#1147](https://github.com/bluealloy/revm/pull/1147))
- *(interpreter)* factor out jump logic ([#1146](https://github.com/bluealloy/revm/pull/1146))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))

## [3.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v3.0.0...revm-interpreter-v3.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))

### Fixed
- replace tuple in sstore return with struct ([#1115](https://github.com/bluealloy/revm/pull/1115))
- *(db)* Set instruction result at outcome insert ([#1117](https://github.com/bluealloy/revm/pull/1117))

### Other
- adding more test for i256 ([#1090](https://github.com/bluealloy/revm/pull/1090))
- *(refactor)* Propagate fatal error ([#1116](https://github.com/bluealloy/revm/pull/1116))
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v2.1.0...revm-interpreter-v3.0.0) - 2024-02-17

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))

### Other
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- Add memory offset ([#1032](https://github.com/bluealloy/revm/pull/1032))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))

## [2.1.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v2.0.0...revm-interpreter-v2.1.0) - 2024-02-12

### Added
- *(interpreter)* relax `make_boxed_instruction_table::FN` to `FnMut` ([#1076](https://github.com/bluealloy/revm/pull/1076))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-interpreter-v1.3.0...revm-interpreter-v2.0.0) - 2024-02-07

Iterpreter will not be called in recursive calls but would return Action ( CALL/CREATE) that will be executed by the main loop.

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- add `BytecodeLocked::original_bytecode` ([#1037](https://github.com/bluealloy/revm/pull/1037))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- add some conversions to InstructionResult ([#910](https://github.com/bluealloy/revm/pull/910))
- implement Default for InstructionResult ([#878](https://github.com/bluealloy/revm/pull/878))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- add methods to `CreateInput` for calculating created address ([#793](https://github.com/bluealloy/revm/pull/793))

### Fixed
- *(Interpreter)* is_revert should call is_revert ([#1007](https://github.com/bluealloy/revm/pull/1007))
- cast overflow in 32-bits OS ([#978](https://github.com/bluealloy/revm/pull/978))
- dont calculate initcode keccak on CREATE ([#969](https://github.com/bluealloy/revm/pull/969))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- safer stack ([#879](https://github.com/bluealloy/revm/pull/879))
- *(interpreter)* Stack `push_slice` fix and dup with pointers ([#837](https://github.com/bluealloy/revm/pull/837))

### Other
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(Execution)* Granular handles create/call,call_return,insert_call_outcome ([#1024](https://github.com/bluealloy/revm/pull/1024))
- *(Interpreter)* Split calls to separate functions ([#1005](https://github.com/bluealloy/revm/pull/1005))
- expose InstructionResult getters in Interpreter result ([#1002](https://github.com/bluealloy/revm/pull/1002))
- *(Inspector)* add CallOutcome to call/call_end ([#985](https://github.com/bluealloy/revm/pull/985))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(Inspector)* Add CreateOutcome in create/create_end return ([#980](https://github.com/bluealloy/revm/pull/980))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* refactor sstore_cost ([#974](https://github.com/bluealloy/revm/pull/974))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(interpreter)* consistency in all_results_are_covered() ([#961](https://github.com/bluealloy/revm/pull/961))
- *(interpreter)* local return_error! macro ([#956](https://github.com/bluealloy/revm/pull/956))
- *(interpreter)* simplify the logic of calc.new_cost() ([#939](https://github.com/bluealloy/revm/pull/939))
- *(interpreter)* fix the name of the macro referenced by record_memory() ([#926](https://github.com/bluealloy/revm/pull/926))
- *(interpreter)* conditionally enable `optional_beneficiary_reward` ([#925](https://github.com/bluealloy/revm/pull/925))
- fix case for CreateInitCodeSizeLimit error ([#896](https://github.com/bluealloy/revm/pull/896))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- *(interpreter)* use the constants from primitives ([#861](https://github.com/bluealloy/revm/pull/861))
- review safety comments ([#811](https://github.com/bluealloy/revm/pull/811))
- rewrite `Stack::push_slice` to allow arbitrary lengths ([#812](https://github.com/bluealloy/revm/pull/812))
- make context memory pub ([#831](https://github.com/bluealloy/revm/pull/831))
- refactor main return to handle ([#808](https://github.com/bluealloy/revm/pull/808))
- *(SharedMemory)* small refactor; tests ([#806](https://github.com/bluealloy/revm/pull/806))
- use `array::from_fn` in `make_instruction_table` ([#809](https://github.com/bluealloy/revm/pull/809))
- make memory-limit private ([#796](https://github.com/bluealloy/revm/pull/796))
- Instruction table ([#759](https://github.com/bluealloy/revm/pull/759))
- Shared memory between calls ([#673](https://github.com/bluealloy/revm/pull/673))
- Fix typos ([#790](https://github.com/bluealloy/revm/pull/790))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v1.3.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>
* 1f86e45 - chore(deps): bump proptest from 1.2.0 to 1.3.1 (#763) (22 hours ago) <dependabot[bot]>

# v1.2.0
date: 28.09.2023

Summary:
* Cancun support:
  * EIP-7516: BLOBBASEFEE opcode
  * EIP-4844: Shard Blob Transactions
  * EIP-1153: Transient storage opcodes
  * EIP-5656: MCOPY - Memory copying instruction
* Rename `SHA3` to `KECCAK256`, this can potentially break some tracers.
* Refactor opcodes and Interpreter dispatch loop. Better performance.
* optimize stack usage for recursive `call` and `create` programs.
    This brings down the native stack usage as calls are in recursion.

Full git log:
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 2c556c0 - refactor: say "warm" instead of "hot" (#754) (25 hours ago) <Paul Razvan Berg>
* 8206193 - feat: add "kzg" as a separate feature (#746) (2 hours ago) <DaniPopes>
* 516f62c - perf(interpreter): remove dynamic dispatch from all instructions (#739) (5 days ago) <DaniPopes>
* 26af13e - EIP-7516: BLOBBASEFEE opcode (#721) (5 days ago) <rakita>
* 36e71fc - fix: dont override instruction result (#736) (6 days ago) <rakita>
* d926728 - perf: refactor interpreter internals and cleanup (#582) (6 days ago) <DaniPopes>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 190f90e - Never inline the prepare functions (#712) (2 weeks ago) <Valentin Mihov>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 616cc7e - chore(cfg): convert chain_id from u256 to u64 (#693) (3 weeks ago) <Lorenzo Feroleto>
* a95a298 - chore: accept byte slice as input (#700) (3 weeks ago) <Matthias Seitz>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* f2929ad - chore(deps): bump proptest-derive from 0.3.0 to 0.4.0 (#652) (4 weeks ago) <dependabot[bot]>
* 37b0192 - perf(interpreter): improve i256 instructions (#630) (4 weeks ago) <DaniPopes>
* 214e65d - chore(interpreter): improve gas calculations (#632) (5 weeks ago) <DaniPopes>
* 6b55b9c - feat(`interpreter`): add hash to bytecode (#628) (5 weeks ago) <evalir>
* 84a5e97 - chore(interpreter): use `let else` (#629) (5 weeks ago) <DaniPopes>
* e9d96cd - chore(interpreter): improve dummy host (#631) (5 weeks ago) <DaniPopes>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* bc4d203 - feat: remove unnecessary var and if branch in gas calc (#592) (7 weeks ago) <bemevolent>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 157ef36 - feat: introduce initcode size limit check taking config into account (#587) (7 weeks ago) <evalir>
* 12558c5 - fix: fix mcopy memory expansion. Add eth tests to ci (#586) (7 weeks ago) <rakita>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* c6c5e88 - make calc public  (#575) (8 weeks ago) <BrazilRaw>
* 0a739e4 - fix(interpreter): mcopy call order (#570) (8 weeks ago) <DaniPopes>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>
* 36de35b - feat: Rename all SHA3 opcodes to KECCAK256 (#514) (3 months ago) <Tung Bui (Leo)>
* 10f81ba - optimize stack usage for recursive `call` and `create` programs (#522) (3 months ago) <Valentin Mihov>
* c153428 - feat(cancun): EIP-5656: MCOPY - Memory copying instruction (#528) (3 months ago) <Waylon Jepsen>
* 51072e6 - consume all gas on invalid opcode (#500) (3 months ago) <teddav>
* ccd0298 - feat: add Memory::into_data (#516) (3 months ago) <Matthias Seitz>
* 69f417f - feat: simplify BYTE opcode (#512) (4 months ago) <teddav>
* c54f079 - fix: replace SHA3 with KECCAK256 opcode name (#511) (4 months ago) <Matthias Seitz>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* 6057cc2 - chore: refactor interpreter run and remove static flag (#481) (5 months ago) <rakita>


# v1.1.2
date: 03.05.2023

* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v1.1.1
date: 14.04.2023

Added back utility function:
* 7d9b38a - [Interpreter]: Add back `spec_gas_opcode` (#446) (9 days ago) <Enrique Ortiz>

# v1.1.0
date: 04.04.2023

Biggest changes are Shanghai support 08ce847 and removal of gas blocks f91d5f9.

Changelog:
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>

# v1.0.0
date: 29.01.2023

Interpreter was extracted from main revm crate at the revm v3.0.0 version.