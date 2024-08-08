# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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