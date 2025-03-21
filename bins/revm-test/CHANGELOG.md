# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/bluealloy/revm/releases/tag/revm-test-v1.0.0) - 2025-03-21

### Added

- EOF (Ethereum Object Format) ([#1143](https://github.com/bluealloy/revm/pull/1143))
- add tests for shift instructions ([#1254](https://github.com/bluealloy/revm/pull/1254))
- EvmBuilder and External Contexts ([#888](https://github.com/bluealloy/revm/pull/888))
- separate initial checks ([#486](https://github.com/bluealloy/revm/pull/486))
- revm-interpreter created ([#320](https://github.com/bluealloy/revm/pull/320))
- *(interpreter)* Unify instruction fn signature ([#283](https://github.com/bluealloy/revm/pull/283))
- Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` ([#239](https://github.com/bluealloy/revm/pull/239))
- Introduce ByteCode format, Update Readme ([#156](https://github.com/bluealloy/revm/pull/156))

### Fixed

- *(eof)* fixture 2 tests ([#1550](https://github.com/bluealloy/revm/pull/1550))
- *(clippy)* fix some clippy lints

### Other

- tag v57 revm 19.6.0 ([#2169](https://github.com/bluealloy/revm/pull/2169))
- tag v56 revm v19.5.0 ([#2066](https://github.com/bluealloy/revm/pull/2066))
- revm v19.4.0 tag v54
- Bump v54 ([#1992](https://github.com/bluealloy/revm/pull/1992))
- v53 revm v19.2.0 ([#1972](https://github.com/bluealloy/revm/pull/1972))
- bump for v51, revm v19.0.0 versions ([#1953](https://github.com/bluealloy/revm/pull/1953))
- tag v50, revm v18.0.0, changelogs ([#1849](https://github.com/bluealloy/revm/pull/1849))
- v49 release ([#1833](https://github.com/bluealloy/revm/pull/1833))
- v48 bump versions and changelogs ([#1831](https://github.com/bluealloy/revm/pull/1831))
- bump major version number
- release-plz update
- Bump v46 versions ([#1826](https://github.com/bluealloy/revm/pull/1826))
- release-plz update ([#1807](https://github.com/bluealloy/revm/pull/1807))
- release-plz update
- *(deps)* bump alloy-sol-types from 0.8.0 to 0.8.2 ([#1762](https://github.com/bluealloy/revm/pull/1762))
- release ([#1729](https://github.com/bluealloy/revm/pull/1729))
- release ([#1722](https://github.com/bluealloy/revm/pull/1722))
- *(deps)* bump alloy and primitives ([#1725](https://github.com/bluealloy/revm/pull/1725))
- *(deps)* bump bytes from 1.6.1 to 1.7.1 ([#1700](https://github.com/bluealloy/revm/pull/1700))
- tag v41 revm v13.0.0 ([#1692](https://github.com/bluealloy/revm/pull/1692))
- release ([#1683](https://github.com/bluealloy/revm/pull/1683))
- *(deps)* bump regex from 1.10.5 to 1.10.6 ([#1682](https://github.com/bluealloy/revm/pull/1682))
- bump versions bcs of primitives ([#1631](https://github.com/bluealloy/revm/pull/1631))
- release ([#1620](https://github.com/bluealloy/revm/pull/1620))
- *(deps)* bump alloy-sol-types from 0.7.6 to 0.7.7 ([#1614](https://github.com/bluealloy/revm/pull/1614))
- *(deps)* bump alloy-sol-macro from 0.7.6 to 0.7.7 ([#1613](https://github.com/bluealloy/revm/pull/1613))
- release ([#1579](https://github.com/bluealloy/revm/pull/1579))
- release ([#1548](https://github.com/bluealloy/revm/pull/1548))
- replace TransactTo with TxKind ([#1542](https://github.com/bluealloy/revm/pull/1542))
- *(deps)* bump regex from 1.10.4 to 1.10.5 ([#1502](https://github.com/bluealloy/revm/pull/1502))
- release ([#1261](https://github.com/bluealloy/revm/pull/1261))
- *(interpreter)* rewrite gas accounting for memory expansion ([#1361](https://github.com/bluealloy/revm/pull/1361))
- revert snailtracer without microbench ([#1259](https://github.com/bluealloy/revm/pull/1259))
- release ([#1231](https://github.com/bluealloy/revm/pull/1231))
- *(deps)* bump other alloy deps 0.7.0 ([#1252](https://github.com/bluealloy/revm/pull/1252))
- *(deps)* bump regex from 1.10.3 to 1.10.4 ([#1223](https://github.com/bluealloy/revm/pull/1223))
- *(deps)* bump bytes from 1.5.0 to 1.6.0 ([#1224](https://github.com/bluealloy/revm/pull/1224))
- release ([#1175](https://github.com/bluealloy/revm/pull/1175))
- tag v32 revm v7.1.0 ([#1176](https://github.com/bluealloy/revm/pull/1176))
- release ([#1125](https://github.com/bluealloy/revm/pull/1125))
- *(deps)* bump alloy-sol-types from 0.6.3 to 0.6.4 ([#1148](https://github.com/bluealloy/revm/pull/1148))
- *(deps)* bump alloy-sol-macro from 0.6.3 to 0.6.4 ([#1136](https://github.com/bluealloy/revm/pull/1136))
- release tag v30 revm v6.1.0 ([#1100](https://github.com/bluealloy/revm/pull/1100))
- clippy cleanup ([#1112](https://github.com/bluealloy/revm/pull/1112))
- *(deps)* bump alloy-sol-types from 0.6.2 to 0.6.3 ([#1103](https://github.com/bluealloy/revm/pull/1103))
- release ([#1082](https://github.com/bluealloy/revm/pull/1082))
- *(deps)* bump alloy-sol-macro from 0.6.2 to 0.6.3 ([#1094](https://github.com/bluealloy/revm/pull/1094))
- license date and revm docs ([#1080](https://github.com/bluealloy/revm/pull/1080))
- release ([#1067](https://github.com/bluealloy/revm/pull/1067))
- tag v27, revm v4.0.0 release ([#1061](https://github.com/bluealloy/revm/pull/1061))
- *(deps)* bump eyre from 0.6.11 to 0.6.12 ([#1051](https://github.com/bluealloy/revm/pull/1051))
- *(deps)* bump alloy-sol-types from 0.6.0 to 0.6.2 ([#1035](https://github.com/bluealloy/revm/pull/1035))
- *(deps)* bump alloy-sol-macro from 0.6.0 to 0.6.2 ([#1013](https://github.com/bluealloy/revm/pull/1013))
- chore(Test) : const to static ([#1016](https://github.com/bluealloy/revm/pull/1016))
- Burntpix criterion bench ([#1004](https://github.com/bluealloy/revm/pull/1004))
- Instruction table ([#759](https://github.com/bluealloy/revm/pull/759))
- rewrite revm-test as a criterion bench ([#579](https://github.com/bluealloy/revm/pull/579))
- optimize stack usage for recursive `call` and `create` programs ([#522](https://github.com/bluealloy/revm/pull/522))
- Bump v24, revm v3.3.0 ([#476](https://github.com/bluealloy/revm/pull/476))
- Release v23, revm v3.2.0 ([#464](https://github.com/bluealloy/revm/pull/464))
- Release v22, revm v3.1.1 ([#460](https://github.com/bluealloy/revm/pull/460))
- v21, revm v3.1.0 ([#444](https://github.com/bluealloy/revm/pull/444))
- remove gas blocks ([#391](https://github.com/bluealloy/revm/pull/391))
- *(deps)* bump bytes from 1.3.0 to 1.4.0 ([#355](https://github.com/bluealloy/revm/pull/355))
- Bump v20, changelog ([#350](https://github.com/bluealloy/revm/pull/350))
- includes to libs ([#338](https://github.com/bluealloy/revm/pull/338))
- Creating revm-primitives, revm better errors and db components  ([#334](https://github.com/bluealloy/revm/pull/334))
- Cleanup, move hot fields toggether in Interpreter ([#321](https://github.com/bluealloy/revm/pull/321))
- native bits ([#278](https://github.com/bluealloy/revm/pull/278))
- *(release)* Bump revm and precompiles versions
- Bump primitive_types. Add statetest spec
- Bump revm v2.1.0 ([#224](https://github.com/bluealloy/revm/pull/224))
- revm bump v2.0.0, precompile bump v1.1.1 ([#212](https://github.com/bluealloy/revm/pull/212))
- Cfg choose create analysis, option on bytecode size limit ([#210](https://github.com/bluealloy/revm/pull/210))
- Cargo sort. Bump lib versions ([#208](https://github.com/bluealloy/revm/pull/208))
- Return `ExecutionResult`, which includes `gas_refunded` ([#169](https://github.com/bluealloy/revm/pull/169))
- Bytecode hash, remove override_spec, ([#165](https://github.com/bluealloy/revm/pull/165))
- revm bump 1.8. update libs. snailtracer rename ([#159](https://github.com/bluealloy/revm/pull/159))
- v6 changelog, bump versions
- Big Refactor. Machine to Interpreter. refactor instructions. call/create struct ([#52](https://github.com/bluealloy/revm/pull/52))
- [revm] pop_top and unsafe comments ([#51](https://github.com/bluealloy/revm/pull/51))
- [precompiles] remove unused borsh
- [recompl] Bump precompile deps, cargo sort on workspace
- [revm] output log. Stetetest test log output. fmt
- Bump versions, Changelogs, fmt, revm readme, clippy.
- [revm] Run test multiple times. fmt, BenchmarkDB
- Multiple changes: web3 db, debugger initial commit, precompile load
- Memory to usize, clippy,fmt
- wip optimize i256
- TEMP switch stacks H256 with U256
- [revm] some perfs
- [revm] Perfs stack pop. Benchmark snailtracer.
- [revm] cleanup
- fmt
- EVM Interface changed. Inspector called separately
- Bump revm v0.3.0. README updated
- DB ref mut polished
- And now we debug
- [revm] Interface. Inspector added, Env cleanup. revm-test passes
- Rename bin to bins
