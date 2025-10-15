# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [8.0.3](https://github.com/bluealloy/revm/compare/revm-database-interface-v8.0.2...revm-database-interface-v8.0.3) - 2025-10-15

### Other

- updated the following local packages: revm-state

## [8.0.2](https://github.com/bluealloy/revm/compare/revm-database-interface-v8.0.1...revm-database-interface-v8.0.2) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-state

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-database-interface-v8.0.0...revm-database-interface-v8.0.1) - 2025-10-09

### Other

- *(database)* remove unnecessary Send+Sync bounds from TryDatabaseCommit for Arc ([#3063](https://github.com/bluealloy/revm/pull/3063))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.5...revm-database-interface-v8.0.0) - 2025-10-07

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- *(database)* extract duplicate test balance constants ([#3017](https://github.com/bluealloy/revm/pull/3017))
- *(database)* avoid panic by conditionally using block_in_place ([#2927](https://github.com/bluealloy/revm/pull/2927))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))

## [7.0.5](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.4...revm-database-interface-v7.0.5) - 2025-08-23

### Other

- *(database)* remove unused dependencies ([#2885](https://github.com/bluealloy/revm/pull/2885))

## [7.0.4](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.3...revm-database-interface-v7.0.4) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-state

## [7.0.3](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.2...revm-database-interface-v7.0.3) - 2025-08-06

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))

## [7.0.2](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.1...revm-database-interface-v7.0.2) - 2025-07-23

### Other

- impl DatabaseRef for WrapDatabaseRef ([#2726](https://github.com/bluealloy/revm/pull/2726))

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-database-interface-v7.0.0...revm-database-interface-v7.0.1) - 2025-07-03

### Other

- updated the following local packages: revm-state

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v6.0.0...revm-database-interface-v7.0.0) - 2025-06-30

### Added

- implement Database traits for either::Either ([#2673](https://github.com/bluealloy/revm/pull/2673))

### Other

- fix copy-pasted inner doc comments ([#2663](https://github.com/bluealloy/revm/pull/2663))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v5.0.0...revm-database-interface-v6.0.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v4.0.1...revm-database-interface-v5.0.0) - 2025-06-06

### Added

- added TxEnv::new_bench() add util function ([#2556](https://github.com/bluealloy/revm/pull/2556))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/bluealloy/revm/pull/2568))

## [4.0.1](https://github.com/bluealloy/revm/compare/revm-database-interface-v4.0.0...revm-database-interface-v4.0.1) - 2025-05-22

### Other

- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v3.0.1...revm-database-interface-v4.0.0) - 2025-05-07

Dependency bump

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-database-interface-v3.0.0...revm-database-interface-v3.0.1) - 2025-05-07

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- remove alloy-sol-types deps ([#2411](https://github.com/bluealloy/revm/pull/2411))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v2.0.0...revm-database-interface-v3.0.0) - 2025-04-09

### Other

- clean unsed indicatif ([#2379](https://github.com/bluealloy/revm/pull/2379))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0...revm-database-interface-v2.0.0) - 2025-03-28

### Other

- Propagate asyncdb feature flag from database-interface to revm  ([#2310](https://github.com/bluealloy/revm/pull/2310))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0-alpha.54...revm-database-interface-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0-alpha.4...revm-database-interface-v1.0.0-alpha.5) - 2025-03-21

### Other

- updated the following local packages: revm-primitives

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0-alpha.3...revm-database-interface-v1.0.0-alpha.4) - 2025-03-16

### Other

- updated the following local packages: revm-primitives

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0-alpha.2...revm-database-interface-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- remove wrong Clone Macro in WrapDatabaseRef ([#2181](https://github.com/bluealloy/revm/pull/2181))
- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-database-interface-v1.0.0-alpha.1...revm-database-interface-v1.0.0-alpha.2) - 2025-03-10

### Added

- TryDatabaseCommit ([#2121](https://github.com/bluealloy/revm/pull/2121))

### Other

- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-database-interface-v1.0.0-alpha.1) - 2025-02-16

### Added

- Evm structure (Cached Instructions and Precompiles) (#2049)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- integrate codspeed (#1935)
- Restucturing Part7 Handler and Context rework (#1865)
- add support for async database (#1809)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
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
