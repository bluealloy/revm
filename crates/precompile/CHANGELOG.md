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