# Secp256k1

This implements Ethereum's precompiled contract `ECRECOVER`, an elliptic curve digital signature algorithm (ECDSA) recovery function that recovers the Ethereum address (public key hash) associated with a given signature. The implementation features two versions, each contingent on whether the secp256k1 cryptographic library is enabled, which depends on the build configuration.

Both versions define a `secp256k1` module that includes an `ecrecover` function. This function takes a digital signature and a message as input, both represented as byte arrays, and returns the recovered Ethereum address. It performs this operation by using the signature to recover the original public key used for signing, then hashing this public key with `Keccak256`, Ethereum's chosen hash function. The hash is then truncated to match Ethereum's 20-byte address size.

When `secp256k1` is not enabled, the ecrecover function uses the `k256` library to parse the signature, recover the public key, and perform the hashing. When `secp256k1` is enabled, the function uses the `secp256k1` library for these operations. Although both versions perform the same fundamental operation, they use different cryptographic libraries, which can offer different optimizations and security properties.

The `ec_recover_run` function is the primary entry point for this precompiled contract. It parses the input to extract the message and signature, checks if enough gas is provided for execution, and calls the appropriate ecrecover function. The result of the recovery operation is returned as a `PrecompileResult`, a type that represents the outcome of a precompiled contract execution in Ethereum.
