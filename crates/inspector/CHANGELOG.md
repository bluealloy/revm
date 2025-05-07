# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-inspector-v3.0.1...revm-inspector-v4.0.0) - 2025-05-07

### Fixed

- *(inspector)* call frame_end after frame_start returns Some ([#2481](https://github.com/bluealloy/revm/pull/2481))
- *(inspector)* fix call return with Some ([#2469](https://github.com/bluealloy/revm/pull/2469))
- *(tracing)* Fix the ordering of EOFCREATE frame traces ([#2398](https://github.com/bluealloy/revm/pull/2398))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- *(journal)* flatten journal entries ([#2440](https://github.com/bluealloy/revm/pull/2440))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/bluealloy/revm/pull/2415)) ([#2416](https://github.com/bluealloy/revm/pull/2416))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-inspector-v3.0.0...revm-inspector-v3.0.1) - 2025-04-15

### Other

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-inspector-v2.0.0...revm-inspector-v3.0.0) - 2025-04-09

### Added

- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0...revm-inspector-v2.0.0) - 2025-03-28

### Added

- cache precompile warming ([#2317](https://github.com/bluealloy/revm/pull/2317))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.7...revm-inspector-v1.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database

## [1.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.6...revm-inspector-v1.0.0-alpha.7) - 2025-03-21

### Added

- InspectEvm fn renames, inspector docs, book cleanup ([#2275](https://github.com/bluealloy/revm/pull/2275))
- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/bluealloy/revm/pull/2233))

### Other

- Add custom instruction example ([#2261](https://github.com/bluealloy/revm/pull/2261))

## [1.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.5...revm-inspector-v1.0.0-alpha.6) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.4...revm-inspector-v1.0.0-alpha.5) - 2025-03-12

### Added

- add custom error to context ([#2197](https://github.com/bluealloy/revm/pull/2197))
- Add tx/block to EvmExecution trait ([#2195](https://github.com/bluealloy/revm/pull/2195))
- rename inspect_previous to inspect_replay ([#2194](https://github.com/bluealloy/revm/pull/2194))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.3...revm-inspector-v1.0.0-alpha.4) - 2025-03-11

### Added

- decouple first_frame_input from inspector ([#2180](https://github.com/bluealloy/revm/pull/2180))

### Fixed

- remove wrong Clone Macro in WrapDatabaseRef ([#2181](https://github.com/bluealloy/revm/pull/2181))
- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- remove CTX phantomdata from precompile providers ([#2178](https://github.com/bluealloy/revm/pull/2178))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.2...revm-inspector-v1.0.0-alpha.3) - 2025-03-10

### Other

- updated the following local packages: revm-interpreter, revm-precompile

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-inspector-v1.0.0-alpha.1...revm-inspector-v1.0.0-alpha.2) - 2025-03-10

### Fixed

- *(op)* Handler deposit tx halt, catch_error handle ([#2144](https://github.com/bluealloy/revm/pull/2144))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- add immutable gas API to LoopControl ([#2134](https://github.com/bluealloy/revm/pull/2134))
- PrecompileErrors to PrecompileError ([#2103](https://github.com/bluealloy/revm/pull/2103))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- remove unused generics from TracerEip3155 ([#2090](https://github.com/bluealloy/revm/pull/2090))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-inspector-v1.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- simplify InspectorContext (#2036)
- Add essential EIP-7756 tracing fields (#2023)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- Split inspector.rs (#1958)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- Merge validation/analyzis with Bytecode (#1793)
- Restructuring Part3 inspector crate (#1788)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- *(tracer)* Flush buffer (#2080)
- *(Inspector)* frame_end called multiple times (#2037)
- *(Inspector)* call handler functions (#2026)
- Clear journal (#1927)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- improve EIP-3155 tracer (#2033)
- simplify some generics (#2032)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- add depth to GasInspector (#1922)
- Simplify GasInspector (#1919)
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
