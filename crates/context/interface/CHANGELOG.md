# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
