Because this is workspace with multi libraries, tags will be simplified, and with this document you can match version of project with git tag.

# v68 tag
date: 09.04.2025

Bump to alloy-primitives, this warants major bump on all libs. No breaking changes

* `revm-primitives`: 17.0.0 -> 18.0.0 (✓ API compatible changes)
* `revm-bytecode`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-state`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-database-interface`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-context-interface`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-context`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-database`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-interpreter`: 17.0.0 -> 18.0.0 (✓ API compatible changes)
* `revm-precompile`: 18.0.0 -> 19.0.0 (✓ API compatible changes)
* `revm-handler`: 2.0.0 -> 3.0.0 (⚠️ API breaking changes)
    * Two traits reexported in different mod
* `revm-inspector`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm`: 21.0.0 -> 22.0.0 (✓ API compatible changes)
* `revme`: 4.0.0 -> 4.0.1 (✓ API compatible changes)
* `op-revm`: 2.0.0 -> 3.0.0 (✓ API compatible changes)
* `revm-statetest-types`: 2.0.0 -> 3.0.0


# v67 tag
date: 28.03.2025

op-revm isthum fix.

* `revm-primitives`: 16.0.0 -> 17.0.0 (⚠ API breaking changes)
* `revm-bytecode`: 1.0.0 -> 2.0.0 (⚠ API breaking changes)
* `revm-database-interface`: 1.0.0 -> 2.0.0 (✓ API compatible changes)
* `revm-context-interface`: 1.0.0 -> 2.0.0 (✓ API compatible changes)
* `revm-context`: 1.0.0 -> 2.0.0 (⚠ API breaking changes)
* `revm-database`: 1.0.0 -> 2.0.0 (✓ API compatible changes)
* `revm-interpreter`: 16.0.0 -> 17.0.0 (✓ API compatible changes)
* `revm-precompile`: 17.0.0 -> 18.0.0 (⚠ API breaking changes)
* `revm-handler`: 1.0.0 -> 2.0.0 (⚠ API breaking changes)
* `revm-inspector`: 1.0.0 -> 2.0.0 (✓ API compatible changes)
* `revm`: 20.0.0 -> 21.0.0 (✓ API compatible changes)
* `revme`: 3.0.0 -> 4.0.0 (⚠ API breaking changes)
* `op-revm`: 1.0.0 -> 2.0.0 (⚠ API breaking changes)
* `revm-state`: 1.0.0 -> 2.0.0
* `revm-statetest-types`: 1.0.0 -> 2.0.0

# v66 tag
date: 24.03.205

Stable release of Revm new Execution API and Evm Framework.

* `revm-primitives`: 16.0.0-alpha.5 -> 16.0.0
* `revm-context-interface`: 1.0.0-alpha.6 -> 1.0.0
* `revm-context`: 1.0.0-alpha.6 -> 1.0.0
* `revm-database`: 1.0.0-alpha.5 -> 1.0.0
* `revm-interpreter`: 16.0.0-alpha.7 -> 16.0.0
* `revm-precompile`: 17.0.0-alpha.7 -> 17.0.0
* `revm-handler`: 1.0.0-alpha.7 -> 1.0.0
* `revm-inspector`: 1.0.0-alpha.7 -> 1.0.0
* `revme`: 3.0.0-alpha.6 -> 3.0.0
* `op-revm`: 1.0.0-alpha.6 -> 1.0.0
* `revm-bytecode`: 1.0.0-alpha.5 -> 1.0.0
* `revm-state`: 1.0.0-alpha.5 -> 1.0.0
* `revm-database-interface`: 1.0.0-alpha.5 -> 1.0.0
* `revm`: 20.0.0-alpha.7 -> 20.0.0

# v65 tag
date 23.03.2025

Optimism fixes, preo for release v20.0.0 release.
Breaking changes related to EVMError, more about this here: https://github.com/bluealloy/revm/pull/2280

* `revm-primitives`: 16.0.0-alpha.4 -> 16.0.0-alpha.5 (⚠ API breaking changes)
* `revm-context-interface`: 1.0.0-alpha.5 -> 1.0.0-alpha.6 (⚠ API breaking changes)
* `revm-context`: 1.0.0-alpha.5 -> 1.0.0-alpha.6 (⚠ API breaking changes)
* `revm-database`: 1.0.0-alpha.4 -> 1.0.0-alpha.5 (✓ API compatible changes)
* `revm-interpreter`: 16.0.0-alpha.6 -> 16.0.0-alpha.7 (✓ API compatible changes)
* `revm-precompile`: 17.0.0-alpha.6 -> 17.0.0-alpha.7 (⚠ API breaking changes)
* `revm-handler`: 1.0.0-alpha.6 -> 1.0.0-alpha.7 (✓ API compatible changes)
* `revm-inspector`: 1.0.0-alpha.6 -> 1.0.0-alpha.7 (⚠ API breaking changes)
* `revme`: 3.0.0-alpha.6 -> 3.0.0-alpha.7 (✓ API compatible changes)
* `op-revm`: 1.0.0-alpha.5 -> 1.0.0-alpha.6 (⚠ API breaking changes)
* `revm-bytecode`: 1.0.0-alpha.4 -> 1.0.0-alpha.5
* `revm-state`: 1.0.0-alpha.4 -> 1.0.0-alpha.5
* `revm-database-interface`: 1.0.0-alpha.4 -> 1.0.0-alpha.5
* `revm`: 20.0.0-alpha.6 -> 20.0.0-alpha.7

# v63 tag
date: 16.03.2025

Docs, prep for v20.0.0 release.

* `revm-primitives`: 16.0.0-alpha.3 -> 16.0.0-alpha.4 (✓ API compatible changes)
* `revm-bytecode`: 1.0.0-alpha.3 -> 1.0.0-alpha.4 (⚠️ API breaking changes)
* `revm-context-interface`: 1.0.0-alpha.4 -> 1.0.0-alpha.5 (✓ API compatible changes)
* `revm-context`: 1.0.0-alpha.4 -> 1.0.0-alpha.5 (✓ API compatible changes)
* `revm-precompile`: 17.0.0-alpha.5 -> 17.0.0-alpha.6 (✓ API compatible changes)
* `revm-handler`: 1.0.0-alpha.5 -> 1.0.0-alpha.6 (✓ API compatible changes)
* `revm-inspector`: 1.0.0-alpha.5 -> 1.0.0-alpha.6 (✓ API compatible changes)
* `op-revm`: 1.0.0-alpha.4 -> 1.0.0-alpha.5 (⚠️ API breaking changes)
* `revm-state`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-database-interface`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-database`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-interpreter`: 16.0.0-alpha.5 -> 16.0.0-alpha.6
* `revm`: 20.0.0-alpha.5 -> 20.0.0-alpha.6
* `revme`: 3.0.0-alpha.5 -> 3.0.0-alpha.6

# v62 tag
date: 12.03.2025

A few small breaking changed in preparation for v20.0.0.

* `revm-context-interface`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-context`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-interpreter`: 16.0.0-alpha.4 -> 16.0.0-alpha.5
* `revm-handler`: 1.0.0-alpha.4 -> 1.0.0-alpha.5
* `revm-inspector`: 1.0.0-alpha.4 -> 1.0.0-alpha.5
* `revme`: 3.0.0-alpha.4 -> 3.0.0-alpha.5
* `op-revm`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-precompile`: 17.0.0-alpha.4 -> 17.0.0-alpha.5
* `revm`: 20.0.0-alpha.4 -> 20.0.0-alpha.5

# v61 tag
date: 11.03.2025

Bug fixes for op-revm.

* `revm-primitives`: 16.0.0-alpha.2 -> 16.0.0-alpha.3
* `revm-bytecode`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-state`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-database-interface`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-context-interface`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-context`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-database`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-interpreter`: 16.0.0-alpha.3 -> 16.0.0-alpha.4
* `revm-precompile`: 17.0.0-alpha.3 -> 17.0.0-alpha.4
* `revm-handler`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm-inspector`: 1.0.0-alpha.3 -> 1.0.0-alpha.4
* `revm`: 20.0.0-alpha.3 -> 20.0.0-alpha.4
* `revme`: 3.0.0-alpha.3 -> 3.0.0-alpha.4
* `op-revm`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-statetest-types`: 1.0.0-alpha.3 -> 1.0.0-alpha.4

# v60 tag
date: 10.03.2025

Bug fix on blockhash opcode.

* `revm-interpreter`: 16.0.0-alpha.2 -> 16.0.0-alpha.3
* `revm-precompile`: 17.0.0-alpha.2 -> 17.0.0-alpha.3
* `revm`: 20.0.0-alpha.2 -> 20.0.0-alpha.3
* `revm-handler`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-inspector`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revm-statetest-types`: 1.0.0-alpha.2 -> 1.0.0-alpha.3
* `revme`: 3.0.0-alpha.2 -> 3.0.0-alpha.3
* `op-revm`: 1.0.0-alpha.1 -> 1.0.0-alpha.2


# v59 tag
date: 10.03.2025

* Few bugs fixes mostly for optimism crate.
* remv-optimism renamed to op-revm.
* revm-specification files moved to revm-primitives
* docs, initial book and cleanup.

Versions:

* `revm-primitives`: 16.0.0-alpha.1 -> 16.0.0-alpha.2
* `revm-bytecode`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-state`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-database-interface`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-context-interface`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-context`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-database`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-interpreter`: 16.0.0-alpha.1 -> 16.0.0-alpha.2
* `revm-precompile`: 17.0.0-alpha.1 -> 17.0.0-alpha.2
* `revm-handler`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm-inspector`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revm`: 20.0.0-alpha.1 -> 20.0.0-alpha.2
* `revm-statetest-types`: 1.0.0-alpha.1 -> 1.0.0-alpha.2
* `revme`: 3.0.0-alpha.1 -> 3.0.0-alpha.2
* `op-revm`: 1.0.0-alpha.1

# v57 tag
date 16.02.2025

Big refactor of the code.
Introduction of Revm Framework a way to extend evm without forking.

* `revm` = 19.5.0 -> 20.0.0-alpha.1
* `revm-primitives` = 15.2.0 -> 16.0.0-alpha.1
* `revm-interpreter` = 15.2.0 -> 16.0.0-alpha.1
* `revm-precompile` = 16.1.0 -> 17.0.0-alpha.1
* `revme` = 2.5.0 -> 3.0.0-alpha.1

New crates:
* `revm-bytecode` = 1.0.0-alpha.1
* `revm-database` = 1.0.0-alpha.1
* `revm-database-interface` = 1.0.0-alpha.1
* `revm-specification` = 1.0.0-alpha.1
* `revm-state` = 1.0.0-alpha.1
* `revm-inspector` = 1.0.0-alpha.1
* `revm-statetest-types` = 1.0.0-alpha.1
* `revm-context` = 1.0.0-alpha.1
* `revm-context-interface` = 1.0.0-alpha.1
* `revm-handler` = 1.0.0-alpha.1



# v56 tag
date: 11.02.2025

Optimism fixes and improvements.

* `revm`: 19.4.0 -> 19.5.0
* `revm-interpreter`: 15.1.0 -> 15.2.0
* `revm-primitives`: 15.1.0 -> 15.2.0
* `revm-precompile`: 16.0.0 -> 16.1.0
* `revme`: 2.4.0 -> 2.5.0

# v55 tag

date: 28.01.2025

Small release for Prague devnet-6 network.

* `revme`: 2.3.0 -> 2.4.0
* `revm`: 19.3.0 -> 19.4.0

# v54 tag

date: 13.01.2025

Changes for Prague devnet-5 network.

* `revme`: 2.2.0 -> 2.3.0
* `revm`: 19.2.0 -> 19.3.0

# v53 tag

date: 06.01.2025

Fix for previous release related to Optimism.

* `revm`: 19.1.0 -> 19.2.0

# v52 tag

date: 06.01.2025

Optimism isthmus spec added.

* `revm`: 19.0.0 -> 19.1.0

# v51 tag
date 26.12.2024
devnet-5 release.

* `revme`: 2.1.0 -> 2.2.0
* `revm`: 18.0.0 -> 19.0.0
* `revm-interpreter`: 14.0.0 -> 15.0.0
* `revm-primitives`: 14.0.0 -> 15.1.0
* `revm-precompile`: 15.0.0 -> 16.0.0

# v50 tag
date 06.11.2024
Maintenance release. Bump alloy-primitives deps, few utilities.

* `revme`: 2.0.0 -> 2.1.0
* `revm`: 17.1.0 -> 18.0.0
* `revm-interpreter`: 13.0.0 -> 14.0.0
* `revm-primitives`: 13.0.0 -> 14.0.0
* `revm-precompile`: 14.0.0 -> 15.0.0

# v49 tag
date 23.10.2024
Maintenance release. Bump alloydb deps.

* `revm`: 17.0.0 -> 17.1.0 (✓ API compatible changes)

# v48 tag
date 23.10.2024
Maintenance release. Bug fix for EIP-7702.

* `revm`: 16.0.0 -> 17.0.0 (✓ API compatible changes)
* `revm-primitives`: 12.0.0 -> 13.0.0 (✓ API compatible changes)
* `revme`: 1.0.0 -> 2.0.0
* `revm-interpreter`: 12.0.0 -> 13.0.0
* `revm-precompile`: 13.0.0 -> 14.0.0

# v47 tag
date: 17.10.2024
Maintenance release. bumping new alloy-eip7702

* `revme`: 0.11.0 -> 1.0.0
* `revm`: 15.0.0 -> 16.0.0 
* `revm-primitives`: 11.0.0 -> 12.0.0
* `revm-precompile`: 12.0.0 -> 13.0.0
* `revm-interpreter`: 11.0.0 -> 12.0.0

# v46 tag
date: 17.10.2024
Maintenance release. EIP-7702 newest changes, alloy-primitives bump.

* `revme`: 0.10.3 -> 0.11.0
* `revm`: 14.0.3 -> 15.0.0 
* `revm-primitives`: 10.0.0 -> 11.0.0
* `revm-precompile`: 11.0.3 -> 12.0.0
* `revm-interpreter`: 10.0.3 -> 11.0.0

# v45 tag
date: 26.09.2024

Maintenance release.

* `revme`: 0.10.2 -> 0.10.3 
* `revm`: 14.0.2 -> 14.0.3 
* `revm-primitives`: 9.0.2 -> 10.0.0
* `revm-interpreter`: 10.0.2 -> 10.0.3
* `revm-precompile`: 11.0.2 -> 11.0.3

# v44 tag
date: 18.09.2024

Small maintenance release.
Code can be found in release/v44 branch.
Fixes bug with Inspector selfdestruct not called every time, and enabled PRAGUE_EOF in statetest for PRAGUE tests.

* `revme`: 0.10.1 -> 0.10.2
* `revm`: 14.0.1 -> 14.0.2
* `revm-interpreter`: 10.0.1 -> 10.0.2
* `revm-primitives`: 9.0.1 -> 9.0.2
* `revm-precompile`: 11.0.1 -> 11.0.2
* `revm-test`: 0.1.0

# v43 tag
date: 30.08.2024

Logo change and doc fix.

* `revm`: 14.0.0 -> 14.0.1
* `revm-interpreter`: 10.0.0 -> 10.0.1
* `revm-primitives`: 9.0.0 -> 9.0.1
* `revm-precompile`: 11.0.0 -> 11.0.1
* `revme`: 0.10.0 -> 0.10.1

# v42 tag
date: 29.08.2024

new EIP-7702 implemented. Passing all EOF and EIP-7702 tests.
Preparation for devnet-3.

* `revme`: 0.9.0 -> 0.10.0
* `revm`: 13.0.0 -> 14.0.0
* `revm-interpreter`: 9.0.0 -> 10.0.0
* `revm-primitives`: 8.0.0 -> 9.0.0
* `revm-precompile`: 10.0.0 -> 11.0.0

# v41 tag
date: 08.08.2024

EOF fixes and improvements.
Optimism Granite fork support.

* `revme`: 0.8.0 -> 0.9.0
* `revm`: 12.1.0 -> 13.0.0
* `revm-interpreter`: 8.1.0 -> 9.0.0
* `revm-primitives`: 7.1.0 -> 8.0.0
* `revm-precompile`: 9.2.0 -> 10.0.0
* `revm-test`: 0.1.0

# v40 tag
date 17.07.2024

EOF bugfix.

* revm: 12.0.0 -> 12.1.0
* revm-interpreter: 8.0.0 -> 8.1.0
* revm-primitives: 7.0.0 -> 7.1.0
* revm-precompile: 9.1.0 -> 8.2.0

# v39 tag
date: 16.07.2024

Fixes for eip7702 and EOF. Kzg precompile alternative kzg-rs added. 

* revme: 0.7.0 -> 0.8.0
* revm: 11.0.0 -> 12.0.0
* revm-interpreter: 7.0.0 -> 8.0.0
* revm-primitives: 6.0.0 -> 7.0.0
* revm-precompile: 9.0.0 -> 9.1.0

# v38 tag
date: 08.07.2024

* Add EIP-7702 for Prague.
* Import AccessList from alloy-eips repo.
* EOF fixes
* Utility changes.

Versions
* revme: 0.6.0 -> 0.7.0
* revm: 10.0.0 -> 11.0.0
* revm-interpreter: 6.0.0 -> 7.0.0
* revm-primitives: 5.0.0 -> 6.0.0
* revm-precompile: 8.0.0 -> 9.0.0

# v37 tag
date: 20.06.2024

Audit of the codebase announced: https://hackmd.io/G7zazTX4TtekCnj6xlgctQ
secp256r1 precompile added.

Prague changes:
* EOF bugs squashed.
* Introducing PragueEOF hardfork.
* EIP-2935 (blockhashes) modified for devnet-1.
* Fixed for BLS12-381 curve.

Versions:
* revme: 0.5.0 -> 0.6.0
* revm: 9.0.0 -> 10.0.0
* revm-interpreter: 5.0.0 -> 6.0.0
* revm-primitives: 4.0.0 -> 5.0.0
* revm-precompile: 7.0.0 -> 8.0.0

# v36 tag
date: 12.05.2024

Support for prague EIPs.
* EOF not fully tested but most of implementation is there.
* EIP-2537: BLS12-381 curve operations
* EIP-2935: Serve historical block hashes from state

EOF removed BytecodeLocked, OpCode table got changed, and CallInputs got refactored.

* revme: 0.4.0 -> 0.5.0 (⚠️ API breaking changes)
* revm: 8.0.0 -> 9.0.0 (⚠️ API breaking changes)
* revm-interpreter: 4.0.0 -> 5.0.0 (⚠️ API breaking changes)
* revm-primitives: 3.1.1 -> 4.0.0 (⚠️ API breaking changes)
* revm-precompile: 6.0.0 -> 7.0.0 (⚠️ API breaking changes)
* revm-test: 0.1.0

# v35 tag
date: 02.04.2024

Small release. Alloy bump. Small refactors and deprecated functions removed.

* revme: 0.3.1 -> 0.4.0 (✓ API compatible changes)
* revm: 7.2.0 -> 8.0.0 (⚠️ API breaking changes)
* revm-interpreter: 3.4.0 -> 4.0.0 (⚠️ API breaking changes)
* revm-primitives: 3.1.0 -> 3.1.1 (✓ API compatible changes)
* revm-precompile: 5.1.0 -> 6.0.0 (⚠️ API breaking changes)
* revm-test: 0.1.0

# v34 tag
date: 20.03.2024

Small release, few utilities and refactoring, precompiles fn and Interpreter helper macros are made public.

* revme: 0.3.0 -> 0.3.1 (✓ API compatible changes)
* revm: 7.1.0 -> 7.2.0 (✓ API compatible changes)
* revm-interpreter: 3.3.0 -> 3.4.0 (✓ API compatible changes)
* revm-primitives: 3.0.0 -> 3.1.0 (✓ API compatible changes)
* revm-precompile: 5.0.0 -> 5.1.0 (✓ API compatible changes)

# v33 tag TODO

# v32 tag
date: 08.03.2024

Publish revm v7.1.0 that extends v7.0.0 with more restrictive context precompile.

* revm: 7.0.0(yanked) -> 7.1.0 (⚠️ API breaking changes)
* revm-interpreter: 3.2.0 -> 3.3.0 (✓ API compatible changes)

# v31 tag
date 08.03.2024

Stateful and context aware precompiles types added. Few improvements and fixes.

* revme: 0.2.2 -> 0.3.0 (⚠️ API breaking changes)
* revm: 6.1.0 -> 7.0.0(yanked) (⚠️ API breaking changes)
* revm-interpreter: 3.1.0 -> 3.2.0 (✓ API compatible changes)
* revm-primitives: 2.1.0 -> 3.0.0 (⚠️ API breaking changes)
* revm-precompile: 4.1.0 -> 5.0.0 (⚠️ API breaking changes)

# v30 tag
date: 23.02.2024

Small release.
Fixes db panic propagation and OP l1block load after cancun.

* revme: 0.2.1 -> 0.2.2 (✓ API compatible changes)
* revm: 6.0.0 -> 6.1.0 (✓ API compatible changes)
* revm-interpreter: 3.0.0 -> 3.1.0 (✓ API compatible changes)
* revm-primitives: 2.0.1 -> 2.1.0 (✓ API compatible changes)
* revm-precompile: 4.0.1 -> 4.1.0 (✓ API compatible changes)

# v29 tag
date: 17.02.2024

Small release, `return_memory_range` included inside `CallInput`.
Few fixes.

* revm: 5.0.0 -> 6.0.0 (⚠️ API breaking changes)
* revm-interpreter: 2.1.0 -> 3.0.0 (⚠️ API breaking changes)
* revm-primitives: 2.0.0 -> 2.0.1 (✓ API compatible changes)
* revm-precompile: 4.0.0 -> 4.0.1 (✓ API compatible changes)

# v28 tag
date: 12.02.2024

Small release, function renaming and some helper functions added.

* revm: 4.0.0 -> 5.0.0 (⚠️ API breaking changes)
* revm-interpreter: 2.0.0 -> 2.1.0 (✓ API compatible changes)
* revm-precompile: 3.0.0 -> 4.0.0 (⚠️ API breaking changes)
* revm-test: 0.1.0

# v27 tag
date: 07.02.2024

Refactor of Evm logic as list of handlers inside EvmHandler and EvmBuilder that open up the Evm and allow overwriting the default behavior.
Change how call loop (Previously it was recursion) is handled in Evm

* revm: v4.0.0
* revm-precompile: v3.0.0
* revm-primitives: v2.0.0
* revm-interpreter: v2.0.0
* revme: 0.2.1

# v26 tag
date 02.10.2023

Migration to alloy primitive types.

* revm: v3.5.0
* revm-precompile: v2.2.0
* revm-primitives: v1.3.0
* revm-interpreter: v1.3.0

# v25 tag
date: 28.09.2023

Bigger release. Cancun support, revm State added and some cleanup refactoring.

* revm: v3.4.0
* revm-precompile: v2.1.0
* revm-primitives: v1.2.0
* revm-interpreter: v1.2.0


# v24 tag
date: 03.05.2023

Consensus bug inside journal and some small changes.

* revm: v3.3.0
* revm-precompile: v2.0.3
* revm-primitives: v1.1.2
* revm-interpreter: v1.1.2

# v23 tag
date: 19.04.2023

consensus bug fix inside journal.

* revm: v3.2.0

# v22 tag
date: 14.04.2023

Fix for k256 build

* revm: v3.1.1
* revm-precompile: v2.0.2
* revm-primitives: v1.1.1
* revm-interpreter: v1.1.1

# v21 tag
date 04.04.2023

Shanghai supported and gas block optimization removed.

* revm: v3.1.0
* revm-precompile: v2.0.1
* revm-primitives: v1.1.0
* revm-interpreter: v1.1.0

# v20 tag
date 29.01.2023
Big release. primitives and interpreter libs and optimizations.
This tag can be found in `main`

* revm: v3.0.0
* revm-precompile: v2.0.0
* revm-primitives: v1.0.0
* revm-interpreter: v1.0.0

# v19 tag
data 22.11.2022
Bump dependency in revm and precompiles
Found on same branch as v17 tag.

* revm: v2.3.1
* revm_precompiles: v1.1.2

# v18 tag
date: 16.11.2022
Found on same branch as v17 tag.

* revm: v2.3.0

# v17 tag
date: 12.11.2022
code with the tag can be found in `release/v17` branch, reason is that `ruint` commit merged in `main` isn't going in this release.

* revm: v2.2.0 consensus bug fix

# v16 tag
date: 25.09.2022

* revm: v2.1.0

# v15 tag
date: 10.09.2022

* revm: v2.0.0 consensus bug fix
* revm_precompiles: v1.1.1
# v14 tag
date: 09.08.2022

* revm: v1.9.0

# v13 tag
date: 01.08.2022

* revm: v1.8.0

# v12 tag
date: 11.06.2022

* revm: v1.7.0
* revm_precompiles: v1.1.0

# v11 tag
date: 02.06.2022

* revm: v1.6.0

# v10 tag
date: 09.06.2022

* revm: v1.5.0: consensus bug fix

# v9 tag [small release]
date 06.06.2022

* revm: v1.4.1
# v8 tag [small release]
date: 03.06.2022

* revm: v1.4.0
# v7 tag [small release]
date: 11.5.2022

* revm: v1.3.1
# v6 tag
date: 30.4.2022

* revm: v1.3.0
* revm_precompiles: v1.0.0

# v5 tag
date: 20.1.2022

* revm_precompiles: v0.4.0
* revm: v1.2.0

# v4 tag
* revm: v1.1.0

# v3 tag

* revm: v1.0.0 
* revme: v0.1.0

# v2 tag

* revm: v0.5.0
* revm_precompiles: v0.3.0

# v1 tag

* revm: v0.4.0
* revm_precompiles: v0.2.0
*revmjs: v0.1.0
