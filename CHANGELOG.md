Because this is workspace with multi libraries, tags will be simplified, and with this document you can match version of project with git tag.

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

Cosnensus bug inside journal and some small changes.

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
