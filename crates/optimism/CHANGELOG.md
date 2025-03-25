# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.6...op-revm-v1.0.0) - 2025-03-24

### Other

- *(op-precompiles)* Add test for checking that op default precompiles is updated ([#2291](https://github.com/bluealloy/revm/pull/2291))
- *(op-precompiles)* Add missing g2 add tests ([#2253](https://github.com/bluealloy/revm/pull/2253))

## [1.0.0-alpha.6](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.5...op-revm-v1.0.0-alpha.6) - 2025-03-21

### Added

- InspectEvm fn renames, inspector docs, book cleanup ([#2275](https://github.com/bluealloy/revm/pull/2275))
- Return Fatal error on bls precompiles if in no_std ([#2249](https://github.com/bluealloy/revm/pull/2249))
- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/bluealloy/revm/pull/2233))

### Fixed

- *(op)* deposit txs are identifier 126 or 0x7e not 0x7f ([#2237](https://github.com/bluealloy/revm/pull/2237))

### Other

- bring operator fee fixes to trunk ([#2273](https://github.com/bluealloy/revm/pull/2273))
- *(op-test-cov)* Add test for serializing deposit transaction parts ([#2267](https://github.com/bluealloy/revm/pull/2267))
- *(op-precompiles)* Check subset of l1 precompiles in op ([#2204](https://github.com/bluealloy/revm/pull/2204))
- *(op-handler)* Add test for halted deposit tx post regolith ([#2269](https://github.com/bluealloy/revm/pull/2269))
- *(op)* Remove redundant trait DepositTransaction ([#2265](https://github.com/bluealloy/revm/pull/2265))
- Fix sys deposit tx gas test ([#2263](https://github.com/bluealloy/revm/pull/2263))
- remove wrong `&mut` and duplicated spec ([#2276](https://github.com/bluealloy/revm/pull/2276))
- *(op-precompiles)* clean up op tx tests ([#2242](https://github.com/bluealloy/revm/pull/2242))
- make str to SpecId conversion fallible ([#2236](https://github.com/bluealloy/revm/pull/2236))
- *(op-precompiles)* Add tests for bls12-381 map fp to g ([#2241](https://github.com/bluealloy/revm/pull/2241))
- add a safe blst wrapper ([#2223](https://github.com/bluealloy/revm/pull/2223))
- *(op-precompiles)* Reuse tests for bls12-381 msm tests for pairing ([#2239](https://github.com/bluealloy/revm/pull/2239))
- *(op-precompiles)* add bls12-381 g2 add and msm tests ([#2231](https://github.com/bluealloy/revm/pull/2231))
- *(op-precompiles)* Add test for g1 msm ([#2227](https://github.com/bluealloy/revm/pull/2227))
- simplify single UT for OpSpecId compatibility. ([#2216](https://github.com/bluealloy/revm/pull/2216))
- use AccessListItem associated type instead of AccessList ([#2214](https://github.com/bluealloy/revm/pull/2214))

## [1.0.0-alpha.5](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.4...op-revm-v1.0.0-alpha.5) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))
- add test for calling `bn128_pair` before and after granite ([#2200](https://github.com/bluealloy/revm/pull/2200))

### Other

- *(op-precompiles)* Add test for calling g1 add ([#2205](https://github.com/bluealloy/revm/pull/2205))
- *(op-test)* Clean up precompile tests ([#2206](https://github.com/bluealloy/revm/pull/2206))
- fix typo in method name ([#2202](https://github.com/bluealloy/revm/pull/2202))
- Add tests for checking fjord precompile activation ([#2199](https://github.com/bluealloy/revm/pull/2199))

## [1.0.0-alpha.4](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.3...op-revm-v1.0.0-alpha.4) - 2025-03-12

### Added

- Add tx/block to EvmExecution trait ([#2195](https://github.com/bluealloy/revm/pull/2195))
- rename inspect_previous to inspect_replay ([#2194](https://github.com/bluealloy/revm/pull/2194))

### Other

- add debug to precompiles type ([#2193](https://github.com/bluealloy/revm/pull/2193))

## [1.0.0-alpha.3](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.2...op-revm-v1.0.0-alpha.3) - 2025-03-11

### Fixed

- fix(op) enable proper precompiles p256 ([#2186](https://github.com/bluealloy/revm/pull/2186))
- *(op)* fix inspection call ([#2184](https://github.com/bluealloy/revm/pull/2184))
- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- remove CTX phantomdata from precompile providers ([#2178](https://github.com/bluealloy/revm/pull/2178))

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/op-revm-v1.0.0-alpha.1...op-revm-v1.0.0-alpha.2) - 2025-03-10

### Other

- updated the following local packages: revm

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-optimism-v1.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- Introduce Auth and AccessList traits (#2079)
- derive Eq for OpSpec (#2073)
- *(op)* Isthmus precompiles (#2054)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- simplify InspectorContext (#2036)
- Context execution (#2013)
- EthHandler trait (#2001)
- extract and export `estimate_tx_compressed_size` (#1985)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- Split inspector.rs (#1958)
- align Block trait (#1957)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- add isthmus spec (#1938)
- integrate codspeed (#1935)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- Restructuring Part3 inspector crate (#1788)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- introducing EvmWiring, a chain-specific configuration (#1672)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- make macro crate-agnostic (#1802)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- backport op l1 fetch perf (#2076)
- remove OpSpec (#2074)
- Add helpers with_inspector with_precompile (#2063)
- *(op)* backport isthmus operator fee (#2059)
- Bump licence year to 2025 (#2058)
- rename OpHaltReason (#2042)
- simplify some generics (#2032)
- align crates versions (#1983)
- Make inspector use generics, rm associated types (#1934)
- add OpTransaction conversion tests (#1939)
- fix comments and docs into more sensible (#1920)
- Rename PRAGUE_EOF to OSAKA (#1903)
- refactor L1BlockInfo::tx_estimated_size_fjord (#1856)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
- Test for l1 gas used and l1 fee for ecotone tx (#1748)
- *(deps)* bump anyhow from 1.0.88 to 1.0.89 (#1772)
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
