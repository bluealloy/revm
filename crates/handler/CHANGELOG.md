# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [11.1.2](https://github.com/bluealloy/revm/compare/revm-handler-v11.1.1...revm-handler-v11.1.2) - 2025-10-15

### Other

- updated the following local packages: revm-bytecode, revm-state, revm-database-interface, revm-context-interface, revm-context, revm-database, revm-interpreter

## [11.1.1](https://github.com/bluealloy/revm/compare/revm-handler-v11.1.0...revm-handler-v11.1.1) - 2025-10-15

### Other

- resize short addresses bitvec instead of reallocating ([#3083](https://github.com/bluealloy/revm/pull/3083))
- *(handler)* extract duplicate gas price validation ([#3045](https://github.com/bluealloy/revm/pull/3045))

## [11.1.0](https://github.com/bluealloy/revm/compare/revm-handler-v11.0.0...revm-handler-v11.1.0) - 2025-10-09

### Other

- helper calculate_caller_fee ([#3040](https://github.com/bluealloy/revm/pull/3040))

## [11.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v10.0.1...revm-handler-v11.0.0) - 2025-10-07

### Added

- Support bubbling up first precompile error messages  ([#2905](https://github.com/bluealloy/revm/pull/2905))
- in JumpTable use Bytes instead of BitVec ([#3014](https://github.com/bluealloy/revm/pull/3014))
- add transaction index to batch execution error handling ([#3000](https://github.com/bluealloy/revm/pull/3000))
- allow EIP-7623 to be disabled ([#2985](https://github.com/bluealloy/revm/pull/2985))
- send bytecode with call input ([#2963](https://github.com/bluealloy/revm/pull/2963))
- *(revme)* ef blockchain tests cli ([#2935](https://github.com/bluealloy/revm/pull/2935))

### Fixed

- Apply spelling corrections from PRs #2926, #2915, #2908 ([#2978](https://github.com/bluealloy/revm/pull/2978))
- interpreter_result_mut should return mutable reference ([#2941](https://github.com/bluealloy/revm/pull/2941))
- FrameStack mark push/end_init as unsafe ([#2929](https://github.com/bluealloy/revm/pull/2929))

### Other

- changelog update for v87 ([#3056](https://github.com/bluealloy/revm/pull/3056))
- add boundless ([#3043](https://github.com/bluealloy/revm/pull/3043))
- helper function gas_balance_spending ([#3030](https://github.com/bluealloy/revm/pull/3030))
- helper caller_initial_modification added ([#3032](https://github.com/bluealloy/revm/pull/3032))
- Frame use From in place of Into ([#3036](https://github.com/bluealloy/revm/pull/3036))
- EvmTr and InspectorEvmTr receive all/all_mut fn ([#3037](https://github.com/bluealloy/revm/pull/3037))
- add ensure_enough_balance helper ([#3033](https://github.com/bluealloy/revm/pull/3033))
- prealloc few frames ([#2965](https://github.com/bluealloy/revm/pull/2965))
- Fix infinite recursion in EthPrecompiles PrecompileProvider methods ([#2962](https://github.com/bluealloy/revm/pull/2962))
- add SECURITY.md ([#2956](https://github.com/bluealloy/revm/pull/2956))
- update `EthFrame::invalid` visibility ([#2947](https://github.com/bluealloy/revm/pull/2947))
- remove unused generic from validate_tx_env and fix call site ([#2946](https://github.com/bluealloy/revm/pull/2946))
- cargo update ([#2930](https://github.com/bluealloy/revm/pull/2930))
- *(handler)* provide `&CallInputs`to`PrecompileProvider::run` ([#2921](https://github.com/bluealloy/revm/pull/2921))

## [10.0.1](https://github.com/bluealloy/revm/compare/revm-handler-v10.0.0...revm-handler-v10.0.1) - 2025-09-23

### Other

- updated the following local packages: revm-context-interface, revm-context, revm-interpreter

## [10.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v9.0.1...revm-handler-v10.0.0) - 2025-08-23

### Added

- *(fusaka)* Add PrecompileId ([#2904](https://github.com/bluealloy/revm/pull/2904))

## [9.0.1](https://github.com/bluealloy/revm/compare/revm-handler-v9.0.0...revm-handler-v9.0.1) - 2025-08-12

### Other

- updated the following local packages: revm-primitives, revm-bytecode, revm-state, revm-context-interface, revm-database, revm-precompile, revm-database-interface, revm-context, revm-interpreter

## [9.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v8.1.0...revm-handler-v9.0.0) - 2025-08-06

### Added

- short address for journal cold/warm check ([#2849](https://github.com/bluealloy/revm/pull/2849))
- gastable, record static gas in Interpreter loop ([#2822](https://github.com/bluealloy/revm/pull/2822))
- fix renamed functions for system_call ([#2824](https://github.com/bluealloy/revm/pull/2824))
- Align naming of SystemCallEvm function to ExecuteEvm ([#2814](https://github.com/bluealloy/revm/pull/2814))

### Fixed

- nonce changed is not reverted in journal if fail due to insufficient balance ([#2805](https://github.com/bluealloy/revm/pull/2805))

### Other

- update README.md ([#2842](https://github.com/bluealloy/revm/pull/2842))
- rm commented code ([#2839](https://github.com/bluealloy/revm/pull/2839))
- *(benches)* clean up criterion callsites ([#2833](https://github.com/bluealloy/revm/pull/2833))
- improve ExtBytecode hash handling ([#2826](https://github.com/bluealloy/revm/pull/2826))
- fix run-tests.sh ([#2801](https://github.com/bluealloy/revm/pull/2801))
- reuse global crypto provide idea ([#2786](https://github.com/bluealloy/revm/pull/2786))
- add rust-version and note about MSRV ([#2789](https://github.com/bluealloy/revm/pull/2789))
- Add dyn Crypto trait to PrecompileFn ([#2772](https://github.com/bluealloy/revm/pull/2772))
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [8.1.0](https://github.com/bluealloy/revm/compare/revm-handler-v8.0.3...revm-handler-v8.1.0) - 2025-07-23

### Added

- add a way for precompiles to revert ([#2711](https://github.com/bluealloy/revm/pull/2711))

### Fixed

- fully deprecate serde-json ([#2767](https://github.com/bluealloy/revm/pull/2767))
- system call should have 30M gas limit ([#2755](https://github.com/bluealloy/revm/pull/2755))
- gas deduction with `disable_balance_check` ([#2699](https://github.com/bluealloy/revm/pull/2699))

### Other

- change gas parameter to immutable reference ([#2702](https://github.com/bluealloy/revm/pull/2702))
- remove State bound from JournalTr in Handler::Evm ([#2715](https://github.com/bluealloy/revm/pull/2715))

## [8.0.3](https://github.com/bluealloy/revm/compare/revm-handler-v8.0.2...revm-handler-v8.0.3) - 2025-07-14

### Other

- simplify gas calculations by introducing a used() method ([#2703](https://github.com/bluealloy/revm/pull/2703))

## [8.0.2](https://github.com/bluealloy/revm/compare/revm-handler-v8.0.1...revm-handler-v8.0.2) - 2025-07-03

### Other

- document external state transitions for EIP-4788 and EIP-2935 ([#2678](https://github.com/bluealloy/revm/pull/2678))
- minor fixes ([#2686](https://github.com/bluealloy/revm/pull/2686))
- fix in pre_execution.rs about nonce bump for CREATE ([#2684](https://github.com/bluealloy/revm/pull/2684))

## [8.0.1](https://github.com/bluealloy/revm/compare/revm-handler-v7.0.1...revm-handler-v8.0.1) - 2025-06-30

### Added

- optional_eip3541 ([#2661](https://github.com/bluealloy/revm/pull/2661))

### Other

- cargo clippy --fix --all ([#2671](https://github.com/bluealloy/revm/pull/2671))
- use TxEnv::builder ([#2652](https://github.com/bluealloy/revm/pull/2652))
- fix copy-pasted inner doc comments ([#2663](https://github.com/bluealloy/revm/pull/2663))

## [7.0.1](https://github.com/bluealloy/revm/compare/revm-handler-v7.0.0...revm-handler-v7.0.1) - 2025-06-20

### Fixed

- call stack_frame.clear() at end ([#2656](https://github.com/bluealloy/revm/pull/2656))

## [7.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v6.0.0...revm-handler-v7.0.0) - 2025-06-19

### Added

- remove EOF ([#2644](https://github.com/bluealloy/revm/pull/2644))
- configurable contract size limit ([#2611](https://github.com/bluealloy/revm/pull/2611)) ([#2642](https://github.com/bluealloy/revm/pull/2642))
- *(precompile)* rug/gmp-based modexp ([#2596](https://github.com/bluealloy/revm/pull/2596))
- change blob_max_count to max_blobs_per_tx ([#2608](https://github.com/bluealloy/revm/pull/2608))
- add optional priority fee check configuration ([#2588](https://github.com/bluealloy/revm/pull/2588))

### Other

- lints handler inspector interpreter ([#2646](https://github.com/bluealloy/revm/pull/2646))
- bump all deps ([#2647](https://github.com/bluealloy/revm/pull/2647))
- re-use frame allocation ([#2636](https://github.com/bluealloy/revm/pull/2636))
- store coinbase address separately to avoid cloning warm addresses in the common case ([#2634](https://github.com/bluealloy/revm/pull/2634))
- rename `transact` methods ([#2616](https://github.com/bluealloy/revm/pull/2616))

## [6.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v5.0.1...revm-handler-v6.0.0) - 2025-06-06

### Added

- add with_caller for system_transact ([#2587](https://github.com/bluealloy/revm/pull/2587))
- *(Osaka)* EIP-7825 tx limit cap ([#2575](https://github.com/bluealloy/revm/pull/2575))
- expand timestamp/block_number to u256 ([#2546](https://github.com/bluealloy/revm/pull/2546))
- transact multi tx ([#2517](https://github.com/bluealloy/revm/pull/2517))

### Other

- tag v75 revm v24.0.1 ([#2563](https://github.com/bluealloy/revm/pull/2563)) ([#2589](https://github.com/bluealloy/revm/pull/2589))
- *(docs)* add lints to database-interface and op-revm crates ([#2568](https://github.com/bluealloy/revm/pull/2568))
- unify calling of journal account loading ([#2561](https://github.com/bluealloy/revm/pull/2561))
- ContextTr rm *_ref, and add *_mut fn ([#2560](https://github.com/bluealloy/revm/pull/2560))
- *(cfg)* add tx_chain_id_check fields. Optimize effective gas cost calc ([#2557](https://github.com/bluealloy/revm/pull/2557))
- simplify Interpreter loop ([#2544](https://github.com/bluealloy/revm/pull/2544))

## [5.0.1](https://github.com/bluealloy/revm/compare/revm-handler-v5.0.0...revm-handler-v5.0.1) - 2025-05-31

### Other

- unify calling of journal account loading

## [5.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v4.1.0...revm-handler-v5.0.0) - 2025-05-22

### Added

- make blob max number optional ([#2532](https://github.com/bluealloy/revm/pull/2532))

### Other

- add TxEnvBuilder::build_fill ([#2536](https://github.com/bluealloy/revm/pull/2536))
- make crates.io version badge clickable ([#2526](https://github.com/bluealloy/revm/pull/2526))
- fix clippy ([#2523](https://github.com/bluealloy/revm/pull/2523))
- Storage Types Alias ([#2461](https://github.com/bluealloy/revm/pull/2461))


## [4.1.0](https://github.com/bluealloy/revm/compare/revm-handler-v4.0.0...revm-handler-v4.1.0) - 2025-05-07

Dependency bump

## [4.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v3.0.1...revm-handler-v4.0.0) - 2025-05-07

### Added

- system_call switch order of inputs, address than bytes ([#2485](https://github.com/bluealloy/revm/pull/2485))
- *(Osaka)* disable EOF ([#2480](https://github.com/bluealloy/revm/pull/2480))
- skip cloning of call input from shared memory ([#2462](https://github.com/bluealloy/revm/pull/2462))
- Add a custom address to the CreateScheme. ([#2464](https://github.com/bluealloy/revm/pull/2464))
- remove spec id verification on `apply_eip7702_auth_list` ([#2466](https://github.com/bluealloy/revm/pull/2466))
- *(Handler)* merge state validation with deduct_caller ([#2460](https://github.com/bluealloy/revm/pull/2460))
- replace input Bytes and refactored code where required ([#2453](https://github.com/bluealloy/revm/pull/2453))
- *(tx)* Add Either RecoveredAuthorization ([#2448](https://github.com/bluealloy/revm/pull/2448))
- *(EOF)* Changes needed for devnet-1 ([#2377](https://github.com/bluealloy/revm/pull/2377))
- Move SharedMemory buffer to context ([#2382](https://github.com/bluealloy/revm/pull/2382))

### Fixed

- *(eof)* extdelegate bytecode check after eip7702 load ([#2417](https://github.com/bluealloy/revm/pull/2417))
- skip account list for legacy ([#2400](https://github.com/bluealloy/revm/pull/2400))

### Other

- Add clones to FrameData ([#2482](https://github.com/bluealloy/revm/pull/2482))
- Add Bytecode address to Interpreter ([#2479](https://github.com/bluealloy/revm/pull/2479))
- copy edit The Book ([#2463](https://github.com/bluealloy/revm/pull/2463))
- bump dependency version ([#2431](https://github.com/bluealloy/revm/pull/2431))
- fixed broken link ([#2421](https://github.com/bluealloy/revm/pull/2421))
- backport from release branch ([#2415](https://github.com/bluealloy/revm/pull/2415)) ([#2416](https://github.com/bluealloy/revm/pull/2416))
- *(lints)* revm-context lints ([#2404](https://github.com/bluealloy/revm/pull/2404))

## [3.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v2.0.0...revm-handler-v3.0.0) - 2025-04-09

### Added

- support for system calls ([#2350](https://github.com/bluealloy/revm/pull/2350))

### Other

- make blob params u64 ([#2385](https://github.com/bluealloy/revm/pull/2385))
- add 0x prefix to b256! and address! calls ([#2345](https://github.com/bluealloy/revm/pull/2345))

## [2.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0...revm-handler-v2.0.0) - 2025-03-28

### Added

- cache precompile warming ([#2317](https://github.com/bluealloy/revm/pull/2317))
- provide more context to precompiles ([#2318](https://github.com/bluealloy/revm/pull/2318))
- Add JournalInner ([#2311](https://github.com/bluealloy/revm/pull/2311))

### Fixed

- broken disable balance check ([#2286](https://github.com/bluealloy/revm/pull/2286))

### Other

- remove outdated TODO comments ([#2325](https://github.com/bluealloy/revm/pull/2325))
- add EIP-170 contract code size limit tests ([#2312](https://github.com/bluealloy/revm/pull/2312))
- Remove LATEST variant from SpecId enum ([#2299](https://github.com/bluealloy/revm/pull/2299))
- add unit test for EIP-3860 initcode size limit ([#2302](https://github.com/bluealloy/revm/pull/2302))

## [1.0.0](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.7...revm-handler-v1.0.0) - 2025-03-24

### Other

- updated the following local packages: revm-database, revm-precompile

## [1.0.0-alpha.7](https://github.com/bluealloy/revm/compare/revm-handler-v1.0.0-alpha.6...revm-handler-v1.0.0-alpha.7) - 2025-03-21

### Added

- Remove PrecompileError from PrecompileProvider ([#2233](https://github.com/bluealloy/revm/pull/2233))
- allow reuse of API for calculating initial tx gas for tx ([#2215](https://github.com/bluealloy/revm/pull/2215))

### Other

- remove wrong `&mut` and duplicated spec ([#2276](https://github.com/bluealloy/revm/pull/2276))
- Add custom instruction example ([#2261](https://github.com/bluealloy/revm/pull/2261))
- use AccessListItem associated type instead of AccessList ([#2214](https://github.com/bluealloy/revm/pull/2214))

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
