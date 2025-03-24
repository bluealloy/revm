# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.7...revm-precompile-v17.0.0) - 2025-03-24

### Added

- add bn_mul benchmark ([#2287](https://github.com/bluealloy/revm/pull/2287))

### Other

- *(op-precompiles)* Add missing g2 add tests ([#2253](https://github.com/bluealloy/revm/pull/2253))

## [17.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.6...revm-precompile-v17.0.0-alpha.7) - 2025-03-21

### Added

- Return Fatal error on bls precompiles if in no_std ([#2249](https://github.com/bluealloy/revm/pull/2249))
- bls special case G1/G2_MUL ([#2248](https://github.com/bluealloy/revm/pull/2248))
- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/bluealloy/revm/pull/2233))

### Other

- add early exit for no-ops in EIP1962 and EIP2537 ([#2271](https://github.com/bluealloy/revm/pull/2271))
- *(op-precompiles)* Check subset of l1 precompiles in op ([#2204](https://github.com/bluealloy/revm/pull/2204))
- Add `g1_mul`, `g1_add` and `read_scalar` methods into substrate wrapper for bn128 ([#2264](https://github.com/bluealloy/revm/pull/2264))
- *(op-precompiles)* clean up op tx tests ([#2242](https://github.com/bluealloy/revm/pull/2242))
- Adds a wrapper around substrate-bn for EIP196 ([#2258](https://github.com/bluealloy/revm/pull/2258))
- add invariant test for G1/G2 Mul ([#2247](https://github.com/bluealloy/revm/pull/2247))
- add documentation for the gas related constants for EIP2537 ([#2246](https://github.com/bluealloy/revm/pull/2246))
- add a safe blst wrapper ([#2223](https://github.com/bluealloy/revm/pull/2223))
- Remove redundant bls12-381 constants and cleanup naming ([#2235](https://github.com/bluealloy/revm/pull/2235))
- Add some documentation for the bls12-381 precompile constants ([#2222](https://github.com/bluealloy/revm/pull/2222))

## [17.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.5...revm-precompile-v17.0.0-alpha.6) - 2025-03-16

### Added

- *(docs)* MyEvm example and book cleanup ([#2218](https://github.com/bluealloy/revm/pull/2218))

## [17.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.4...revm-precompile-v17.0.0-alpha.5) - 2025-03-12

### Other

- updated the following local packages: revm-context-interface

## [17.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.3...revm-precompile-v17.0.0-alpha.4) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

### Other

- *(precompile)* refactor out msm helper ([#2179](https://github.com/bluealloy/revm/pull/2179))

## [17.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.2...revm-precompile-v17.0.0-alpha.3) - 2025-03-10

### Fixed

- *(precompiles)* add portable flag for bls ([#2174](https://github.com/bluealloy/revm/pull/2174))

### Other

- v59 release-plz update ([#2170](https://github.com/bluealloy/revm/pull/2170))

## [17.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-precompile-v17.0.0-alpha.1...revm-precompile-v17.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Fixed

- *(blst)* Fix blst wrong constants values and usage ([#2102](https://github.com/bluealloy/revm/pull/2102))

### Other

- simplify bn128::run_pair ([#2137](https://github.com/bluealloy/revm/pull/2137))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- PrecompileErrors to PrecompileError ([#2103](https://github.com/bluealloy/revm/pull/2103))
- Update broken link secp256r1.rs ([#2099](https://github.com/bluealloy/revm/pull/2099))
- G1_msm base gas fee const correction ([#2100](https://github.com/bluealloy/revm/pull/2100))
- Split blst constants out to individual file #2085
- *(deps)* bump breaking deps ([#2093](https://github.com/bluealloy/revm/pull/2093))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [17.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revm-precompile-v16.1.0...revm-precompile-v17.0.0-alpha.1) - 2025-02-16

### Added

- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- integrate codspeed (#1935)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Fixed

- *(Inspector)* frame_end called multiple times (#2037)

### Other

- backport op l1 fetch perf (#2076)
- Bump licence year to 2025 (#2058)
- bump kzg.rs (#2002)
- align crates versions (#1983)
- blst reprice, remove g1/g2 mul (#1981)
- integrate rust-secp256k1 (#1915)
- fix comments and docs into more sensible (#1920)
- Rename PRAGUE_EOF to OSAKA (#1903)
- *(precompile)* use secp256k1 global context for ecrecover (#1843)
- fix some typos (#1800)
- Replace `PrecompileError` variant ([#1797](https://github.com/bluealloy/revm/pull/1797))
- *(deps)* bump once_cell from 1.19.0 to 1.20.0 (#1773)

# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [16.1.0](https://github.com/bluealloy/revm/compare/revm-precompile-v16.0.0...revm-precompile-v16.1.0) - 2025-02-11

### Other

- revm v19.4.0 tag v54

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v15.0.0...revm-precompile-v16.0.0) - 2024-12-26

### Added

- blst reprice, remove g1/g2 mul, eest test bump ([#1951](https://github.com/bluealloy/revm/pull/1951))
- add Isthmus spec ([#1948](https://github.com/bluealloy/revm/pull/1948))

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v14.0.0...revm-precompile-v15.0.0) - 2024-11-06

### Other

- bump alloy-eip7702 and remove `Parity` re-export ([#1842](https://github.com/bluealloy/revm/pull/1842))
- *(precompile)* use secp256k1 global context for ecrecover  ([#1845](https://github.com/bluealloy/revm/pull/1845))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v13.0.0...revm-precompile-v14.0.0) - 2024-10-23

### Other

- updated the following local packages: revm-primitives

## [13.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v12.0.0...revm-precompile-v13.0.0) - 2024-10-17

### Other

- updated the following local packages: revm-primitives

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v11.0.3...revm-precompile-v12.0.0) - 2024-10-17

### Added

- Rename PRAGUE_EOF to OSAKA ([#1822](https://github.com/bluealloy/revm/pull/1822))

## [11.0.3](https://github.com/bluealloy/revm/compare/revm-precompile-v11.0.2...revm-precompile-v11.0.3) - 2024-09-26

### Other

- updated the following local packages: revm-primitives

## [11.0.2](https://github.com/bluealloy/revm/compare/revm-precompile-v11.0.1...revm-precompile-v11.0.2) - 2024-09-18

### Other

- make clippy happy ([#1755](https://github.com/bluealloy/revm/pull/1755))

## [11.0.1](https://github.com/bluealloy/revm/compare/revm-precompile-v11.0.0...revm-precompile-v11.0.1) - 2024-08-30

### Other
- Bump new logo ([#1735](https://github.com/bluealloy/revm/pull/1735))
- bump kzg-rs version ([#1734](https://github.com/bluealloy/revm/pull/1734))

## [10.1.0](https://github.com/bluealloy/revm/compare/revm-precompile-v10.0.0...revm-precompile-v10.1.0) - 2024-08-29

### Added
- c-kzg bump, cleanup on kzgsetting ([#1719](https://github.com/bluealloy/revm/pull/1719))

### Other
- bump `kzg-rs` version ([#1726](https://github.com/bluealloy/revm/pull/1726))
- switch gas check order in blake2 precompile ([#1718](https://github.com/bluealloy/revm/pull/1718))

## [9.3.0](https://github.com/bluealloy/revm/compare/revm-precompile-v9.2.0...revm-precompile-v9.3.0) - 2024-08-08

### Added
- use batch bn256 pair operation ([#1643](https://github.com/bluealloy/revm/pull/1643))

### Other
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/bluealloy/revm/pull/1685))
- *(deps)* bump rstest from 0.21.0 to 0.22.0 ([#1681](https://github.com/bluealloy/revm/pull/1681))
- *(deps)* bump blst from 0.3.12 to 0.3.13 ([#1669](https://github.com/bluealloy/revm/pull/1669))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/bluealloy/revm/pull/1661))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))

## [9.1.0](https://github.com/bluealloy/revm/compare/revm-precompile-v9.0.0...revm-precompile-v9.1.0) - 2024-07-16

### Added
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/bluealloy/revm/pull/1558))

### Other
- main CHANGELOG ([#1592](https://github.com/bluealloy/revm/pull/1592))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v8.0.0...revm-precompile-v8.1.0) - 2024-07-08

### Added
- *(Precompiles)* Throw fatal error if c-kzg is disabled ([#1589](https://github.com/bluealloy/revm/pull/1589))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v7.0.0...revm-precompile-v8.0.0) - 2024-06-20

### Added
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/bluealloy/revm/pull/1499))
- add ecAdd to precompile bench ([#1496](https://github.com/bluealloy/revm/pull/1496))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/bluealloy/revm/pull/1436))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- check canonical Fp elements ([#1434](https://github.com/bluealloy/revm/pull/1434))
- *(precompile)* ignore infinity points in G1 MSM ([#1432](https://github.com/bluealloy/revm/pull/1432))
- *(precompile)* BLS G2 MSM ([#1428](https://github.com/bluealloy/revm/pull/1428))

### Other
- avoid cloning precompiles ([#1486](https://github.com/bluealloy/revm/pull/1486))
- *(precompiles)* Fix some nits in bls12_381 ([#1495](https://github.com/bluealloy/revm/pull/1495))
- *(deps)* allow multiple versions of secp256k1 ([#1490](https://github.com/bluealloy/revm/pull/1490))
- *(deps)* bump rstest from 0.19.0 to 0.21.0 ([#1482](https://github.com/bluealloy/revm/pull/1482))
- *(deps)* bump blst from 0.3.11 to 0.3.12 ([#1481](https://github.com/bluealloy/revm/pull/1481))
- add test for map_fp_to_g1 precompile ([#1465](https://github.com/bluealloy/revm/pull/1465))
- add docs for BLS scalar input decoding ([#1446](https://github.com/bluealloy/revm/pull/1446))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v6.0.0...revm-precompile-v7.0.0) - 2024-05-12

### Added
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bluealloy/revm/pull/1389))
- *(precompile)* add Prague hardfork specification ([#1387](https://github.com/bluealloy/revm/pull/1387))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))

### Fixed
- *(precompile)* blst dangling pointers, cleanup ([#1391](https://github.com/bluealloy/revm/pull/1391))
- *(precompile)* inherit Prague precompiles from Cancun ([#1392](https://github.com/bluealloy/revm/pull/1392))

### Other
- bump c-kzg to 1.0.2 ([#1390](https://github.com/bluealloy/revm/pull/1390))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- *(deps)* bump aurora-engine-modexp from 1.0.0 to 1.1.0 ([#1339](https://github.com/bluealloy/revm/pull/1339))
- *(deps)* bump secp256k1 from 0.28.2 to 0.29.0 ([#1260](https://github.com/bluealloy/revm/pull/1260))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v5.1.0...revm-precompile-v6.0.0) - 2024-04-02

### Fixed
- update/enable bn128 tests ([#1242](https://github.com/bluealloy/revm/pull/1242))
- use correct bn128 mul input length ([#1238](https://github.com/bluealloy/revm/pull/1238))
- use correct rand package for thread_rng ([#1233](https://github.com/bluealloy/revm/pull/1233))

### Other
- remove unnecessary call to into_u256() for bn128 add ([#1239](https://github.com/bluealloy/revm/pull/1239))

## [5.1.0](https://github.com/bluealloy/revm/compare/revm-precompile-v5.0.0...revm-precompile-v5.1.0) - 2024-03-19

### Added
- add benchmarks for ecpairing, kzg, ecrecover precompiles ([#1211](https://github.com/bluealloy/revm/pull/1211))
- use Message::from_digest in secp256k1 ecrecover ([#1199](https://github.com/bluealloy/revm/pull/1199))

### Other
- expose functionality for custom EVMs ([#1201](https://github.com/bluealloy/revm/pull/1201))
- c-kzg v1.0.0 ([#1190](https://github.com/bluealloy/revm/pull/1190))
- fix some typos ([#1189](https://github.com/bluealloy/revm/pull/1189))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v4.1.0...revm-precompile-v5.0.0) - 2024-03-08

### Added
- *(precompile)* add a bool to bytes32 helper function ([#1170](https://github.com/bluealloy/revm/pull/1170))
- precompile with generic context ([#1155](https://github.com/bluealloy/revm/pull/1155))
- Add boxed precompile trait ([#1131](https://github.com/bluealloy/revm/pull/1131))

## [4.1.0](https://github.com/bluealloy/revm/compare/revm-precompile-v4.0.1...revm-precompile-v4.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))

### Other
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))

## [4.0.1](https://github.com/bluealloy/revm/compare/revm-precompile-v4.0.0...revm-precompile-v4.0.1) - 2024-02-17

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))

### Other
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v3.0.0...revm-precompile-v4.0.0) - 2024-02-12

### Other
- *(precompile)* don't allocate if padding is not needed ([#1075](https://github.com/bluealloy/revm/pull/1075))
- *(precompile)* simplify bn128 precompile implementations ([#1074](https://github.com/bluealloy/revm/pull/1074))
- *(precompile)* make use of padding utilities, simplify secp256k1 ([#1073](https://github.com/bluealloy/revm/pull/1073))
- precompile bn128 copy ([#1071](https://github.com/bluealloy/revm/pull/1071))
- *(revm)* Add helpers to Build Revm with Context ([#1068](https://github.com/bluealloy/revm/pull/1068))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-precompile-v2.2.0...revm-precompile-v3.0.0) - 2024-02-07

Precompiles are refactored from list to HashMap, this allows adding arbitrary precompiles to the list.

### Added
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- *(precompile)* use Aurora modexp lib. ([#769](https://github.com/bluealloy/revm/pull/769))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))

### Fixed
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))

### Other
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/bluealloy/revm/pull/1059))
- *(deps)* bump secp256k1 from 0.28.1 to 0.28.2 ([#1038](https://github.com/bluealloy/revm/pull/1038))
- *(Cancun)* update Cancun precompiles docs ([#1015](https://github.com/bluealloy/revm/pull/1015))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- *(deps)* bump k256 from 0.13.2 to 0.13.3 ([#959](https://github.com/bluealloy/revm/pull/959))
- *(deps)* bump secp256k1 from 0.28.0 to 0.28.1 ([#954](https://github.com/bluealloy/revm/pull/954))
- *(deps)* bump once_cell from 1.18.0 to 1.19.0 ([#908](https://github.com/bluealloy/revm/pull/908))
- bump k256 and use normalize_s ([#870](https://github.com/bluealloy/revm/pull/870))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- *(precompiles)* Make PrecompileWithAddress field public, from impl ([#857](https://github.com/bluealloy/revm/pull/857))
- change addresses to iterator and add into_addresses ([#855](https://github.com/bluealloy/revm/pull/855))
- bump c-kzg to v0.4.0 ([#849](https://github.com/bluealloy/revm/pull/849))
- Refactor precompile list from Hash to vec ([#823](https://github.com/bluealloy/revm/pull/823))
- *(eip4844)* update kzg trusted setup ([#822](https://github.com/bluealloy/revm/pull/822))
- secp256k1 from 0.27 to 0.28 ([#817](https://github.com/bluealloy/revm/pull/817))
- for now support 1.69 rust compiler ([#814](https://github.com/bluealloy/revm/pull/814))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v2.2.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>

# v2.1.0
date 28.09.2023

 Summary:
 * Cancun EIP-4844 precompile. It is behind `c-kzg` that is enabled by default
    the reason is that c-kzg fails to build on wasm and some docker images.
 * no_std support
 * small fixes to return out of gas for modepx and pairing precompiles.

Full git log:
* 4f916be - chore: bump c-kzg to create lib (#758) (5 hours ago) <rakita>
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* b9938a8 - chore(deps): bump sha2 from 0.10.7 to 0.10.8 (#752) (30 hours ago) <dependabot[bot]>
* 8206193 - feat: add "kzg" as a separate feature (#746) (2 hours ago) <DaniPopes>
* 73f6ad7 - modexp gas check (#737) (24 hours ago) <Alessandro Mazza>
* cb39117 - fix(eip4844): Pass eth tests, additional conditions added. (#735) (6 days ago) <rakita>
* fa13fea - (lorenzo/main) feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 175aaec - Removed the last dependencies breaking no-std build. (#669) (4 weeks ago) <Lucas Clemente Vella>
* 0fa4504 - fix: pairing cost formula  (#659) (4 weeks ago) <xkx>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* 5cdaa97 - chore: avoid unnecessary allocations (#581) (6 weeks ago) <DaniPopes>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>

# v2.0.3
date: 03.05.2023

Bump revm primitives.

# v2.0.2
date: 14.04.2023

* b2c5262 - fix: k256 compile error (#451) (7 days ago) <rakita>

# v2.0.1
date: 04.04.2023

Small changes

Changelog:
* 992a11c - (HEAD -> v/310, origin/lib_versions) bump all (89 minutes ago) <rakita>
* d935525 - chore(deps): bump secp256k1 from 0.26.0 to 0.27.0 (#429) (2 weeks ago) <dependabot[bot]>
* f2656b7 - chore: add primitive SpecId to precompile SpecId conversion (#408) (4 weeks ago) <Matthias Seitz>
# v2.0.0
date: 29.01.2023

Renamed to `revm-precompiles` from `revm_precompiles`

# v1.1.2
date: 22.11.2022

Bump dependency versions.

# v1.1.1
date: 06.09.2022

Small release:
* refactor(precompiles): Vec -> BTreeMap (#177) (3 weeks ago) <Alexey Shekhirin>
* Cache precompile map with once_cell
* Bump dependencies version

# v1.1.0
date: 11.06.2022

Small release:
* Bump k256,secp256 libs
* rename Byzantine to Byzantium

# v1.0.0
date: 30.04.2022

Promoting it to stable version, and i dont expect for precompiles to change in any significant way in future.

* propagate the back the error of Signature::try_from. Thanks to: Nicolas Trippar
* Updating dependency versions: secp256k1, k256,primitive_types
# v0.4.0
date: 20.1.2022

* Added feature for k256 lib. We now have choise to use bitcoin c lib and k256 for ecrecovery.

# v0.3.0

* switch stacks H256 with U256 
* Error type is changed to `Return` in revm so it is in precompiles.
# v0.2.0

Removed parity-crypto and use only needed secp256k1 lib. Added `ecrecover` feature to allow dissabling it for wasm windows builds.

# v0.1.0

Initial version.