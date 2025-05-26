# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-bytecode-v3.0.0...revm-bytecode-v4.0.0) - 2025-05-07

### Added

- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- *(bytecode)* improve legacy bytecode padding ([#2423](https://github.com/bluealloy/revm/pull/2423))

### Other

- typos ([#2474](https://github.com/bluealloy/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- simplify opcode iter ([#2458](https://github.com/bluealloy/revm/pull/2458))
- make OPCODE_INFO a static ([#2459](https://github.com/bluealloy/revm/pull/2459))
- simplify reading signed integers ([#2456](https://github.com/bluealloy/revm/pull/2456))
- cache and use JumpTable::default ([#2439](https://github.com/bluealloy/revm/pull/2439))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-bytecode-v2.0.0...revm-bytecode-v3.0.0) - 2025-04-09

### Added

- Iteration of bytecode opcodes ([#2349](https://github.com/bluealloy/revm/pull/2349))

### Other

- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0...revm-bytecode-v2.0.0) - 2025-03-28

### Fixed

- fix typo ([#2333](https://github.com/bluealloy/revm/pull/2333))

### Other

- move Eof::new to first place && add doc for some pub functions ([#2334](https://github.com/bluealloy/revm/pull/2334))
- remove outdated TODO comments ([#2325](https://github.com/bluealloy/revm/pull/2325))

## [1.0.0 ](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0-alpha.5...revm-bytecode-v1.0.0) - 2025-03-24

Stable version

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0-alpha.4...revm-bytecode-v1.0.0-alpha.5) - 2025-03-21

### Other

- updated the following local packages: revm-primitives

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0-alpha.3...revm-bytecode-v1.0.0-alpha.4) - 2025-03-16

### Fixed

- JumpMap from_slice requires len ([#2203](https://github.com/bluealloy/revm/pull/2203))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0-alpha.2...revm-bytecode-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-bytecode-v1.0.0-alpha.1...revm-bytecode-v1.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Fixed

- relax jumptable len check ([#2149](https://github.com/bluealloy/revm/pull/2149))

### Other

- Add docs to revm-bytecode crate ([#2108](https://github.com/bluealloy/revm/pull/2108))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-bytecode-v1.0.0-alpha.1) - 2025-02-16

### Added

- Add essential EIP-7756 tracing fields (#2023)
- EthHandler trait (#2001)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- *(database)* implement order-independent equality for Reverts (#1827)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- Merge validation/analyzis with Bytecode (#1793)
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
- align crates versions (#1983)
- fix comments and docs into more sensible (#1920)
- *(crates/bytecode)* fix some comments (#1851)
- some no_std cleanup (#1834)
- fix `constants` module typo (#1801)
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
