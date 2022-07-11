
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