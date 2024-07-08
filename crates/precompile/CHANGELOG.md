# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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