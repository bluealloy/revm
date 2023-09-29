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