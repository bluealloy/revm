# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-alpha.2](https://github.com/bluealloy/revm/compare/revm-context-v1.0.0-alpha.1...revm-context-v1.0.0-alpha.2) - 2025-03-10

### Added

- added with_ref_db fn to Context ([#2164](https://github.com/bluealloy/revm/pull/2164))
- remove specification crate ([#2165](https://github.com/bluealloy/revm/pull/2165))
- make journal entries generic ([#2154](https://github.com/bluealloy/revm/pull/2154))
- Standalone Host, remove default fn from context ([#2147](https://github.com/bluealloy/revm/pull/2147))
- add constructor with hardfork ([#2135](https://github.com/bluealloy/revm/pull/2135))
- allow host to be implemented on custom context ([#2112](https://github.com/bluealloy/revm/pull/2112))
- add the debug impl for Evm and EvmData type ([#2126](https://github.com/bluealloy/revm/pull/2126))

### Other

- pre EIP-7702 does not need to load code ([#2162](https://github.com/bluealloy/revm/pull/2162))
- JournalTr, JournalOutput, op only using revm crate ([#2155](https://github.com/bluealloy/revm/pull/2155))
- rename transact_previous to replay, move EvmTr traits ([#2153](https://github.com/bluealloy/revm/pull/2153))
- move mainnet builder to handler crate ([#2138](https://github.com/bluealloy/revm/pull/2138))
- remove `optional_gas_refund` as unused ([#2132](https://github.com/bluealloy/revm/pull/2132))
- Adding function derive_tx_type to TxEnv ([#2118](https://github.com/bluealloy/revm/pull/2118))
- remove wrong `&mut`/`TODO`, and avoid useless `get_mut` ([#2111](https://github.com/bluealloy/revm/pull/2111))
- export eip2930 eip7702 types from one place ([#2097](https://github.com/bluealloy/revm/pull/2097))
- move all dependencies to workspace ([#2092](https://github.com/bluealloy/revm/pull/2092))
- re-export all crates from `revm` ([#2088](https://github.com/bluealloy/revm/pull/2088))

## [1.0.0-alpha.1](https://github.com/bluealloy/revm/releases/tag/revm-context-v1.0.0-alpha.1) - 2025-02-16

### Added

- Split Inspector trait from EthHandler into standalone crate (#2075)
- Introduce Auth and AccessList traits (#2079)
- integrate alloy-eips (#2078)
- *(eip7702)* devnet6 changes and bump eest tests (#2055)
- Evm structure (Cached Instructions and Precompiles) (#2049)
- Context execution (#2013)
- EthHandler trait (#2001)
- *(EIP-7840)* Add blob schedule to execution client cfg (#1980)
- *(eip7702)* apply latest EIP-7702 changes, backport from v52 (#1969)
- simplify Transaction trait (#1959)
- align Block trait (#1957)
- expose precompile address in Journal, DB::Error: StdError (#1956)
- Make Ctx journal generic (#1933)
- Restucturing Part7 Handler and Context rework (#1865)
- restructuring Part6 transaction crate (#1814)
- *(examples)* generate block traces (#895)
- implement EIP-4844 (#668)
- *(Shanghai)* All EIPs: push0, warm coinbase, limit/measure initcode (#376)
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239)
- Introduce ByteCode format, Update Readme (#156)

### Fixed

- clear JournalState and set first journal vec (#1929)
- Clear journal (#1927)
- *(revme)* include correct bytecode for snailtracer  (#1917)
- fix typos ([#620](https://github.com/bluealloy/revm/pull/620))

### Other

- set alpha.1 version
- Add helpers with_inspector with_precompile (#2063)
- simplify some generics (#2032)
- Add helper functions for JournalInit #1879 (#1961)
- fix journal naming for inc/dec balance (#1976)
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
