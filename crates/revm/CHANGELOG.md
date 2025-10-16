# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [30.1.2](https://github.com/bluealloy/revm/compare/revm-v30.1.1...revm-v30.1.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-handler, revm-inspector

## [30.1.1](https://github.com/bluealloy/revm/compare/revm-v30.1.0...revm-v30.1.1) - 2025-10-15

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context, revm-interpreter, revm-precompile, revm-handler, revm-state, revm-database-interface, revm-context-interface, revm-database, revm-inspector

## [30.1.0](https://github.com/bluealloy/revm/compare/revm-v30.0.0...revm-v30.1.0) - 2025-10-09

### Other

- updated the following local packages: revm-database-interface, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector, revm-context-interface, revm-context

## [30.0.0](https://github.com/bluealloy/revm/compare/revm-v29.0.1...revm-v30.0.0) - 2025-10-07

### Added

- allow EIP-7623 to be disabled ([#2985](https://github.com/bluealloy/revm/pull/2985))
- [**breaking**] Remove kzg-rs ([#2909](https://github.com/bluealloy/revm/pull/2909))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- forward optional_fee_charge feature ([#3005](https://github.com/bluealloy/revm/pull/3005))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))

## [29.0.1](https://github.com/bluealloy/revm/compare/revm-v29.0.0...revm-v29.0.1) - 2025-09-23

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter, revm-handler, revm-inspector

## [29.0.0](https://github.com/bluealloy/revm/compare/revm-v28.0.1...revm-v29.0.0) - 2025-08-23

### Other

- updated the following local packages: revm-bytecode, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-state, revm-inspector

## [28.0.1](https://github.com/bluealloy/revm/compare/revm-v28.0.0...revm-v28.0.1) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-context-interface, revm-database, revm-precompile, revm-inspector, revm-database-interface, revm-context, revm-interpreter, revm-handler

## [28.0.0](https://github.com/bluealloy/revm/compare/revm-v27.1.0...revm-v28.0.0) - 2025-08-06

### Added

- Reuse bls12-381 codepaths to implement kzg point evaluation precompile ([#2809](https://github.com/bluealloy/revm/pull/2809))
- refactor test utils ([#2813](https://github.com/bluealloy/revm/pull/2813))
- add system transaction inspection support ([#2808](https://github.com/bluealloy/revm/pull/2808))

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- reuse global crypto provide idea ([#2786](https://github.com/bluealloy/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [27.1.0](https://github.com/bluealloy/revm/compare/revm-v27.0.3...revm-v27.1.0) - 2025-07-23

### Added

- expose sha3-keccak in revm and revm-primitives ([#2713](https://github.com/bluealloy/revm/pull/2713))

### Fixed

- features and check in ci ([#2766](https://github.com/bluealloy/revm/pull/2766))
- gas deduction with `disable_balance_check` ([#2699](https://github.com/bluealloy/revm/pull/2699))

### Other

- add asm-sha2 feature for sha2 precompile ([#2712](https://github.com/bluealloy/revm/pull/2712))

## [27.0.3](https://github.com/bluealloy/revm/compare/revm-v27.0.2...revm-v27.0.3) - 2025-07-14

### Other

- updated the following local packages: revm-context, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [27.0.2](https://github.com/bluealloy/revm/compare/revm-v27.0.1...revm-v27.0.2) - 2025-07-03

### Other

- updated the following local packages: revm-bytecode, revm-handler, revm-inspector, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter

## [27.0.1](https://github.com/bluealloy/revm/compare/revm-v26.0.1...revm-v27.0.1) - 2025-06-30

### Added

- optional_eip3541 ([#2661](https://github.com/bluealloy/revm/pull/2661))

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))
- inline documentation of revm top modules ([#2666](https://github.com/bluealloy/revm/pull/2666))
- use TxEnv::builder ([#2652](https://github.com/bluealloy/revm/pull/2652))

## [26.0.1](https://github.com/bluealloy/revm/compare/revm-v26.0.0...revm-v26.0.1) - 2025-06-20

### Fixed

- call stack_frame.clear() at end ([#2656](https://github.com/bluealloy/revm/pull/2656))

## [26.0.0](https://github.com/bluealloy/revm/compare/revm-v25.0.0...revm-v26.0.0) - 2025-06-19

### Added

- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- enable P256 in Osaka ([#2601](https://github.com/bluealloy/revm/pull/2601))

### Fixed

- compare_or_save_testdata sig ([#2637](https://github.com/bluealloy/revm/pull/2637))

### Other

- lints for examples ([#2650](https://github.com/bluealloy/revm/pull/2650))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [25.0.0](https://github.com/bluealloy/revm/compare/revm-v24.0.1...revm-v25.0.0) - 2025-06-06

### Added

- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Fixed

- *(multitx)* Add local flags for create and selfdestruct ([#2581](https://github.com/bluealloy/revm/pull/2581))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))

## [24.0.1](https://github.com/bluealloy/revm/compare/revm-v24.0.0...revm-v24.0.1) - 2025-05-31

### Other

- updated the following local packages: revm-context, revm-handler, revm-inspector

## [24.0.0](https://github.com/bluealloy/revm/compare/revm-v23.1.0...revm-v24.0.0) - 2025-05-22

### Other

- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))

## [23.1.0](https://github.com/bluealloy/revm/compare/revm-v23.0.0...revm-v23.1.0) - 2025-05-07

Dependency bump

## [23.0.0](https://github.com/bluealloy/revm/compare/revm-v22.0.1...revm-v23.0.0) - 2025-05-07

### Other

- typos ([#2474](https://github.com/bluealloy/revm/pull/2474))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))

## [22.0.1](https://github.com/bluealloy/revm/compare/revm-v22.0.0..revm-v22.0.1) - 2025-04-15

### Other

## [22.0.0](https://github.com/bluealloy/revm/compare/revm-v21.0.0...revm-v22.0.0) - 2025-04-09

### Added

- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))
- Reorder revm deps in Cargo.toml ([#2331](https://github.com/bluealloy/revm/pull/2331))

## [21.0.0](https://github.com/bluealloy/revm/compare/revm-v20.0.0...revm-v21.0.0) - 2025-03-28

### Added

- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))
- Add a wrapper for arkworks for EIP196 ([#2305](https://github.com/bluealloy/revm/pull/2305))

### Other

- Propagate asyncdb feature flag from database-interface to revm  ([#2310](https://github.com/bluealloy/revm/pull/2310))

## [20.0.0](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.7...revm-v20.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database, revm-precompile

## [20.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.6...revm-v20.0.0-alpha.7) - 2025-03-21

### Other

- updated the following local packages: revm-primitives, revm-context-interface, revm-context, revm-database, revm-interpreter, revm-precompile, revm-handler, revm-inspector

## [20.0.0-alpha.6](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.5...revm-v20.0.0-alpha.6) - 2025-03-16

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-context-interface, revm-context, revm-precompile, revm-handler, revm-inspector

## [20.0.0-alpha.5](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.4...revm-v20.0.0-alpha.5) - 2025-03-12

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter, revm-handler, revm-inspector

## [20.0.0-alpha.4](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.3...revm-v20.0.0-alpha.4) - 2025-03-11

### Fixed

- correct propagate features ([#2177](https://github.com/bluealloy/revm/pull/2177))

## [20.0.0-alpha.3](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.2...revm-v20.0.0-alpha.3) - 2025-03-10

### Fixed

- *(precompiles)* add portable flag for bls ([#2174](https://github.com/bluealloy/revm/pull/2174))

## [20.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-v20.0.0-alpha.1...revm-v20.0.0-alpha.2) - 2025-03-10

### Added

- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))

### Other

- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- examples to use main revm crate ([#2152](https://github.com/bluealloy/revm/pull/2152))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/bluealloy/revm/pull/2132))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [20.0.0-alpha.1](https://github.com/bluealloy/revm/compare/revm-v19.5.0...revm-v20.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- integrate alloy-eips (#2078)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7623)* adjuct floor gas check order (main) (#1991)
- *(EIP-7623)* Increase calldata cost. backport from rel/v51 (#1965)
- simplify Transaction trait (#1959)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- integrate codspeed (#1935)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- push NonceChange to Journal in deduct_caller (#1804)
- Merge validation/analyzis with Bytecode (#1793)
- Restructuring Part3 inspector crate (#1788)
- restructure Part2 database crate (#1784)
- project restructuring Part1 (#1776)
- to_plain_state (#1778)
- *(example)* deploy bytecode from scratch (#1767)
- introducing EvmWiring, a chain-specific configuration (#1672)

### Fixed

- make macro crate-agnostic (#1802)

### Other

- backport op l1 fetch perf (#2076)
- API cleanup (#2067)
- Add helpers with_inspector with_precompile (#2063)
- Bump licence year to 2025 (#2058)
- bump alloy versions to match latest (#2007)
- align crates versions (#1983)
- Make inspector use generics, rm associated types (#1934)
- fix comments and docs into more sensible (#1920)
- EVM transact, make output generic for POSTEXEC (#1931)
- Move CfgEnv from context-interface to context crate (#1910)
- bumps select alloy crates to 0.6 (#1854)
- some no_std cleanup (#1834)
- bump alloy to 0.4.2 (#1817)
- *(primitives)* replace HashMap re-exports with alloy_primitives::map (#1805)
- *(deps)* bump anyhow from 1.0.88 to 1.0.89 (#1772)
- simplify SuccessOrHalt trait bound (#1768)


## [19.5.0](https://github.com/bluealloy/revm/compare/revm-v19.4.0...revm-v19.5.0) - 2025-02-11

### Fixed

- *(op-isthmus)* missing granite precompiles and disable eof (#2065)
- *(op)* add operator fee scalar scaling factor on refund (#2050)
- fix ethersdb.rs forking ([#2047](https://github.com/bluealloy/revm/pull/2047))

## [19.4.0](https://github.com/bluealloy/revm/compare/revm-v19.3.0...revm-v19.4.0) - 2025-01-28

### Added

- Include EIP7702 address for extcodecopy ([#2016](https://github.com/bluealloy/revm/pull/2016))
- add precompiles for BLS12-381 to isthmus ([#2000](https://github.com/bluealloy/revm/pull/2000))

### Fixed

- *(op)* isthmus compilation ([#2022](https://github.com/bluealloy/revm/pull/2022))

## [19.3.0](https://github.com/bluealloy/revm/compare/revm-v19.2.0...revm-v19.3.0) - 2025-01-13

### Added

- *(EIP-7623)* adjuct floor gas check order ([#1990](https://github.com/bluealloy/revm/pull/1990))

## [19.2.0](https://github.com/bluealloy/revm/compare/revm-v19.1.0...revm-v19.2.0) - 2025-01-06

### Fixed

- *(op)* move l1block loading to verification ([#1970](https://github.com/bluealloy/revm/pull/1970))

## [19.1.0](https://github.com/bluealloy/revm/compare/revm-v19.0.0...revm-v19.1.0) - 2025-01-06

### Added

- handle isthmus operator fee ([#1960](https://github.com/bluealloy/revm/pull/1960))

### Other

- optimize tx l1 fetches ([#1967](https://github.com/bluealloy/revm/pull/1967))

## [19.0.0](https://github.com/bluealloy/revm/compare/revm-v18.0.0...revm-v19.0.0) - 2024-12-26

### Added

- apply latest EIP-7702 changes ([#1850](https://github.com/bluealloy/revm/pull/1850))
- *(Prague)* EIP-7623 Increase Calldata Cost ([#1744](https://github.com/bluealloy/revm/pull/1744))

### Other

- eip7702 chainid u256 change ([#1950](https://github.com/bluealloy/revm/pull/1950))
- refactor L1BlockInfo::tx_estimated_size_fjord ([#1862](https://github.com/bluealloy/revm/pull/1862))
- [v50] perf: remove unneeded clones ([#1863](https://github.com/bluealloy/revm/pull/1863))
- Add static Optimism precompiles for Fjord & Granite ([#1864](https://github.com/bluealloy/revm/pull/1864))

## [18.0.0](https://github.com/bluealloy/revm/compare/revm-v17.1.0...revm-v18.0.0) - 2024-11-06

### Added

- to_plain_state ([#1778](https://github.com/bluealloy/revm/pull/1778)) ([#1841](https://github.com/bluealloy/revm/pull/1841))

### Other

- bump alloy-eip7702 and remove `Parity` re-export ([#1842](https://github.com/bluealloy/revm/pull/1842))
- Osaka Activation (release_49 branch) ([#1835](https://github.com/bluealloy/revm/pull/1835))

## [17.1.0](https://github.com/bluealloy/revm/compare/revm-v17.0.0...revm-v17.1.0) - 2024-10-23

### Other

- chore: bump alloydb deps ([#1832](https://github.com/bluealloy/revm/pull/1832))

## [17.0.0](https://github.com/bluealloy/revm/compare/revm-v16.0.0...revm-v17.0.0) - 2024-10-23

### Other

- bump alloy-eip7702 ([#1829](https://github.com/bluealloy/revm/pull/1829))

## [16.0.0](https://github.com/bluealloy/revm/compare/revm-v15.0.0...revm-v16.0.0) - 2024-10-17

### Other

- update Cargo.toml dependencies

## [15.0.0](https://github.com/bluealloy/revm/compare/revm-v14.0.3...revm-v15.0.0) - 2024-10-17

### Added

- Rename PRAGUE_EOF to OSAKA ([#1822](https://github.com/bluealloy/revm/pull/1822))
- *(EIP-7702)* devnet-4 changes ([#1821](https://github.com/bluealloy/revm/pull/1821))

## [14.0.3](https://github.com/bluealloy/revm/compare/revm-v14.0.2...revm-v14.0.3) - 2024-09-26

### Other

- update Cargo.lock dependencies

## [14.0.2](https://github.com/bluealloy/revm/compare/revm-v14.0.1...revm-v14.0.2) - 2024-09-18

### Fixed

- *(inspector)* always call selfdestruct if entry is made ([#1746](https://github.com/bluealloy/revm/pull/1746))

### Other

- *(deps)* bump alloy-sol-types from 0.8.0 to 0.8.2 ([#1762](https://github.com/bluealloy/revm/pull/1762))
- *(deps)* bump anyhow from 1.0.86 to 1.0.87 ([#1760](https://github.com/bluealloy/revm/pull/1760))
- make clippy happy ([#1755](https://github.com/bluealloy/revm/pull/1755))
- Test l1 gas used fjord ([#1749](https://github.com/bluealloy/revm/pull/1749))
- Add test for `revm::optimism::L1BlockInfo::calculate_tx_l1_cost_fjord` ([#1743](https://github.com/bluealloy/revm/pull/1743))
- *(deps)* bump tokio from 1.39.2 to 1.40.0 ([#1739](https://github.com/bluealloy/revm/pull/1739))

## [14.0.1](https://github.com/bluealloy/revm/compare/revm-v14.0.0...revm-v14.0.1) - 2024-08-30

### Other
- Bump new logo ([#1735](https://github.com/bluealloy/revm/pull/1735))

## [14.0.0](https://github.com/bluealloy/revm/compare/revm-v13.0.0...revm-v14.0.0) - 2024-08-29

### Added
- *(eip7702)* Impl newest version of EIP  ([#1695](https://github.com/bluealloy/revm/pull/1695))

### Other
- *(deps)* bump alloy and primitives ([#1725](https://github.com/bluealloy/revm/pull/1725))
- cast block number to u64 and not usize ([#1727](https://github.com/bluealloy/revm/pull/1727))
- clean up some journalstate docs ([#1712](https://github.com/bluealloy/revm/pull/1712))
- update some docs related to state ([#1711](https://github.com/bluealloy/revm/pull/1711))

## [12.2.0](https://github.com/bluealloy/revm/compare/revm-v12.1.0...revm-v12.2.0) - 2024-08-08

### Added
- check for typos in CI ([#1686](https://github.com/bluealloy/revm/pull/1686))
- *(EOF)* EOF Validation add code type and sub container tracker ([#1648](https://github.com/bluealloy/revm/pull/1648))

### Other
- Add OP-Granite hardfork, limiting bn256Pairing input size ([#1685](https://github.com/bluealloy/revm/pull/1685))
- *(deps)* bump rstest from 0.21.0 to 0.22.0 ([#1681](https://github.com/bluealloy/revm/pull/1681))
- *(deps)* bump tokio from 1.38.1 to 1.39.2 ([#1668](https://github.com/bluealloy/revm/pull/1668))
- *(clippy)* 1.80 rust clippy list paragraph ident ([#1661](https://github.com/bluealloy/revm/pull/1661))
- avoid cloning original_bytes ([#1646](https://github.com/bluealloy/revm/pull/1646))
- use `is_zero` for `U256` and `B256` ([#1638](https://github.com/bluealloy/revm/pull/1638))
- fix some typos & remove useless Arc::clone ([#1621](https://github.com/bluealloy/revm/pull/1621))
- *(eof)* simplify magic checks ([#1633](https://github.com/bluealloy/revm/pull/1633))

## [12.0.0](https://github.com/bluealloy/revm/compare/revm-v11.0.0...revm-v12.0.0) - 2024-07-16

### Added
- pass interpreter into Inspector::log ([#1610](https://github.com/bluealloy/revm/pull/1610))
- *(EOF)* Bytecode::new_raw supports EOF, new_raw_checked added ([#1607](https://github.com/bluealloy/revm/pull/1607))
- use `kzg-rs` for kzg point evaluation ([#1558](https://github.com/bluealloy/revm/pull/1558))

### Fixed
- *(eip7702)* Add tests and fix some bugs ([#1605](https://github.com/bluealloy/revm/pull/1605))
- correctly calculate eofcreate address ([#1619](https://github.com/bluealloy/revm/pull/1619))
- allow non-static lifetime in HandleRegisterBox ([#1608](https://github.com/bluealloy/revm/pull/1608))
- *(EOF)* Use cfg code size limit for eofcreate ([#1606](https://github.com/bluealloy/revm/pull/1606))

### Other
- bump alloy deps ([#1623](https://github.com/bluealloy/revm/pull/1623))
- *(deps)* bump alloy-sol-types from 0.7.6 to 0.7.7 ([#1614](https://github.com/bluealloy/revm/pull/1614))
- group optimism invalid txn errors ([#1604](https://github.com/bluealloy/revm/pull/1604))
- load_account -> warm_preloaded_addresses ([#1584](https://github.com/bluealloy/revm/pull/1584))
- Refactor code, and check if precompile for create collision ([#1600](https://github.com/bluealloy/revm/pull/1600))
- *(revm)* defer bytecode load ([#1588](https://github.com/bluealloy/revm/pull/1588))
- Rename gas_price to gas_limit for precompile args ([#1593](https://github.com/bluealloy/revm/pull/1593))

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-v10.0.0...revm-v11.0.0) - 2024-07-08

### Added
- add bytecode_address from CallInputs to Contract during construction. ([#1568](https://github.com/bluealloy/revm/pull/1568))
- *(Prague)* Add EIP-7702 ([#1565](https://github.com/bluealloy/revm/pull/1565))
- *(EOF)* disallow ExtDelegateCall to legacy bytecode ([#1572](https://github.com/bluealloy/revm/pull/1572))
- *(EOF)* Add target address expansion checks ([#1570](https://github.com/bluealloy/revm/pull/1570))

### Other
- bump precompile to v9.0.0 ([#1590](https://github.com/bluealloy/revm/pull/1590))
- *(README)* add rbuilder to used-by ([#1585](https://github.com/bluealloy/revm/pull/1585))
- Use HandleOrRuntime to allow alloydb/ethersdb to hold a custom runtime ([#1576](https://github.com/bluealloy/revm/pull/1576))
- store tokio::runtime::Handle in ethers/alloyDB ([#1557](https://github.com/bluealloy/revm/pull/1557))
- use const blocks ([#1522](https://github.com/bluealloy/revm/pull/1522))
- fix compile for alloydb ([#1559](https://github.com/bluealloy/revm/pull/1559))
- replace AccessList with alloy version ([#1552](https://github.com/bluealloy/revm/pull/1552))
- replace U256 with u64 in BLOCKHASH ([#1505](https://github.com/bluealloy/revm/pull/1505))

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-v9.0.0...revm-v10.0.0) - 2024-06-20

### Added
- *(revm)* derive serde for `BundleState` ([#1539](https://github.com/bluealloy/revm/pull/1539))
- bump alloy, re-enable alloydb ([#1533](https://github.com/bluealloy/revm/pull/1533))
- mutable access for all fields in BundleBuilder ([#1524](https://github.com/bluealloy/revm/pull/1524))
- *(EOF)* Put EOF bytecode behind an Arc ([#1517](https://github.com/bluealloy/revm/pull/1517))
- *(EOF)* EXTCODECOPY,EXTCODESIZE,EXTCODEHASH eof support ([#1504](https://github.com/bluealloy/revm/pull/1504))
- add helpers for working with instruction tables ([#1493](https://github.com/bluealloy/revm/pull/1493))
- *(precompiles)* fatal error for precompiles ([#1499](https://github.com/bluealloy/revm/pull/1499))
- Persist reverted account and storage slot lookups in `JournaledState` ([#1437](https://github.com/bluealloy/revm/pull/1437))
- *(EOF)* EIP-7698 eof creation transaction ([#1467](https://github.com/bluealloy/revm/pull/1467))
- *(EOF)* Add EOF to inspector handle register ([#1469](https://github.com/bluealloy/revm/pull/1469))
- *(optimism)* Implement new L1 cost function for Fjord ([#1420](https://github.com/bluealloy/revm/pull/1420))
- *(optimism)* Add secp256r1 precompile for Fjord ([#1436](https://github.com/bluealloy/revm/pull/1436))
- *(revm)* revert EIP-2935 BLOCKHASH opcode changes ([#1450](https://github.com/bluealloy/revm/pull/1450))
- load account should return db error ([#1447](https://github.com/bluealloy/revm/pull/1447))
- *(EOF)* remove TXCREATE ([#1415](https://github.com/bluealloy/revm/pull/1415))

### Fixed
- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- *(eof)* output gas for eofcreate ([#1540](https://github.com/bluealloy/revm/pull/1540))
- *(revm)* remove storage reset that clears is_cold flag ([#1518](https://github.com/bluealloy/revm/pull/1518))
- *(op)* Remove `U256::from(<float>)` ([#1498](https://github.com/bluealloy/revm/pull/1498))
- *(EOF)* panic on empty input range, and continue exec after eofcreate ([#1477](https://github.com/bluealloy/revm/pull/1477))
- *(Interpreter)* wrong block number used ([#1458](https://github.com/bluealloy/revm/pull/1458))
- blockchash for devnet-0  ([#1427](https://github.com/bluealloy/revm/pull/1427))

### Other
- Add CI build target for no-std + optimism, use matrix builds ([#1551](https://github.com/bluealloy/revm/pull/1551))
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- avoid cloning precompiles ([#1486](https://github.com/bluealloy/revm/pull/1486))
- add setters to `BundleBuilder` with `&mut self` ([#1527](https://github.com/bluealloy/revm/pull/1527))
- pluralize EOFCreateInput ([#1523](https://github.com/bluealloy/revm/pull/1523))
- added simular to used-by ([#1521](https://github.com/bluealloy/revm/pull/1521))
- Removed .clone() in ExecutionHandler::call, and reusing output buffer in Interpreter ([#1512](https://github.com/bluealloy/revm/pull/1512))
- remove old deprecated items ([#1489](https://github.com/bluealloy/revm/pull/1489))
- *(deps)* bump rstest from 0.19.0 to 0.21.0 ([#1482](https://github.com/bluealloy/revm/pull/1482))
- *(deps)* bump tokio from 1.37.0 to 1.38.0 ([#1480](https://github.com/bluealloy/revm/pull/1480))
- *(primitives)* rename State/Storage to EvmState/EvmStorage ([#1459](https://github.com/bluealloy/revm/pull/1459))
- remove 'checked' bytecode bench causing benchmarks to crash due to name ([#1461](https://github.com/bluealloy/revm/pull/1461))
- cargo update ([#1451](https://github.com/bluealloy/revm/pull/1451))
- cleanup host blockhash fn ([#1430](https://github.com/bluealloy/revm/pull/1430))
- Revert "Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))" ([#1426](https://github.com/bluealloy/revm/pull/1426))
- Revert "feat: implement EIP-2935 ([#1354](https://github.com/bluealloy/revm/pull/1354))" ([#1424](https://github.com/bluealloy/revm/pull/1424))
- *(deps)* bump anyhow from 1.0.82 to 1.0.83 ([#1404](https://github.com/bluealloy/revm/pull/1404))

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-v8.0.0...revm-v9.0.0) - 2024-05-12

### Added
- *(precompile)* Prague - EIP-2537 - BLS12-381 curve operations ([#1389](https://github.com/bluealloy/revm/pull/1389))
- add a hook to execute individual frames ([#1369](https://github.com/bluealloy/revm/pull/1369))
- *(Handler)* Add ClearHandle ([#1368](https://github.com/bluealloy/revm/pull/1368))
- Add uniswap V2 WETH-USDC swap example ([#1353](https://github.com/bluealloy/revm/pull/1353))
- *(interpreter)* add helpers for spending all gas ([#1360](https://github.com/bluealloy/revm/pull/1360))
- add helper methods to CallInputs ([#1345](https://github.com/bluealloy/revm/pull/1345))
- *(revm)* make `ItemOrResult` serializable ([#1282](https://github.com/bluealloy/revm/pull/1282))
- add flag to force hashbrown usage ([#1284](https://github.com/bluealloy/revm/pull/1284))
- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- *(`db`)* Introduce `alloydb` ([#1257](https://github.com/bluealloy/revm/pull/1257))
- *(interpreter)* remove SPEC generic from gas calculation functions ([#1243](https://github.com/bluealloy/revm/pull/1243))
- *(interpreter)* test Host object-safety, allow `dyn Host` in instructions ([#1245](https://github.com/bluealloy/revm/pull/1245))

### Fixed
- *(eip2935)* Preload blockchash storage address ([#1395](https://github.com/bluealloy/revm/pull/1395))
- return the correct error in resize_memory ([#1359](https://github.com/bluealloy/revm/pull/1359))

### Other
- add Trin to used by list ([#1393](https://github.com/bluealloy/revm/pull/1393))
- refactor lints ([#1386](https://github.com/bluealloy/revm/pull/1386))
- bump alloy & specify dep rev ([#1380](https://github.com/bluealloy/revm/pull/1380))
- *(interpreter)* branch less in as_usize_or_fail ([#1374](https://github.com/bluealloy/revm/pull/1374))
- *(ci)* bump action/deploy ([#1372](https://github.com/bluealloy/revm/pull/1372))
- shrink OpCodeInfo and add more methods ([#1307](https://github.com/bluealloy/revm/pull/1307))
- *(deps)* bump anyhow from 1.0.81 to 1.0.82 ([#1293](https://github.com/bluealloy/revm/pull/1293))
- fix some warnings ([#1305](https://github.com/bluealloy/revm/pull/1305))
- Update documentation ([#1275](https://github.com/bluealloy/revm/pull/1275))
- *(interpreter)* use `pop_top!` where possible ([#1267](https://github.com/bluealloy/revm/pull/1267))
- add and use EvmContext::take_error ([#1264](https://github.com/bluealloy/revm/pull/1264))

## [8.0.0](https://github.com/bluealloy/revm/compare/revm-v7.2.0...revm-v8.0.0) - 2024-04-02

### Added
- [**breaking**] TracerEip3155 optionally traces memory ([#1234](https://github.com/bluealloy/revm/pull/1234))

### Fixed
- *(TracerEip3155)* clear Inspector data after transaction. ([#1230](https://github.com/bluealloy/revm/pull/1230))
- *(GasInspector)* calculate correct remaining gas after call return ([#1236](https://github.com/bluealloy/revm/pull/1236))
- fix eip3155 summary gas_used bug and add fork name ([#1216](https://github.com/bluealloy/revm/pull/1216))

### Other
- use uint macro & fix various small things ([#1253](https://github.com/bluealloy/revm/pull/1253))
- *(deps)* bump tokio from 1.36.0 to 1.37.0 ([#1244](https://github.com/bluealloy/revm/pull/1244))
- *(interpreter)* unbox contract field ([#1228](https://github.com/bluealloy/revm/pull/1228))
- *(primitives)* kzg intro ([#1209](https://github.com/bluealloy/revm/pull/1209))
- *(interpreter)* keep track of remaining gas rather than spent ([#1221](https://github.com/bluealloy/revm/pull/1221))
- Improve `EthersDB` ([#1208](https://github.com/bluealloy/revm/pull/1208))

## [7.2.0](https://github.com/bluealloy/revm/compare/revm-v7.1.0...revm-v7.2.0) - 2024-03-19

### Added
- add convert_boxed and insert_boxed for InstructionTable ([#1194](https://github.com/bluealloy/revm/pull/1194))
- optional nonce check ([#1195](https://github.com/bluealloy/revm/pull/1195))

### Other
- Change unwrap to ? to propagate errors ([#1207](https://github.com/bluealloy/revm/pull/1207))
- fix wonky test ([#1197](https://github.com/bluealloy/revm/pull/1197))
- clippy ([#1196](https://github.com/bluealloy/revm/pull/1196))
- *(deps)* bump anyhow from 1.0.80 to 1.0.81 ([#1187](https://github.com/bluealloy/revm/pull/1187))
- fix some typos ([#1189](https://github.com/bluealloy/revm/pull/1189))
- Fix typo in readme ([#1185](https://github.com/bluealloy/revm/pull/1185))
- Update post_execution.rs ([#1180](https://github.com/bluealloy/revm/pull/1180))

## [7.1.0](https://github.com/bluealloy/revm/compare/revm-v7.0.0...revm-v8.0.0) - 2024-03-08

### Added
- Restrict ContextPrecompiles only to EvmContext ([#1174](https://github.com/bluealloy/revm/pull/1174))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-v6.1.0...revm-v7.0.0) - 2024-03-08

This release got yanked and replaced with 7.1.0

### Added
- add insert method on instruction table ([#1167](https://github.com/bluealloy/revm/pull/1167))
- precompile with generic context ([#1155](https://github.com/bluealloy/revm/pull/1155))
- use `impl` instead of `dyn` in `GetInspector` ([#1157](https://github.com/bluealloy/revm/pull/1157))
- add more JournaledState methods to EvmContext ([#1158](https://github.com/bluealloy/revm/pull/1158))
- add example for using a database by reference ([#1150](https://github.com/bluealloy/revm/pull/1150))
- Add boxed precompile trait ([#1131](https://github.com/bluealloy/revm/pull/1131))
- add with_handler method to EvmBuilder ([#1124](https://github.com/bluealloy/revm/pull/1124))

### Fixed
- *(revme)* revme error output and remove double summary ([#1169](https://github.com/bluealloy/revm/pull/1169))
- gas cost calculation ([#1166](https://github.com/bluealloy/revm/pull/1166))
- reset tstorage on finalize ([#1168](https://github.com/bluealloy/revm/pull/1168))
- make `feature = "optional_gas_refund"` work ([#1134](https://github.com/bluealloy/revm/pull/1134))

### Other
- *(deps)* bump ethers-contract from 2.0.13 to 2.0.14 ([#1161](https://github.com/bluealloy/revm/pull/1161))
- *(interpreter)* evaluate instruction table constructor at compile time ([#1140](https://github.com/bluealloy/revm/pull/1140))
- remove clone for context in handler_register.rs ([#1138](https://github.com/bluealloy/revm/pull/1138))
- Check runtime dynamically ([#1135](https://github.com/bluealloy/revm/pull/1135))
- *(deps)* bump auto_impl from 1.1.2 to 1.2.0 ([#1132](https://github.com/bluealloy/revm/pull/1132))
- Add `db` and `db_mut` to evm ([#1133](https://github.com/bluealloy/revm/pull/1133))
- add ToString for no_std add exports some types in no_std ([#1128](https://github.com/bluealloy/revm/pull/1128))
- Add `clone` method to `ContextWithHandlerCfg` ([#1127](https://github.com/bluealloy/revm/pull/1127))
- remove unused EvmInstructionTables type alias ([#1123](https://github.com/bluealloy/revm/pull/1123))

## [6.1.0](https://github.com/bluealloy/revm/compare/revm-v6.0.0...revm-v6.1.0) - 2024-02-22

### Added
- bump c-kzg, add portable feature, make it default ([#1106](https://github.com/bluealloy/revm/pull/1106))
- split off serde_json dependency to its own feature ([#1104](https://github.com/bluealloy/revm/pull/1104))

### Fixed
- replace tuple in sstore return with struct ([#1115](https://github.com/bluealloy/revm/pull/1115))
- fix EthersDB deadlock ([#1089](https://github.com/bluealloy/revm/pull/1089))
- Handle fatal db error on load_account ([#1111](https://github.com/bluealloy/revm/pull/1111))

### Other
- Ensure `L1Block` is in the cache ([#1121](https://github.com/bluealloy/revm/pull/1121))
- Fix feature name for generate_block_traces example ([#1120](https://github.com/bluealloy/revm/pull/1120))
- *(refactor)* Propagate fatal error ([#1116](https://github.com/bluealloy/revm/pull/1116))
- Revert "fix EthersDB deadlock ([#1089](https://github.com/bluealloy/revm/pull/1089))" ([#1118](https://github.com/bluealloy/revm/pull/1118))
- Remove DatabaseRef bound on CacheDB ([#1113](https://github.com/bluealloy/revm/pull/1113))
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))
- *(deps)* bump anyhow from 1.0.79 to 1.0.80 ([#1108](https://github.com/bluealloy/revm/pull/1108))
- improve EIP-3155 implementation ([#1105](https://github.com/bluealloy/revm/pull/1105))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-v5.0.0...revm-v6.0.0) - 2024-02-17

### Added
- improve OriginalValuesKnown docs ([#1083](https://github.com/bluealloy/revm/pull/1083))

### Fixed
- rename and pass optimism-default-handler to revm-primitives ([#1098](https://github.com/bluealloy/revm/pull/1098))
- modify cfg spec_id ([#1095](https://github.com/bluealloy/revm/pull/1095)) ([#1096](https://github.com/bluealloy/revm/pull/1096))
- optimism compilation ([#1091](https://github.com/bluealloy/revm/pull/1091))

### Other
- *(state)* avoid cloning full account ([#1097](https://github.com/bluealloy/revm/pull/1097))
- *(precompile)* use `Bytes` in precompile functions ([#1085](https://github.com/bluealloy/revm/pull/1085))
- Add memory offset ([#1032](https://github.com/bluealloy/revm/pull/1032))

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-v4.0.0...revm-v5.0.0) - 2024-02-12

### Fixed
- properly set context env ([#1070](https://github.com/bluealloy/revm/pull/1070))
- typo on internal append_handle_register methods ([#1069](https://github.com/bluealloy/revm/pull/1069))
- *(op)* skip validation on deposit tx ([#1065](https://github.com/bluealloy/revm/pull/1065))

### Other
- *(precompile)* make use of padding utilities, simplify secp256k1 ([#1073](https://github.com/bluealloy/revm/pull/1073))
- *(revm)* Add helpers to Build Revm with Context ([#1068](https://github.com/bluealloy/revm/pull/1068))
- *(revme)* statetests new format and return error ([#1066](https://github.com/bluealloy/revm/pull/1066))

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-v3.5.0...revm-v4.0.0) - 2024-02-07

Refactored the logic inside Handler and added EvmBuilder that allows overwriting the default behavior.
Few major renaming: EVMImpl to Evm, EVM to EvmFactory and EVMData to EvmContext.

### Added
- *(handler)* Change spec id on &mut ([#1055](https://github.com/bluealloy/revm/pull/1055))
- *(Handler)* add push and pop of hanler registers ([#1053](https://github.com/bluealloy/revm/pull/1053))
- tweeks for v4.0 revm release ([#1048](https://github.com/bluealloy/revm/pull/1048))
- *(op)* Ecotone hardfork ([#1009](https://github.com/bluealloy/revm/pull/1009))
- *(inspector)* Share call/create inputs in Inspector call_end/create_end ([#1003](https://github.com/bluealloy/revm/pull/1003))
- Convert optimism panic into graceful error ([#982](https://github.com/bluealloy/revm/pull/982))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- add asm-keccak feature ([#972](https://github.com/bluealloy/revm/pull/972))
- *(ethersdb)* propagate errors instead of panicking in basic_ref ([#935](https://github.com/bluealloy/revm/pull/935))
- *(revm)* implement prepend_state for BundleState ([#907](https://github.com/bluealloy/revm/pull/907))
- add serde derives for `CacheDB` under "serde" flag ([#911](https://github.com/bluealloy/revm/pull/911))
- *(examples)* generate block traces ([#895](https://github.com/bluealloy/revm/pull/895))
- *(revm)* Evm Context Tests and test-utils Feature ([#903](https://github.com/bluealloy/revm/pull/903))
- `Canyon` hardfork behind `optimism` feature flag ([#871](https://github.com/bluealloy/revm/pull/871))
- Loop call stack ([#851](https://github.com/bluealloy/revm/pull/851))
- transition account balance delta ([#843](https://github.com/bluealloy/revm/pull/843))
- *(cfg)* optionally disable beneficiary reward ([#834](https://github.com/bluealloy/revm/pull/834))
- add more `auto_impl`s to revm traits ([#799](https://github.com/bluealloy/revm/pull/799))
- *(interpreter)* add more helper methods to memory ([#794](https://github.com/bluealloy/revm/pull/794))
- derive more traits ([#745](https://github.com/bluealloy/revm/pull/745))
- add methods to `CreateInput` for calculating created address ([#793](https://github.com/bluealloy/revm/pull/793))
- *(revm)* implement DatabaseRef trait for EthersDB ([#774](https://github.com/bluealloy/revm/pull/774))

### Fixed
- fix previous commit ([#1044](https://github.com/bluealloy/revm/pull/1044))
- *(State)* Preserve original values on delete revert ([#1010](https://github.com/bluealloy/revm/pull/1010))
- optimism gas refunds ([#989](https://github.com/bluealloy/revm/pull/989))
- dont calculate initcode keccak on CREATE ([#969](https://github.com/bluealloy/revm/pull/969))
- *(ci)* Workflow Touchups ([#901](https://github.com/bluealloy/revm/pull/901))
- safer stack ([#879](https://github.com/bluealloy/revm/pull/879))
- *(op)* Base Goerli `op-reth` sync patches ([#824](https://github.com/bluealloy/revm/pull/824))
- fix typos in revm crate ([#821](https://github.com/bluealloy/revm/pull/821))
- Optimism execution ([#789](https://github.com/bluealloy/revm/pull/789))
- rename `DatabaseRef` trait functions to `*_ref` ([#795](https://github.com/bluealloy/revm/pull/795))

### Other
- bump c-kzg and enable blst portable feature ([#1059](https://github.com/bluealloy/revm/pull/1059))
- spelling on last commit ([#1058](https://github.com/bluealloy/revm/pull/1058))
- helper functions around Env ([#1057](https://github.com/bluealloy/revm/pull/1057))
- *(deps)* bump tokio from 1.35.1 to 1.36.0 ([#1052](https://github.com/bluealloy/revm/pull/1052))
- *(EvmBuilder)* rename builder functions to HandlerCfg ([#1050](https://github.com/bluealloy/revm/pull/1050))
- *(deps)* bump ethers-contract from 2.0.11 to 2.0.13 ([#1034](https://github.com/bluealloy/revm/pull/1034))
- *(std)* Add std HashMap,HashSet ([#1041](https://github.com/bluealloy/revm/pull/1041))
- group handlers ([#1030](https://github.com/bluealloy/revm/pull/1030))
- *(Inspector)* add inspector depth test ([#1028](https://github.com/bluealloy/revm/pull/1028))
- *(op)* Move op l1 block load to op handler ([#1026](https://github.com/bluealloy/revm/pull/1026))
- *(clippy)* nightly clippy ([#1025](https://github.com/bluealloy/revm/pull/1025))
- *(Execution)* Granular handles create/call,call_return,insert_call_outcome ([#1024](https://github.com/bluealloy/revm/pull/1024))
- *(Inspector)* Add return_memory_offset to Inspector::call ([#1006](https://github.com/bluealloy/revm/pull/1006))
- update call end docs ([#1000](https://github.com/bluealloy/revm/pull/1000))
- add getter for specId ([#998](https://github.com/bluealloy/revm/pull/998))
- Remove preserve_order in serde_json ([#997](https://github.com/bluealloy/revm/pull/997))
- update create docs ([#999](https://github.com/bluealloy/revm/pull/999))
- *(revme)* EmptyDb Blockhash string, json-outcome flag, set prevrandao in statetest ([#994](https://github.com/bluealloy/revm/pull/994))
- *(Inspector)* add CallOutcome to call/call_end ([#985](https://github.com/bluealloy/revm/pull/985))
- set deduct_caller in optimism handler ([#988](https://github.com/bluealloy/revm/pull/988))
- fix serde std flags for no-std build ([#987](https://github.com/bluealloy/revm/pull/987))
- *(Inspector)* Add CreateOutcome in create/create_end return ([#980](https://github.com/bluealloy/revm/pull/980))
- *(log)* use alloy_primitives::Log ([#975](https://github.com/bluealloy/revm/pull/975))
- *(EvmBuilder)* Remove unnecessary BuilderStage trait ([#979](https://github.com/bluealloy/revm/pull/979))
- enhance readability ([#968](https://github.com/bluealloy/revm/pull/968))
- *(interpreter)* refactor sstore_cost ([#974](https://github.com/bluealloy/revm/pull/974))
- *(interpreter)* improve enum naming ([#962](https://github.com/bluealloy/revm/pull/962))
- *(deps)* bump anyhow from 1.0.77 to 1.0.79 ([#950](https://github.com/bluealloy/revm/pull/950))
- relax Bytes requirement and use slice instead ([#937](https://github.com/bluealloy/revm/pull/937))
- *(deps)* bump futures from 0.3.29 to 0.3.30 ([#927](https://github.com/bluealloy/revm/pull/927))
- *(deps)* bump anyhow from 1.0.75 to 1.0.76 ([#921](https://github.com/bluealloy/revm/pull/921))
- *(deps)* bump tokio from 1.34.0 to 1.35.0 ([#909](https://github.com/bluealloy/revm/pull/909))
- *(revm)* leverage StorageSlot methods, where appropriate ([#899](https://github.com/bluealloy/revm/pull/899))
- relax state generic ([#881](https://github.com/bluealloy/revm/pull/881))
- clippy ([#877](https://github.com/bluealloy/revm/pull/877))
- *(deps)* bump ethers-contract from 2.0.10 to 2.0.11 ([#867](https://github.com/bluealloy/revm/pull/867))
- bump k256 and use normalize_s ([#870](https://github.com/bluealloy/revm/pull/870))
- simplify use statements ([#864](https://github.com/bluealloy/revm/pull/864))
- Fix error message for LackOfFundForMaxFee ([#858](https://github.com/bluealloy/revm/pull/858))
- Fix rustdoc warnings ([#859](https://github.com/bluealloy/revm/pull/859))
- *(deps)* bump tokio from 1.33.0 to 1.34.0 ([#856](https://github.com/bluealloy/revm/pull/856))
- change addresses to iterator and add into_addresses ([#855](https://github.com/bluealloy/revm/pull/855))
- use keccak256 for blockhash ([#854](https://github.com/bluealloy/revm/pull/854))
- review safety comments ([#811](https://github.com/bluealloy/revm/pull/811))
- *(deps)* bump futures from 0.3.28 to 0.3.29 ([#839](https://github.com/bluealloy/revm/pull/839))
- *(state)* consistent selfdestruct status transition ([#847](https://github.com/bluealloy/revm/pull/847))
- *(state)* move account status transitions to `AccountStatus` ([#844](https://github.com/bluealloy/revm/pull/844))
- *(state)* simplify control flow in `CacheState::apply_evm_state` ([#842](https://github.com/bluealloy/revm/pull/842))
- Refactor precompile list from Hash to vec ([#823](https://github.com/bluealloy/revm/pull/823))
- *(state)* make `State::apply_transition` pub ([#832](https://github.com/bluealloy/revm/pull/832))
- *(state)* make bundle state non-optional ([#828](https://github.com/bluealloy/revm/pull/828))
- Refactor evm data to its file ([#815](https://github.com/bluealloy/revm/pull/815))
- for now support 1.69 rust compiler ([#814](https://github.com/bluealloy/revm/pull/814))
- refactor main return to handle ([#808](https://github.com/bluealloy/revm/pull/808))
- *(SharedMemory)* small refactor; tests ([#806](https://github.com/bluealloy/revm/pull/806))
- use `array::from_fn` in `make_instruction_table` ([#809](https://github.com/bluealloy/revm/pull/809))
- remove `step` and `step_end` return result ([#804](https://github.com/bluealloy/revm/pull/804))
- Instruction table ([#759](https://github.com/bluealloy/revm/pull/759))
- getter for  field of ([#792](https://github.com/bluealloy/revm/pull/792))
- Shared memory between calls ([#673](https://github.com/bluealloy/revm/pull/673))
- Fix typos ([#790](https://github.com/bluealloy/revm/pull/790))
- *(deps)* bump tokio from 1.32.0 to 1.33.0 ([#785](https://github.com/bluealloy/revm/pull/785))
- Use upstream create and create2 implementations ([#775](https://github.com/bluealloy/revm/pull/775))
- reorder JournalState impl ([#772](https://github.com/bluealloy/revm/pull/772))
- document everything, dedup existing docs ([#741](https://github.com/bluealloy/revm/pull/741))

# v3.5.0
date 02.10.2023

Migration to alloy primitive types.

Full git log:
* 4e78fbe - docs: document non-zero amounts in State::increment_balances (#760) (15 hours ago) <Dan Cline>
* af4146a - feat: Alloy primitives (#724) (15 hours ago) <evalir>

# v3.4.0
date: 28.09.2023

Summary:
* Cancun ready. all EIP implemented.
  Check interpreter CHANGELOG
* revm State. a `Database` that handles Reverts and state transitions.
* Optimism support
* no_std build

Note: c-kzg can't be build for wasm and is behind "c-kzg" feature flag.

Full git log:
* ea0d8d8 - fix: use u128 for calc data fee result (#757) (46 minutes ago) <Dan Cline>
* 4f916be - chore: bump c-kzg to create lib (#758) (5 hours ago) <rakita>
* ded673c - docs: Readme Updates (#756) (16 hours ago) <refcell.eth>
* f79d0e1 - feat: Optimism execution changes (#682) (16 hours ago) <clabby>
* d2a066b - ci: concurrency for github actions (#750) (25 hours ago) <Paul Razvan Berg>
* d03dfcb - Improve wording and fix typos (#749) (25 hours ago) <Paul Razvan Berg>
* 2c556c0 - refactor: say "warm" instead of "hot" (#754) (25 hours ago) <Paul Razvan Berg>
* 8a85d19 - fix: balance check disabled (#751) (25 hours ago) <Wodann>
* b9938a8 - chore(deps): bump sha2 from 0.10.7 to 0.10.8 (#752) (30 hours ago) <dependabot[bot]>
* 4829e6a - chore(deps): bump thiserror from 1.0.48 to 1.0.49 (#753) (30 hours ago) <dependabot[bot]>
* 8206193 - feat: add "kzg" as a separate feature (#746) (3 hours ago) <DaniPopes>
* 4b5fa61 - EIP-6780: SELFDESTRUCT only in same transaction (#719) (5 days ago) <Lorenzo Feroleto>
* f72eaa0 - chore: error type for block header (#731) (5 days ago) <hack3r-0m>
* cb39117 - fix(eip4844): Pass eth tests, additional conditions added. (#735) (6 days ago) <rakita>
* c2cde03 - fix: use CANCUN precompile id for CANCUN SpecId (#733) (6 days ago) <Dan Cline>
* d926728 - perf: refactor interpreter internals and cleanup (#582) (6 days ago) <DaniPopes>
* 1b8cd57 - make BundleBuilder publicly available (#729) (8 days ago) <Thomas Coratger>
* fa13fea - feat: implement EIP-4844 (#668) (11 days ago) <DaniPopes>
* 9f00e37 - feat(state): remove state sorting, no_std ci,remove rayon (#717) (13 days ago) <rakita>
* 429da73 - chore(perf): only recalc code hash if its the default (#716) (13 days ago) <evalir>
* e2ecd5e - docs: add warning on panic conditions in take_bundle (#715) (2 weeks ago) <Dan Cline>
* 190f90e - Never inline the prepare functions (#712) (2 weeks ago) <Valentin Mihov>
* 26dc07d - feat: return wiped inside storage changeset (#711) (2 weeks ago) <rakita>
* 5d68dd5 - chore(deps): bump bytes from 1.4.0 to 1.5.0 (#707) (2 weeks ago) <dependabot[bot]>
* fd8d4c5 - chore(deps): bump ethers-contract from 2.0.9 to 2.0.10 (#705) (2 weeks ago) <dependabot[bot]>
* e86c19b - chore(state): do not insert empty reverts in state (#702) (3 weeks ago) <Lorenzo Feroleto>
* 7eacc3a - chore: implement `Default` for other databases (#691) (3 weeks ago) <DaniPopes>
* 1d6a039 - chore: make `impl Default for StateBuilder` generic (#690) (3 weeks ago) <DaniPopes>
* c60abcf - feat(state): Nits, builder option and OriginalValueKnown flags (#699) (3 weeks ago) <rakita>
* 7e7a339 - bundle size hint (#670) (3 weeks ago) <Roman Krasiuk>
* f6c9c7f - chore: deprecate `RefDBWrapper` (#696) (3 weeks ago) <DaniPopes>
* d04aad3 - chore: expose StateDBBox (#694) (3 weeks ago) <rakita>
* ee13aac - feat(StateBuilder): switch builder option from without_bundle to with_bundle (#688) (3 weeks ago) <rakita>
* 7d7f63f - chore(state): Make Database more generic. (#687) (3 weeks ago) <rakita>
* a9dce30 - chore: nits and renamings (#684) (3 weeks ago) <rakita>
* b500718 - feat(state): take N reverts from BundleState, struct refactor (#681) (3 weeks ago) <rakita>
* fde6df1 - apply builder pattern for BundleState initialization (#649) (3 weeks ago) <Eric>
* 2897655 - fix(state): Extend now properly transfers wiped storage (#675) (3 weeks ago) <rakita>
* 6bd05c9 - chore: impl Eq, PartialEq for TransitionState (#677) (4 weeks ago) <Dan Cline>
* 175aaec - Removed the last dependencies breaking no-std build. (#669) (4 weeks ago) <Lucas Clemente Vella>
* 4272535 - fix(state): retain destroyed account status on bundle extend (#667) (4 weeks ago) <rakita>
* bef9edd - chore(state): bundle retention (#666) (4 weeks ago) <Roman Krasiuk>
* 1053d0e - fix(state): Regresion, remove present info on selfdestruct (#664) (4 weeks ago) <rakita>
* 6c4cd31 - feat: add BundleState::revert_latest (#661) (4 weeks ago) <Matthias Seitz>
* fd2cc88 - fix(state): state transition regression (#662) (4 weeks ago) <Roman Krasiuk>
* c14f8a9 - feat(state): add a flag allowing transition merge without reverts (#657) (4 weeks ago) <Roman Krasiuk>
* 33498d7 - chore(deps): bump ethers-contract from 2.0.8 to 2.0.9 (#640) (4 weeks ago) <dependabot[bot]>
* 9a88c99 - chore: filter out empty bytecode from bundle (#656) (4 weeks ago) <rakita>
* 98a4a18 - feat(state): Make Bundle extend wipe aware (#655) (4 weeks ago) <rakita>
* 1bf0315 - feat(state): ability to disable reverts collection in bundle state (#654) (4 weeks ago) <Roman Krasiuk>
* 3eea324 - fix(state): drop storage only for DestroyedChanged (#651) (4 weeks ago) <rakita>
* 37027db - fix revert from DestroyedChanged to DestroyedAgain (#648) (5 weeks ago) <rakita>
* cec7f82 - chore(state): bundle state split (#646) (5 weeks ago) <Roman Krasiuk>
* ff5a2bc - add value parameter to Inspector::selfdestruct (#645) (5 weeks ago) <Tony Ke>
* b2d6f7a - Refactor: Split transaction pre verification to separate function (#573) (5 weeks ago) <Lorenzo Feroleto>
* afbc896 - fix(state): check if storage revert is empty (#643) (5 weeks ago) <Roman Krasiuk>
* 0b9c12e - test(state): bundle selfdestructs (#627) (5 weeks ago) <Roman Krasiuk>
* 6b55b9c - feat(`interpreter`): add hash to bytecode (#628) (5 weeks ago) <evalir>
* 2054293 - chore: misc improvements (#633) (5 weeks ago) <DaniPopes>
* 43d535c - style: bundle state (#637) (5 weeks ago) <Roman Krasiuk>
* f843592 - fix(state): return RevertToSlot struct with more info (#636) (5 weeks ago) <rakita>
* aee1d1c - bug(state): remove redundunt info revert on destruct (#635) (5 weeks ago) <rakita>
* 321152a - book workflow (#537) (5 weeks ago) <Waylon Jepsen>
* 0028193 - feat: Optional coinbase tip (#625) (5 weeks ago) <clabby>
* 6ea1edc - test(state): bundle reverts collapse (#626) (5 weeks ago) <Roman Krasiuk>
* a40f272 - feat(state): Use preloaded bundle inside state (#622) (5 weeks ago) <rakita>
* 68820da - feat(state): Block hash cache and overrides (#621) (5 weeks ago) <rakita>
* eb6a9f0 - Revert "feat: alloy migration (#535)" (#616) (6 weeks ago) <rakita>
* e5227c4 - test(state): account & storage revert value preservation (#614) (6 weeks ago) <Roman Krasiuk>
* c1bad0d - chore: spell check (#615) (6 weeks ago) <Roman Krasiuk>
* 588503a - chore: get or insert bundle state (#613) (6 weeks ago) <Roman Krasiuk>
* 7e83c7f - fix(inspector): call call_end/create_end when inspector shortcircuits calls (#609) (6 weeks ago) <evalir>
* adf42b2 - chore(deps): bump anyhow from 1.0.74 to 1.0.75 (#606) (6 weeks ago) <dependabot[bot]>
* 0e85fdf - chore(deps): bump tokio from 1.31.0 to 1.32.0 (#607) (6 weeks ago) <dependabot[bot]>
* 449d6b9 - chore: export some `unreachable_pub` items (#598) (6 weeks ago) <DaniPopes>
* 5d0b54d - chore(deps): bump anyhow from 1.0.72 to 1.0.74 (#602) (6 weeks ago) <dependabot[bot]>
* c785115 - fix: Load caller in safe way in finalization fn (#604) (6 weeks ago) <rakita>
* dfae7fe - chore: fix test build, use new types (#605) (6 weeks ago) <rakita>
* fc2107c - chore: Revert test, not change storage check , renaming of original slot value (#601) (6 weeks ago) <rakita>
* f95b7a4 - feat: alloy migration (#535) (6 weeks ago) <DaniPopes>
* 49a6470 - chore: `TransitionState::with_capacity` -> `TransitionState::single` (#600) (6 weeks ago) <Roman Krasiuk>
* f4224d8 - perf: pre-allocate inner bundle state (#599) (6 weeks ago) <Roman Krasiuk>
* 5cdaa97 - chore: avoid unnecessary allocations (#581) (6 weeks ago) <DaniPopes>
* da26d0d - chore(deps): bump tokio from 1.29.1 to 1.31.0 (#595) (6 weeks ago) <dependabot[bot]>
* ef57a46 - feat: State with account status (#499) (7 weeks ago) <rakita>
* 1478724 - chore: move precompiles to EVMData for inspectors (#588) (7 weeks ago) <evalir>
* fe6c54e - fix(transient_storage): set previous value in journal (#585) (7 weeks ago) <rakita>
* bd84a07 - refactor: rewrite revm-test as a criterion bench (#579) (7 weeks ago) <DaniPopes>
* 5734f12 - fix: AccessList with two same addresses (#578) (8 weeks ago) <rakita>
* 06b1f6b - feat: EIP-1153 Transient storage opcodes (#546) (8 weeks ago) <Mark Tyneway>
* 4686cb3 - fix(revm): EIP-3155 tracer tx output without debug artefact (#552) (9 weeks ago) <Perama>
* 26126ad - fix(revm): extra return in EIP3155 inspector (#563) (9 weeks ago) <Perama>
* 3f6052c - fix(revm): include CREATE/CREATE2 in EIP3155 inspector (#562) (9 weeks ago) <Perama>
* 5ce9dc9 - chore: clippy and fmt (#568) (9 weeks ago) <rakita>
* 30bfa73 - fix(doc): Inline documentation of re-exports (#560) (9 weeks ago) <Yiannis Marangos>
* 10f81ba - optimize stack usage for recursive `call` and `create` programs (#522) (3 months ago) <Valentin Mihov>
* 323370a - fix comment (#529) (3 months ago) <Ethan-000>
* 51072e6 - consume all gas on invalid opcode (#500) (3 months ago) <teddav>
* 63f9460 - chore(deps): bump auto_impl from 1.0.1 to 1.1.0 (#478) (3 months ago) <dependabot[bot]>
* 3a77ee5 - docs: fix comment typo (#517) (3 months ago) <Sabnock>
* d343858 - fix: typo in eip-3155 output (#497) (4 months ago) <Perama>
* f8ff6b3 - feat: separate initial checks (#486) (5 months ago) <rakita>
* c3b0312 - docs: add some CacheDB docs (#484) (5 months ago) <Matthias Seitz>
* c81acc6 - feat: Create account checkpoint (#483) (5 months ago) <rakita>
* 6057cc2 - chore: refactor interpreter run and remove static flag (#481) (5 months ago) <rakita>
* d193418 - chore: Bundle inspector crate/call calls (#480) (5 months ago) <rakita>
* 75a6136 - feat: Introduce account status as bitflag inside JournalState (#477) (5 months ago) <rakita>


# v3.3.0
date: 03.05.2023

Consensus bug:
* cde2f2d - fix: revert of selfdestruct with same target address (#475) (2 hours ago) <Roman Krasiuk>

Other small changes:
* bd0fad8 - (HEAD -> reles, origin/main, origin/HEAD) chore(deps): bump tokio from 1.27.0 to 1.28.0 (#470) (52 minutes ago) <dependabot[bot]>
* ccefbca - chore(deps): bump ruint from 1.7.0 to 1.8.0 (#465) (52 minutes ago) <dependabot[bot]>
* 7c2e0f5 - chore(deps): bump anyhow from 1.0.70 to 1.0.71 (#471) (53 minutes ago) <dependabot[bot]>
* d7adfd5 - Fix typo in primitives/src/state.rs (#474) (53 minutes ago) <Udoagwa Franklin>
* d0cd897 - add example to revm crate (#468) (8 days ago) <Sambhav>
* 08091e1 - fix: compile errors for features (#467) (13 days ago) <rakita>

# v3.2.0
date: 19.04.2023

consensus bug:
* fix: touched account on creation (#463) (2 hours ago) <Roman Krasiuk>

# v3.1.1
date: 14.04.2023

bump revm dependency versions.

# v3.1.0
date: 04.04.2022

Main changes can be summarizes in:
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
    * removal of gas block allowed us to have more compact analysis data. Gas block from beginning didn't have big impact on performance but introduced not intuitive gas calculations that was
    source of some bugs. 
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
    * revm is Shanghai ready
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
    * There wasone  bug inside CacheDB that was here for a long time, and would happen only if
    selfdestruct/create2 is called in multiple transaction on same account on same cache data.
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>


Changelogs:
* 9edb8f4 - (origin/main, origin/HEAD) Improve EthersDB::new (#440) (5 days ago) <lazymio>
* c2ee8ff - add feature for ignoring base fee check (#436) (6 days ago) <Dan Cline>
* 6b09caf - chore(deps): bump serde_json from 1.0.94 to 1.0.95 (#434) (6 days ago) <dependabot[bot]>
* 77f1735 - chore(deps): bump walkdir from 2.3.2 to 2.3.3 (#426) (8 days ago) <dependabot[bot]>
* ed981c3 - chore(deps): bump serde from 1.0.157 to 1.0.158 (#425) (8 days ago) <dependabot[bot]>
* 0eff6a7 - Fix panic! message (#431) (2 weeks ago) <David Kulman>
* 2d5b710 - Comment Fix (#430) (2 weeks ago) <David Kulman>
* d0038e3 - chore(deps): bump arbitrary from 1.2.3 to 1.3.0 (#428) (2 weeks ago) <dependabot[bot]>
* d935525 - chore(deps): bump secp256k1 from 0.26.0 to 0.27.0 (#429) (2 weeks ago) <dependabot[bot]>
* a85ff79 - Update README.md (#424) (2 weeks ago) <Waylon Jepsen>
* 9645015 - chore(deps): bump thiserror from 1.0.38 to 1.0.40 (#421) (2 weeks ago) <dependabot[bot]>
* aa6519f - chore(deps): bump enumn from 0.1.6 to 0.1.8 (#422) (2 weeks ago) <dependabot[bot]>
* d63146f - chore(deps): bump futures from 0.3.26 to 0.3.27 (#416) (2 weeks ago) <dependabot[bot]>
* 52fe7c4 - chore(deps): bump serde_json from 1.0.93 to 1.0.94 (#401) (2 weeks ago) <dependabot[bot]>
* b98d9c9 - chore(deps): bump serde from 1.0.152 to 1.0.157 (#423) (2 weeks ago) <dependabot[bot]>
* 3d8ca66 - feat: add Output::into_data (#420) (3 weeks ago) <Matthias Seitz>
* afc3066 - fix(db): preserve existing account state (#414) (4 weeks ago) <Roman Krasiuk>
* dd0e227 - feat: Add all internals results to Halt (#413) (4 weeks ago) <rakita>
* d8dc652 - fix(interpreter): halt on CreateInitcodeSizeLimit (#412) (4 weeks ago) <Roman Krasiuk>
* b1208fe - feat: add contract+target to selfdestruct hook (#410) (4 weeks ago) <Matthias Seitz>
* a193d79 - chore: enabled primtive default feature in precompile (#409) (4 weeks ago) <Matthias Seitz>
* f2656b7 - chore: add primitive SpecId to precompile SpecId conversion (#408) (4 weeks ago) <Matthias Seitz>
* 1720729 - chore: add display impl for Opcode (#406) (4 weeks ago) <Matthias Seitz>
* 33bf8a8 - feat: use singular bytes for the jumpmap (#402) (4 weeks ago) <Bjerg>
* 394e8e9 - feat: extend SuccessOrHalt (#405) (4 weeks ago) <Matthias Seitz>
* cff1070 - Update readmdoc of `perf_analyse_created_bytecodes` (#404) (4 weeks ago) <rakita>
* fbc62a3 - chore: fix typo StorageChange (#403) (4 weeks ago) <Matthias Seitz>
* 7bb73da - feat: Add check for chainID (#393) (4 weeks ago) <chirag-bgh>
* 3a17ca8 - feat: add b256<->u256 from impls (#398) (4 weeks ago) <Matthias Seitz>
* 3789509 - feat: add API to retrieve unpadded bytecode (#397) (5 weeks ago) <Wodann>
* 5ab154a - chore(deps): bump tokio from 1.25.0 to 1.26.0 (#395) (5 weeks ago) <dependabot[bot]>
* f91d5f9 - refactor: remove gas blocks (#391) (5 weeks ago) <Bjerg>
* 8dc024a - Add copyright start year (#387) (5 weeks ago) <Marius Kjærstad>
* 4d2f074 - feat: add EVM::with_env (#385) (6 weeks ago) <Matthias Seitz>
* 5efd9d1 - impl NonceTooHigh/ NonceTooLow checks (#383) (6 weeks ago) <gd>
* 8e6f4f2 - chore: fix compilation if serde not enabled (#381) (7 weeks ago) <rakita>
* 92f08be - feat: json opcode traces EIP-3155 (#356) (7 weeks ago) <pistomat>
* ec582a8 - chore(deps): bump once_cell from 1.17.0 to 1.17.1 (#378) (7 weeks ago) <dependabot[bot]>
* 188dacf - improvement: derive Debug for DatabaseComponentError (#377) (7 weeks ago) <Wodann>
* 0401cfd - Add B160/B256 From primitive_types traits (#380) (7 weeks ago) <Francesco Cinà>
* a8ae3f4 - fix: using pop_top instead of pop in eval_exp (#379) (7 weeks ago) <flyq>
* 08ce847 - feat(Shanghai): All EIPs: push0, warm coinbase, limit/measure initcode (#376) (7 weeks ago) <rakita>
* 6710511 - add no_std to primitives (#366) (7 weeks ago) <rakita>
* d5ebdb0 - chore(deps): bump tokio from 1.24.2 to 1.25.0 (#352) (7 weeks ago) <dependabot[bot]>
* ebaccca - chore(deps): bump futures from 0.3.25 to 0.3.26 (#353) (7 weeks ago) <dependabot[bot]>
* 5788340 - chore(deps): bump bytes from 1.3.0 to 1.4.0 (#355) (7 weeks ago) <dependabot[bot]>
* d3fba88 - chore(deps): bump serde_json from 1.0.92 to 1.0.93 (#365) (7 weeks ago) <dependabot[bot]>
* e22c3f3 - fix: call create_end for all code paths (#362) (7 weeks ago) <Wodann>
* b4c62e9 - chore: rename Then to Than (#368) (7 weeks ago) <Matthias Seitz>
* 1c3e9e3 - improvement: use alloc & core for Arc impl (#367) (8 weeks ago) <Wodann>
* 3158ce9 - feat: implement Debug for DatabaseComponentError if supported (#363) (8 weeks ago) <Wodann>
* d9727c2 - improvement: add error details to InvalidTransaction::LackOfFundForGasLimit (#364) (8 weeks ago) <Wodann>
* 6b170b4 - Use gas price in place of effective gas price for initial balance check (#359) (8 weeks ago) <gd>
* 5d6ecd0 - improvement: implement BlockHash for Arc<BlockHashRef> (#361) (8 weeks ago) <Wodann>
* ae9baba - improvement: implement State for Arc<StateRef> (#360) (8 weeks ago) <Wodann>
* 2e4e800 - chore(deps): bump serde_json from 1.0.91 to 1.0.92 (#357) (8 weeks ago) <dependabot[bot]>
* 1fca102 - chore(deps): bump proptest from 1.0.0 to 1.1.0 (#358) (8 weeks ago) <dependabot[bot]>
* 9b663bb - feat: Different OutOfGas Error types (#354) (9 weeks ago) <Chirag Baghasingh>
* 10187ed - data change (9 weeks ago) <rakita>

# v3.0.0
date 29.01.2022

This is big release that has core changes that breaks compatibility. In summary:
*  Project is refactored into `revm-primitives`,`revm-precompile`,`revm-interpreter` and `revm` to have more flexibility and separation of concerns. And include paths in revm reflect that. So try to find include as `revm::primitives` or `revm::interpreter`
* Parity `primitive-types` was replaced with `ruint` for big numbers and subset of macros are used for native `B160`/`B256` types. 
* Interpreter instructions are unified and now all of them have same signature.
* web3 db was replaces with ethers alternative.
* revmjs lib was removed from crates.
* `revm_precompiles` was renamed to `revm-precompile.`

* Return types are made to have more insight of what have happened inside revm.
* Snailtracer benchmark got around 20% faster.

Github Changelog:
* dc9818f - (HEAD -> o/bump, origin/bump_v20) Bump v20 (13 hours ago) <rakita>
* 75ef0f1 - (origin/main, origin/HEAD) feat: Staticcall internal return (#349) (13 hours ago) <rakita>
* 0194b37 - (t) fix bug introduced in last commit (13 hours ago) <rakita>
* 7b00f32 - Cleanup imports (#348) (14 hours ago) <rakita>
* c14d7ea - fix: enable the examples to run with the current revm (#347) (16 hours ago) <flyq>
* 329fd94 - Wrap all calls to interpreter.gas.erase_cost with checks if USE_GAS is enabled (#346) (2 days ago) <christn>
* 72355f4 - improvement: add logs & return value to revert (#343) (3 days ago) <Wodann>
* 142a1c9 - expose hashbrown::HashMap in primitives (#345) (3 days ago) <Andy Thomson>
* ba393d7 - fix: disable balance check (#342) (4 days ago) <Wodann>
* 876fad1 - refactor: simplify DatabaseComponentError (#339) (6 days ago) <Wodann>
* 81534ad - chore: includes to libs (#338) (7 days ago) <rakita>
* e2f4d32 - Creating revm-primitives, revm better errors and db components  (#334) (10 days ago) <rakita>
* de83db6 - fix: feature flags (#330) (2 weeks ago) <Wodann>
* b60269c - `revm`: mark `with-serde` feature as deprecated (#328) (2 weeks ago) <Enrique Ortiz>
* 63bf475 - make load_account pub (#325) (3 weeks ago) <rakita>
* 0ef0197 - Cleanup, move hot fields toggether in Interpreter (#321) (3 weeks ago) <rakita>
* 81942d6 - enable proptest with arbitrary feature (#323) (3 weeks ago) <joshieDo>
* 2be3798 - feat: revm-interpreter created (#320) (3 weeks ago) <rakita>
* 7e98fef - fix: feature flag compiler errors (#256) (5 weeks ago) <Wodann>
* 488ef8a - Add example for fork + ref_transact impl (#296) (6 weeks ago) <0xDmtri>
* 56e6c22 - feat: allow disabling of balance checks (#297) (6 weeks ago) <Wodann>
* 8661467 - feat: Export CustomPrinter insector from revm (#300) (6 weeks ago) <rakita>
* 222b8e9 - feature: substitute web3db to ethersdb (#293) (6 weeks ago) <0xDmtri>
* fd01083 - feature(revm): Return `bytes` in Create calls (#289) (7 weeks ago) <Nicolas Gotchac>
* 2fb0933 - docs: Correct typo (#282) (7 weeks ago) <Przemyslaw Rzad>
* 90fe01e - feat(interpreter): Unify instruction fn signature (#283) (7 weeks ago) <rakita>
* 54e0333 - bug: Integer overflow while calculating the remaining gas in GasInspector (#287) (8 weeks ago) <rakita>
* acdbaac - native bits (#278) (8 weeks ago) <rakita>
* 69e302b - feat(revm): Add prevrandao field to EnvBlock (#271) (2 months ago) <rakita>
* d1703cd - Export StorageSlot (#265) (3 months ago) <Francesco Cinà>
* 560bb03 - Fix: typos (#263) (3 months ago) <HAPPY>
* 369244e - feat(refactor): make keccak in one place. (#247) (3 months ago) <rakita>
* c96c878 - feat: Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239) (3 months ago) <Alexey Shekhirin>


# v2.3.1
date: 22.11.2022

Bump dependency versions.


# v2.3.0
date: 16.11.2022
Very small release. Exposes one field and added prevrandao to remove footgun of forgeting to set difficulty.

* 927d16c - disable gas refunds with env flag (#267) (14 minutes ago) <gd>
* 47a8310 - Add prevrandao field to EnvBlock (3 minutes ago) <rakita>
* 2c45b04 - Export StorageSlot (#265) (23 minutes ago) <Francesco Cinà>

# v2.2.0
date: 12.11.2022

Small release that contains consensus bug fix. Additionally added few small feature flags needed for hardhat, opcode utility function and removal of web3db block number check. 

* dc3414a - Added OEF spec for tests. Skip HighGasPrice (4 minutes ago) <rakita>
* f462f9d - Bugfix: if returndatacopy is len 0 return after initial cost (#259) (4 minutes ago) <gd>
* ea2f2a2 - fix web3db sanity check (#245) (12 days ago) <Wulder>
* 9f8cdbd - feat: allow block gas limit to be toggled off (#238) (3 weeks ago) <Wodann>
* efd9afc - feat: allow eip3607 to be toggled off (#237) (3 weeks ago) <Wodann>
* 88c72a7 - fix: return out of gas code for precompiled contracts (#234) (3 weeks ago) <Wodann>
* 30462a3 - Fix: typos (#232) (3 weeks ago) <omahs>
* 9f513c1 - Borrow self and add derive traits for OpCode (#231) (4 weeks ago) <Franfran>

# v2.1.0
date: 25.09.2022

GasInspector added by Alexey Shekhirin and some helper functions.
Changes:

* ca14d61 - gas inspector (#222) (7 days ago) <Alexey Shekhirin>
* 1e25c99 - chore: expose original value on storageslot (#216) (13 days ago) <Matthias Seitz>
* aa39d64 - feat: add Memory::shrink_to_fit (#215) (13 days ago) <Matthias Seitz

# v2.0.0
date: 10.09.2022

Release with `Database` interface changed, execution result, consensus bug fixes and support for all past forks. Additional optimizations on evm initialization.

Main changes:
* Add support for old forks. (#191) (9 days ago)
* revm/evm: Return `ExecutionResult`, which includes `gas_refunded` (#169) (4 weeks ago) <Nicolas Gotchac>
* JournaledState (#175)
    * Optimize handling of precompiles. Initialization and account loading.
    * Fixes SELFDESTRUCT bug.
* Optimize calldataload. Some cleanup (#168)
* Handle HighNonce tests (#176)
* feat: expose hash on `BytecodeLocked` (#189) (12 days ago) <Bjerg>
* revm: Update account storage methods in CacheDB (#171) (4 weeks ago) <Nicolas Gotchac>
* reexport revm_precompiles as precompiles (#197) (6 days ago) <Matthias Seitz>
* chore(ci): use ethtests profile for CI tests (#188) (2 weeks ago) <Alexey Shekhirin>
* Bump dependencies version
* current_opcode fn and rename program_counter to instruction_pointer (#211)
* Cfg choose create analysis, option on bytecode size limit (#210)
* Cleanup remove U256 and use u64 for gas calculation (#213)

Consensus bugs:
* SELFDESTRUCT was not handled correctly. It would remove account/storage but it should just mark it for removal. This bug was here from earlier version of revm. (#175)
* fix: set gas_block to empty bytecode (#172). Introduced in v1.8.0 with bytecode format.

# v1.9.0
date: 09.08.2022

Small release. Optimizations

* Cache bytecode hash
* Move override_spec config from Inspector to cfg

# v1.8.0
date: 01.08.2022

Medium release, good performance boost. Database trait has changed to support Bytecode.

* Introduce Bytecode format (#156)
* Update readme files.
* Merge eth/tests supported.

# v1.7.0
date: 11.06.2022

small release:
* Make CacheDB field pub and add few utility functions
* Rename Byzantine to Byzantium

# v1.6.0
date: 02.06.2022

Most changes are relayed to CacheDB and how it saved accounts.

* Introduce account `Touched/Cleared/None` state in CacheDB
* Add missing inspectors `call_end` calls
* bump dependencies and few standard derives.

# v1.5.0
date: 09.06.2022

Consensus error related to gas block optimization and `sstore` min stipend. Solution is to make `sstore` instruction as `gas_block_end` as to not spend future instruction gas when checking min stipend condition introduced in EIP-2200.

* Consensus error with gas block for SSTORE stipend check (#124)
* enable EIP2200 in Istanbul (#125)

# v1.4.1
date: 06.06.2022

Small release:
* chore: export evm_inner (#122)

# v1.4.0
date: 03.06.2022

Small release:
* fix: BLOCKHASH should return 0 if number not in last 256 blocks (#112)
* feat: add getters for cachedb (#119)
* bump some lib versions.

# v1.3.1
date: 11.4.2022

Small fixes release.
* Empty keccak constant and remove access_list.clone (#111)
* chore: typo fixes
* fix is_static for Inspector initialize_interp

# v1.3.0
date: 30.4.2022

There are a lot of big changes that are included in this release as revm was integrated inside foundry.

* A lot of changed on Inspector, added new calls and flushed out how it should be called. Big effort mostly driven by Oliver Nordbjerg
* Big internal refactor and renaming: Machine->Inspector, call/create info are now in structs.
* feat: add serde support to model types. Thank you Matthias Seitz
* Added rust feature that sets memory limit on interpreter that is configurable with env.cfg. by Oliver Nordbjerg.
* Library bumped to higher version.

# v1.2.0
date 20.1.2022

Changes:
* Bump revm_precompile and added new feature for k256 lib.

# v1.1.0
date: 14.1.2022

There is bug introduced in last release with gas blcok optimization, it will crash revm if anywhere in contract is unknown OpCode. And now returning log after execution (ups) included them in eth/tests verification.

Changes:
* Bug fix for unknown OpCode
* Omit edgecase high nonce test. tracer gas fix 
* Some internal cleanup

# v1.0.0
date: 18.12.2021

It feel's like that the lib is in the state that is okay to promote it to the v1 version. Other than that, a lot of optimizations are done and the inspector trait was rewritten.

Changes: 
*  web3 db
*  precalculated gas blocks. Optimization
*  PC opcode as pointer. Optimization
*  U256 div_rem optimization
*  Inspector refactored and it is now closer to Host interface.

Optimization thread: https://github.com/bluealloy/revm/issues/7


# v0.5.0
date: 17.11.2021

A lot of optimization on machine(Interpreter) part, it is now at least 3x faster. On interface side, Error enum was renamed to Return and it is simplified. Additionally if needed gas measuring can be removed with rust feature.

Changes: 
* push instruction optimized.
* mload/mstore and memory optimized
* Gas calculation optimized
* optimize i256
* switch stacks from H256 with U256
* Error's refactor to Return
* clippy/warnings/fmt cleanup
* Bump auto_impl to v0.5
* opcode renaming
* Gas measurment can be removed with rust features.

# v0.4.1
date: 02.11.2021

Change in interface and how you can call evm. There is now multiple Database traits for use and inspector is taken on transact call as reference.

* 20ac70b - Database traits made useful.
* 46b5bcd - EVM Interface changed. Inspector called separately.


# v0.3.1
date: 27.10.2021

remove some warnings for unused imports and done cargo fmt.
# v0.3.0
date: 27.10.2021

Interface revamped and now looks a lot better.

Log:
* 1b1ebd8 - [revm] Interface. Inspector added, Env cleanup. revm-test passes (9 hours ago) <rakita>
* 351d4e0 - BIG interface change (11 hours ago) <rakita>
* a723827 - no_sdt to no_std (2 days ago) <rakita>
* a449bed - [precompiles] spelling, small cleanup (2 days ago) <rakita>


# v0.2.2

Same as v0.2.1 but added readme.
# v0.2.1
date: 25.10.2021

Big refactor, cleanup changes, and updating tests. EIP-3607 added.

Log:
* a6e01de - BIG reorg. workspace added. revm-precompile lib (20 minutes ago) <rakita>
* e50f6d3 - Move merkle trie from revm to eth/tests crate (4 hours ago) <rakita>
* 633ffd4 - Bump tests to v10.1 (28 hours ago) <rakita>
* 14b3de1 - Payment overflow check (30 hours ago) <rakita>
* 6e964ba - EIP-3607: Reject transactions from senders with deployed code (30 hours ago) <rakita>


# v0.2.0
date: 23.10.2021:

Published v0.2.0, first initial version of code. London supported and all eth state test are 100% passing or Istanbul/Berlin/London.


### 17.10.2021:
-For past few weeks working on this structure and project in general become really good and I like it. For me it surved as good distraction for past few weeks and i think i am going to get drained if i continue working on it, so i am taking break and i intend to come back after few months and finish it.
- For status:
    * machine/spec/opcodes/precompiles(without modexp) feels good and I probably dont need to touch them.
    * inspector: is what i wanted, full control on insides of EVM so that we can control it and modify it. will probably needs to add some small tweaks to interface but nothing major.
    * subroutines: Feels okay but it needs more scrutiny just to be sure that all corner cases are covered.
    * Test that are failing (~20) are mostly related to EIP-158: State clearing. For EIP-158 I will time to do it properly.
    * There is probably benefit of replaing HashMap hasher with something simpler, but this is research for another time.
## Project structure: