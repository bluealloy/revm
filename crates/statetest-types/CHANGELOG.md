# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.1.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v4.0.0...revm-statetest-types-v4.1.0) - 2025-05-07

Dependency bump

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v3.0.1...revm-statetest-types-v4.0.0) - 2025-05-07

### Added

- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))

### Other

- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [3.0.1](https://github.com/bluealloy/revm/compare/revm-statetest-types-v3.0.0..revm-statetest-types-v3.0.1) - 2025-04-15

### Other

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v2.0.0...revm-statetest-types-v3.0.0) - 2025-04-09

### Other

- updated the following local packages: revm

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0...revm-statetest-types-v2.0.0) - 2025-03-28

### Other

- updated the following local packages: revm

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.4...revm-statetest-types-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.4...revm-statetest-types-v1.0.0-alpha.5) - 2025-03-11

### Other

- updated the following local packages: revm

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.3...revm-statetest-types-v1.0.0-alpha.4) - 2025-03-11

### Other

- updated the following local packages: revm

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.2...revm-statetest-types-v1.0.0-alpha.3) - 2025-03-10

### Other

- updated the following local packages: revm

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-statetest-types-v1.0.0-alpha.1...revm-statetest-types-v1.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- allow duplicate v and yparity in test files ([#2150](https://github.com/bluealloy/revm/pull/2150))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-statetest-types-v1.0.0-alpha.1) - 2025-02-16

### Added

- Introduce Auth and AccessList traits (#2079)
- integrate alloy-eips (#2078)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- simplify Transaction trait (#1959)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- extract statetest models/structs to standalone crate (#1808)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- *(eof)* dont run precompile on ext delegate call (#1964)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- bump devnet5 v1.3.0 tests (#2020)
- align crates versions (#1983)
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
