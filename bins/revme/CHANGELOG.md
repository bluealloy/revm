# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [8.2.2](https://github.com/bluealloy/revm/compare/revme-v8.2.1...revme-v8.2.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-inspector, revm, revm-statetest-types

## [8.2.1](https://github.com/bluealloy/revm/compare/revme-v8.2.0...revme-v8.2.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context, revm-state, revm-database-interface, revm-context-interface, revm-database, revm-inspector, revm, revm-statetest-types

## [8.2.0](https://github.com/bluealloy/revm/compare/revme-v8.0.1...revme-v8.2.0) - 2025-10-09

## [8.0.1](https://github.com/bluealloy/revm/compare/revme-v8.0.0...revme-v8.0.1) - 2025-10-09

### Other

- use NoOpInspector for inspector benches ([#3060](https://github.com/bluealloy/revm/pull/3060))

## [8.0.0](https://github.com/bluealloy/revm/compare/revme-v7.2.3...revme-v8.0.0) - 2025-10-07

### Added

- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- *(revme)* Insert block hashes in State ([#3024](https://github.com/bluealloy/revm/pull/3024))
- support 0x prefix in evmrunner hex input ([#2970](https://github.com/bluealloy/revm/pull/2970))
- *(revme)* Avoid panic on non-UTF filenames in statetest runner ([#2948](https://github.com/bluealloy/revm/pull/2948))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- pretty print state in revme statetest ([#2979](https://github.com/bluealloy/revm/pull/2979))
- Fix CLI exit code for invalid bytecode input ([#2968](https://github.com/bluealloy/revm/pull/2968))

## [7.2.3](https://github.com/bluealloy/revm/compare/revme-v7.2.2...revme-v7.2.3) - 2025-09-23

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-inspector, revm, revm-statetest-types

## [7.2.2](https://github.com/bluealloy/revm/compare/revme-v7.2.1...revme-v7.2.2) - 2025-08-23

### Other

- updated the following local packages: revm-bytecode, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-state, revm-inspector, revm, revm-statetest-types

## [7.2.1](https://github.com/bluealloy/revm/compare/revme-v7.2.0...revme-v7.2.1) - 2025-08-12

### Other

- codspeed sstore sload opcodes ([#2881](https://github.com/bluealloy/revm/pull/2881))

## [7.2.0](https://github.com/bluealloy/revm/compare/revme-v7.1.0...revme-v7.2.0) - 2025-08-06

### Added

- Reuse bls12-381 codepaths to implement kzg point evaluation precompile ([#2809](https://github.com/bluealloy/revm/pull/2809))

### Other

- *(benches)* rename anaysis-inspector to snailtracer-inspect ([#2834](https://github.com/bluealloy/revm/pull/2834))
- *(benches)* clean up criterion callsites ([#2833](https://github.com/bluealloy/revm/pull/2833))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
- fix clippy ([#2785](https://github.com/bluealloy/revm/pull/2785))
- add gas_limit to revme evm ([#2779](https://github.com/bluealloy/revm/pull/2779))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [7.1.0](https://github.com/bluealloy/revm/compare/revme-v7.0.4...revme-v7.1.0) - 2025-07-23

### Added

- count inspector and bench test ([#2730](https://github.com/bluealloy/revm/pull/2730))

### Fixed

- fully deprecate serde-json ([#2767](https://github.com/bluealloy/revm/pull/2767))

### Other

- back to hashbrown map in revme ([#2770](https://github.com/bluealloy/revm/pull/2770))
- back to better map ([#2768](https://github.com/bluealloy/revm/pull/2768))
- bump develop statetests to devnet-3 ([#2754](https://github.com/bluealloy/revm/pull/2754))
- add clz_50 codspeed ([#2743](https://github.com/bluealloy/revm/pull/2743))

## [7.0.4](https://github.com/bluealloy/revm/compare/revme-v7.0.3...revme-v7.0.4) - 2025-07-14

### Other

- incorrect StorageKey and StorageValue parameter order in burntpix benchmark ([#2704](https://github.com/bluealloy/revm/pull/2704))

## [7.0.3](https://github.com/bluealloy/revm/compare/revme-v7.0.2...revme-v7.0.3) - 2025-07-03

### Other

- update Cargo.lock dependencies

## [7.0.2](https://github.com/bluealloy/revm/compare/revme-v7.0.1...revme-v7.0.2) - 2025-06-30

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))
- statetest runner cleanup ([#2665](https://github.com/bluealloy/revm/pull/2665))
- use TxEnv::builder ([#2652](https://github.com/bluealloy/revm/pull/2652))

## [7.0.1](https://github.com/bluealloy/revm/compare/revme-v7.0.0...revme-v7.0.1) - 2025-06-20

### Other

- updated the following local packages: revm-context-interface, revm-context, revm, revm-inspector, revm-statetest-types

## [7.0.0](https://github.com/bluealloy/revm/compare/revme-v6.0.0...revme-v7.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- change blob_max_count to max_blobs_per_tx ([#2608](https://github.com/bluealloy/revm/pull/2608))

### Other

- re-use frame allocation ([#2636](https://github.com/bluealloy/revm/pull/2636))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [6.0.0](https://github.com/bluealloy/revm/compare/revme-v5.1.1...revme-v6.0.0) - 2025-06-06

### Added

- Config blob basefee fraction ([#2551](https://github.com/bluealloy/revm/pull/2551))
- expand timestamp/block_number to u256 ([#2546](https://github.com/bluealloy/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))
- use iter_batched for revme benches ([#2584](https://github.com/bluealloy/revm/pull/2584))
- unify calling of journal account loading ([#2561](https://github.com/bluealloy/revm/pull/2561))
- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/bluealloy/revm/pull/2560))
- *(cfg)* add tx_chain_id_check fields. Optimize effective gas cost calc ([#2557](https://github.com/bluealloy/revm/pull/2557))
- add gas-cost-estimator selected bytecodes ([#2555](https://github.com/bluealloy/revm/pull/2555))

## [5.1.1](https://github.com/bluealloy/revm/compare/revme-v5.1.0...revme-v5.1.1) - 2025-05-31

### Other

- unify calling of journal account loading

## [5.1.0](https://github.com/bluealloy/revm/compare/revme-v5.0.0...revme-v5.1.0) - 2025-05-22

### Added

- make blob max number optional ([#2532](https://github.com/bluealloy/revm/pull/2532))

### Other

- Change legacy statetests repo ([#2519](https://github.com/bluealloy/revm/pull/2519))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))

## [5.0.0](https://github.com/bluealloy/revm/compare/revme-v4.1.0...revme-v5.0.0) - 2025-05-07

Dependency bump

## [4.1.0](https://github.com/bluealloy/revm/compare/revme-v4.0.2...revme-v4.1.0) - 2025-05-07

### Added

- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/bluealloy/revm/pull/2462))
- *(tx)* Add Either RecoveredAuthorization ([#2448](https://github.com/bluealloy/revm/pull/2448))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/bluealloy/revm/pull/2382))

### Fixed

- *(revme)* evm command caller/target for bench ([#2476](https://github.com/bluealloy/revm/pull/2476))

### Other

- *(revme)* simplify repeated code ([#2432](https://github.com/bluealloy/revm/pull/2432))
- backport from release branch ([#2415](https://github.com/bluealloy/revm/pull/2415)) ([#2416](https://github.com/bluealloy/revm/pull/2416))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [4.0.2](https://github.com/bluealloy/revm/compare/revme-v4.0.0..revme-v4.0.1) - 2025-04-15

### Other


## [4.0.1](https://github.com/bluealloy/revm/compare/revme-v4.0.0...revme-v4.0.1) - 2025-04-09

### Other

- *(revme)* remove revm-handler dependency ([#2366](https://github.com/bluealloy/revm/pull/2366))

## [4.0.0](https://github.com/bluealloy/revm/compare/revme-v3.0.0...revme-v4.0.0) - 2025-03-28

### Added

- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))
- Add criterion to revme bench command ([#2295](https://github.com/bluealloy/revm/pull/2295))

### Other

- add criterion benchmark for evm build ([#2319](https://github.com/bluealloy/revm/pull/2319))
- add check for path and existence existence ([#2320](https://github.com/bluealloy/revm/pull/2320))
- bump bench time, and reduce burntpix iterations ([#2315](https://github.com/bluealloy/revm/pull/2315))
- update EOF validation logic and improve error handling ([#2304](https://github.com/bluealloy/revm/pull/2304))

## [3.0.0](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.7...revme-v3.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database

## [3.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.6...revme-v3.0.0-alpha.7) - 2025-03-21

### Added

- InspectEvm fn renames, inspector docs, book cleanup ([#2275](https://github.com/bluealloy/revm/pull/2275))

### Fixed

- correct eof kind in verification tests ([#2250](https://github.com/bluealloy/revm/pull/2250))

### Other

- *(revme)* remove deprecated #[clap] attribute

## [3.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.5...revme-v3.0.0-alpha.6) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context-interface, revm-context, revm-handler, revm-inspector

## [3.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.4...revme-v3.0.0-alpha.5) - 2025-03-12

### Added

- rename inspect_previous to inspect_replay ([#2194](https://github.com/bluealloy/revm/pull/2194))

## [3.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.3...revme-v3.0.0-alpha.4) - 2025-03-11

### Other

- Add comments to handler methods ([#2188](https://github.com/bluealloy/revm/pull/2188))

## [3.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.2...revme-v3.0.0-alpha.3) - 2025-03-10

### Other

- updated the following local packages: revm

## [3.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revme-v3.0.0-alpha.1...revme-v3.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- rename transact_previous to replay, move EvmTr traits ([#2153](https://github.com/bluealloy/revm/pull/2153))
- Add docs to revm-bytecode crate ([#2108](https://github.com/bluealloy/revm/pull/2108))
- *(deps)* bump breaking deps ([#2093](https://github.com/bluealloy/revm/pull/2093))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))

## [3.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revme-v2.5.0...revme-v3.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- integrate alloy-eips (#2078)
- *(op)* Isthmus precompiles (#2054)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- bump eof validation tests (#1963)
- simplify Transaction trait (#1959)
- Split inspector.rs (#1958)
- align Block trait (#1957)
- integrate codspeed (#1935)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- extract statetest models/structs to standalone crate (#1808)
- Merge validation/analyzis with Bytecode (#1793)
- restructure part3 fix examples  (#1792)
- Restructuring Part3 inspector crate (#1788)
- restructure Part2 database crate (#1784)
- use TestAuthorization and skip decoding of eip7702 tx (#1785)
- project restructuring Part1 (#1776)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Fixed

- *(revme)* Statetest stop exec when print output is true (#1995)
- *(revme)* statetest remove redundant json output (#1994)
- *(eof)* dont run precompile on ext delegate call (#1964)
- *(revme)* Burntpix bench (#1937)
- *(revme)* include correct bytecode for snailtracer  (#1917)
- statetest json set spec_id (#1766)

### Other

- set alpha.1 version
- Bump licence year to 2025 (#2058)
- improve EIP-3155 tracer (#2033)
- align crates versions (#1983)
- remove analysis bench inner loops (#1936)
- fix comments and docs into more sensible (#1920)
- tie journal database with database getter (#1923)
- use stderr for revme tracer. not panic on bytecode (#1916)
- put snailtracer and analysis contracts in files (#1911)
- Move CfgEnv from context-interface to context crate (#1910)
- Rename PRAGUE_EOF to OSAKA (#1903)
- bump EOF evmone tests to v0.13.0 (#1816)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
- *(revme)* replace `structopt` with `clap` (#1754)

## [2.5.0](https://github.com/bluealloy/revm/compare/revme-v2.4.0...revme-v2.5.0) - 2025-02-11

### Other

- updated the following local packages: revm

## [2.4.0](https://github.com/bluealloy/revm/compare/revme-v2.3.0...revme-v2.4.0) - 2025-01-28

### Other

- devnet5 tests v1.3.0 ([#2021](https://github.com/bluealloy/revm/pull/2021))

## [2.3.0](https://github.com/bluealloy/revm/compare/revme-v2.2.0...revme-v2.3.0) - 2025-01-13

### Added

- *(EIP-7623)* adjuct floor gas check order ([#1990](https://github.com/bluealloy/revm/pull/1990))

### Other

- v53 revm v19.2.0 ([#1972](https://github.com/bluealloy/revm/pull/1972))

## [2.2.0](https://github.com/bluealloy/revm/compare/revme-v2.1.0...revme-v2.2.0) - 2024-12-26

### Added

- blst reprice, remove g1/g2 mul, eest test bump ([#1951](https://github.com/bluealloy/revm/pull/1951))
- eip7691 fraction update ([#1900](https://github.com/bluealloy/revm/pull/1900))

### Other

- Uncouple blob count between CL and EL ([#1899](https://github.com/bluealloy/revm/pull/1899))

## [2.1.0](https://github.com/bluealloy/revm/compare/revme-v2.0.0...revme-v2.1.0) - 2024-11-06

### Other

- Osaka Activation (release_49 branch) ([#1835](https://github.com/bluealloy/revm/pull/1835))
- v49 release ([#1833](https://github.com/bluealloy/revm/pull/1833))

## [2.0.0](https://github.com/bluealloy/revm/compare/revme-v1.0.0...revme-v2.0.0) - 2024-10-23

### Other

- update Cargo.lock dependencies

## [1.0.0](https://github.com/bluealloy/revm/compare/revme-v0.11.0...revme-v1.0.0) - 2024-09-26

### Other

- update Cargo.lock dependencies

## [0.11.0](https://github.com/bluealloy/revm/compare/revme-v0.10.3...revme-v0.11.0) - 2024-10-17

### Added

- Rename PRAGUE_EOF to OSAKA ([#1822](https://github.com/bluealloy/revm/pull/1822))
- *(EIP-7702)* devnet-4 changes ([#1821](https://github.com/bluealloy/revm/pull/1821))

### Other

- remove test u8 check ([#1825](https://github.com/bluealloy/revm/pull/1825))

## [0.10.3](https://github.com/bluealloy/revm/compare/revme-v0.10.2...revme-v0.10.3) - 2024-09-26

### Other

- update Cargo.lock dependencies

## [0.10.2](https://github.com/bluealloy/revm/compare/revme-v0.10.1...revme-v0.10.2) - 2024-09-18

### Added

- *(statetest)* enable EOF in Prague tests ([#1753](https://github.com/bluealloy/revm/pull/1753))


## [0.10.1](https://github.com/bluealloy/revm/compare/revme-v0.10.0...revme-v0.10.1) - 2024-08-30

### Other
- updated the following local packages: revm

## [0.10.0](https://github.com/bluealloy/revm/compare/revme-v0.9.0...revme-v0.10.0) - 2024-08-29

### Added
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/bluealloy/revm/pull/1695))
- c-kzg bump, cleanup on kzgsetting ([#1719](https://github.com/bluealloy/revm/pull/1719))

## [0.9.0](https://github.com/bluealloy/revm/compare/revme-v0.8.0...revme-v0.9.0) - 2024-08-08

### Added
- *(EOF)* Run EOF tests from eth/tests ([#1690](https://github.com/bluealloy/revm/pull/1690))
- *(EOF)* add evmone test suite ([#1689](https://github.com/bluealloy/revm/pull/1689))
- *(EOF)* Add EOF validation in revme bytecode cmd ([#1660](https://github.com/bluealloy/revm/pull/1660))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/bluealloy/revm/pull/1648))

### Fixed
- *(statetest)* make bytecode analyzed ([#1666](https://github.com/bluealloy/revm/pull/1666))
- *(EOF)* returning to non-returning jumpf, enable valition error ([#1664](https://github.com/bluealloy/revm/pull/1664))
- *(statetest)* Add back Merge spec ([#1658](https://github.com/bluealloy/revm/pull/1658))

### Other
- Add EOF Layout Fuzz Loop to `revme bytecode` ([#1677](https://github.com/bluealloy/revm/pull/1677))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/bluealloy/revm/pull/1661))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))
- bump versions bcs of primitives ([#1631](https://github.com/bluealloy/revm/pull/1631))

## [0.8.0](https://github.com/bluealloy/revm/compare/revme-v0.7.0...revme-v0.8.0) - 2024-07-16

### Added
- *(eof)* cli eof-validation ([#1622](https://github.com/bluealloy/revm/pull/1622))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bluealloy/revm/pull/1607))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))

### Other
- *(GeneralState)* skip fewer specs ([#1603](https://github.com/bluealloy/revm/pull/1603))

## [0.7.0](https://github.com/bluealloy/revm/compare/revme-v0.6.0...revme-v0.7.0) - 2024-07-08

### Other
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))

## [0.6.0](https://github.com/bluealloy/revm/compare/revme-v0.5.0...revme-v0.6.0) - 2024-06-20

### Added
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(revme)* add prague spec ([#1506](https://github.com/bluealloy/revm/pull/1506))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))

### Other
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- skip tests with storage check and return status ([#1452](https://github.com/bluealloy/revm/pull/1452))

## [0.5.0](https://github.com/bluealloy/revm/compare/revme-v0.4.0...revme-v0.5.0) - 2024-05-12

### Added
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bluealloy/revm/pull/1389))
- add trace option in `revme evm` ([#1376](https://github.com/bluealloy/revm/pull/1376))
- *(revme)* add --keep-going to statetest command ([#1277](https://github.com/bluealloy/revm/pull/1277))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))

### Fixed
- *(revme)* Print one json outcome in statetest ([#1347](https://github.com/bluealloy/revm/pull/1347))
- Drops check for .json when testing a single file ([#1301](https://github.com/bluealloy/revm/pull/1301))

### Other
- *(revme)* increment statetest bar *after* running the test ([#1377](https://github.com/bluealloy/revm/pull/1377))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))

## [0.4.0](https://github.com/bluealloy/revm/compare/revme-v0.3.1...revme-v0.4.0) - 2024-04-02

### Added
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/bluealloy/revm/pull/1234))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))

## [0.3.1](https://github.com/bluealloy/revm/compare/revme-v0.3.0...revme-v0.3.1) - 2024-03-19

### Other
- tag v32 revm v7.1.0 ([#1176](https://github.com/bluealloy/revm/pull/1176))

## [0.3.0](https://github.com/bluealloy/revm/compare/revme-v0.2.2...revme-v0.3.0) - 2024-03-08

### Added
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- add evm script ([#1039](https://github.com/bluealloy/revm/pull/1039))

### Fixed
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/bluealloy/revm/pull/1169))

### Other
- *(deps)* bump walkdir from 2.4.0 to 2.5.0 ([#1149](https://github.com/bluealloy/revm/pull/1149))

## [0.2.2](https://github.com/bluealloy/revm/compare/revme-v0.2.1...revme-v0.2.2) - 2024-02-22

### Added
- split off serde_json dependency to its own feature ([#1104](https://github.com/bluealloy/revm/pull/1104))

## [0.2.1](https://github.com/bluealloy/revm/compare/revme-v0.2.0...revme-v0.2.1) - 2024-02-07

### Added
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(revme)* make it runnable by goevmlab ([#990](https://github.com/bluealloy/revm/pull/990))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- *(revme)* format kzg setup ([#818](https://github.com/bluealloy/revm/pull/818))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- Alloy primitives ([#724](https://github.com/bluealloy/revm/pull/724))
- implement EIP-4844 ([#668](https://github.com/bluealloy/revm/pull/668))
- *(StateBuilder)* switch builder option from without_bundle to with_bundle ([#688](https://github.com/bluealloy/revm/pull/688))
- alloy migration ([#535](https://github.com/bluealloy/revm/pull/535))
- State with account status ([#499](https://github.com/bluealloy/revm/pull/499))
- *(cancun)* EIP-5656: MCOPY - Memory copying instruction ([#528](https://github.com/bluealloy/revm/pull/528))
- json opcode traces EIP-3155 ([#356](https://github.com/bluealloy/revm/pull/356))
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode ([#376](https://github.com/bluealloy/revm/pull/376))
- revm-interpreter created ([#320](https://github.com/bluealloy/revm/pull/320))
- Export CustomPrinter insector from revm ([#300](https://github.com/bluealloy/revm/pull/300))
- substitute web3db to ethersdb ([#293](https://github.com/bluealloy/revm/pull/293))
- *(interpreter)* Unify instruction fn signature ([#283](https://github.com/bluealloy/revm/pull/283))
- *(revm)* Add prevrandao field to EnvBlock ([#271](https://github.com/bluealloy/revm/pull/271))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/bluealloy/revm/pull/239))
- *(revm, revme)* gas inspector ([#222](https://github.com/bluealloy/revm/pull/222))

### Fixed
- *(eip4844)* Pass eth tests, additional conditions added. ([#735](https://github.com/bluealloy/revm/pull/735))
- *(test)* Check expect exception and revm error ([#734](https://github.com/bluealloy/revm/pull/734))
- k256 compile error ([#451](https://github.com/bluealloy/revm/pull/451))

### Other
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/bluealloy/revm/pull/1050))
- *(Interpreter)* Split calls to separate functions ([#1005](https://github.com/bluealloy/revm/pull/1005))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/bluealloy/revm/pull/994))
- *(revme)* add recovery of address from secret key ([#992](https://github.com/bluealloy/revm/pull/992))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- *(docs)* revme readme update ([#898](https://github.com/bluealloy/revm/pull/898))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- decode KZG points directly into the buffers ([#840](https://github.com/bluealloy/revm/pull/840))
- bump v26 revm v3.5.0 ([#765](https://github.com/bluealloy/revm/pull/765))
- tag v25, revm v3.4.0 ([#755](https://github.com/bluealloy/revm/pull/755))
- BLOBBASEFEE opcode ([#721](https://github.com/bluealloy/revm/pull/721))
- Never inline the prepare functions ([#712](https://github.com/bluealloy/revm/pull/712))
- *(deps)* bump bytes from 1.4.0 to 1.5.0 ([#707](https://github.com/bluealloy/revm/pull/707))
- make `impl Default for StateBuilder` generic ([#690](https://github.com/bluealloy/revm/pull/690))
- *(deps)* bump walkdir from 2.3.3 to 2.4.0 ([#692](https://github.com/bluealloy/revm/pull/692))
- *(cfg)* convert chain_id from u256 to u64 ([#693](https://github.com/bluealloy/revm/pull/693))
- Revert "feat: alloy migration ([#535](https://github.com/bluealloy/revm/pull/535))" ([#616](https://github.com/bluealloy/revm/pull/616))
- spell check ([#615](https://github.com/bluealloy/revm/pull/615))
- avoid unnecessary allocations ([#581](https://github.com/bluealloy/revm/pull/581))
- clippy and fmt ([#568](https://github.com/bluealloy/revm/pull/568))
- optimize stack usage for recursive `call` and `create` programs ([#522](https://github.com/bluealloy/revm/pull/522))
- *(deps)* bump hashbrown from 0.13.2 to 0.14.0 ([#519](https://github.com/bluealloy/revm/pull/519))
- Bump v24, revm v3.3.0 ([#476](https://github.com/bluealloy/revm/pull/476))
- *(deps)* bump ruint from 1.7.0 to 1.8.0 ([#465](https://github.com/bluealloy/revm/pull/465))
- Release v23, revm v3.2.0 ([#464](https://github.com/bluealloy/revm/pull/464))
- Release v22, revm v3.1.1 ([#460](https://github.com/bluealloy/revm/pull/460))
- v21, revm v3.1.0 ([#444](https://github.com/bluealloy/revm/pull/444))
- bump all
- remove gas blocks ([#391](https://github.com/bluealloy/revm/pull/391))
- *(deps)* bump bytes from 1.3.0 to 1.4.0 ([#355](https://github.com/bluealloy/revm/pull/355))
- Bump v20, changelog ([#350](https://github.com/bluealloy/revm/pull/350))
- Cleanup imports ([#348](https://github.com/bluealloy/revm/pull/348))
- includes to libs ([#338](https://github.com/bluealloy/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/bluealloy/revm/pull/334))
- Correct typo ([#282](https://github.com/bluealloy/revm/pull/282))
- Integer overflow while calculating the remaining gas in GasInspector ([#287](https://github.com/bluealloy/revm/pull/287))
- native bits ([#278](https://github.com/bluealloy/revm/pull/278))
- *(release)* Bump revm and precompiles versions
- Bump primitive_types. Add statetest spec
- Bump revm to v2.3.0
- typos ([#263](https://github.com/bluealloy/revm/pull/263))
- *(eth/test)* Added OEF spec for tests. Skip HighGasPrice ([#261](https://github.com/bluealloy/revm/pull/261))
- Bump revm v2.1.0 ([#224](https://github.com/bluealloy/revm/pull/224))
# v0.1.0
date: 18.12.2021

Initial release. statetest are done, other things I have just started working on.