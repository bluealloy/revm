# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.5...revm-handler-v1.0.0-alpha.6) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.4...revm-handler-v1.0.0-alpha.5) - 2025-03-12

### Added

- add custom error to context ([#2197](https://github.com/bluealloy/revm/pull/2197))
- Add tx/block to EvmExecution trait ([#2195](https://github.com/bluealloy/revm/pull/2195))

### Other

- add debug to precompiles type ([#2193](https://github.com/bluealloy/revm/pull/2193))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.3...revm-handler-v1.0.0-alpha.4) - 2025-03-11

### Added

- decouple first_frame_input from inspector ([#2180](https://github.com/bluealloy/revm/pull/2180))

### Fixed

- *(op)* fix inspection call ([#2184](https://github.com/bluealloy/revm/pull/2184))
- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- Add comments to handler methods ([#2188](https://github.com/bluealloy/revm/pull/2188))
- remove CTX phantomdata from precompile providers ([#2178](https://github.com/bluealloy/revm/pull/2178))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.2...revm-handler-v1.0.0-alpha.3) - 2025-03-10

### Other

- updated the following local packages: revm-interpreter, revm-precompile

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.1...revm-handler-v1.0.0-alpha.2) - 2025-03-10

### Added

- *(handler)* add MainnetContext alias generic over Database ([#2166](https://github.com/bluealloy/revm/pull/2166))
- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Fixed

- *(op)* Handler deposit tx halt, catch_error handle ([#2144](https://github.com/bluealloy/revm/pull/2144))
- call clear ([#2091](https://github.com/bluealloy/revm/pull/2091))

### Other

- op-revm cleanup and few docs ([#2156](https://github.com/bluealloy/revm/pull/2156))
- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- rename transact_previous to replay, move EvmTr traits ([#2153](https://github.com/bluealloy/revm/pull/2153))
- docs and cleanup (rm Custom Inst) ([#2151](https://github.com/bluealloy/revm/pull/2151))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- add immutable gas API to LoopControl ([#2134](https://github.com/bluealloy/revm/pull/2134))
- PrecompileErrors to PrecompileError ([#2103](https://github.com/bluealloy/revm/pull/2103))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-handler-v1.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- Introduce Auth and AccessList traits (#2079)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Add essential EIP-7756 tracing fields (#2023)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- align Block trait (#1957)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- Make Ctx journal generic (#1933)
- removed create address collision check (#1928)
- Restucturing Part7 Handler and Context rework (#1865)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- *(Inspector)* call handler functions (#2026)
- deduplicate validate_initial_tx_gas API (#2006)
- *(eof)* dont run precompile on ext delegate call (#1964)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- backport op l1 fetch perf (#2076)
- relax trait req in EthPrecompiles::default (#2071)
- add default generics for InterpreterTypes (#2070)
- API cleanup (#2067)
- Add helpers with_inspector with_precompile (#2063)
- Add bytecode hash in interpreter [#1888](https://github.com/bluealloy/revm/pull/1888) ([#1952](https://github.com/bluealloy/revm/pull/1952))
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- Rename PRAGUE_EOF to OSAKA (#1903)
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
