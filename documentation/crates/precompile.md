# Precompile

The precompile crate contains the implementation of the precompile opcodes in the evm. Precompiles are a shortcut to execute a function implemented by the EVM itself, rather than an actual contract. Precompiled contracts are essentially predefined smart contracts on Ethereum, residing at hardcoded addresses and used for computationally heavy operations that are cheaper when implemented this way. There are 6 precompiles implemented in REVM, and they are: blake2, bn128 curve, identity, secp256k1, modexp, and sha256 and ripemd160 has functions.

## blake2 hash

In [EIP-152](https://eips.ethereum.org/EIPS/eip-152) introduced a new precompiled contract that implements the `BLAKE2` cryptographic hashing algorithm's compression function. The purpose of this is to enhance the interoperability between Ethereum and Zcash, as well as to introduce more versatile cryptographic hash primitives to the Ethereum Virtual Machine (EVM).

BLAKE2 is not just a powerful cryptographic hash function and SHA3 contender, but it also allows for the efficient validation of the Equihash Proof of Work (PoW) used in Zcash. This could make a Bitcoin Relay-style Simplified Payment Verification (SPV) client feasible on Ethereum, as it enables the verification of Zcash block headers without excessive computational cost. `BLAKE2b`, a common 64-bit `BLAKE2` variant, is highly optimized and performs faster than MD5 on modern processors.

## bn128 curve

[EIP-197](https://eips.ethereum.org/EIPS/eip-197) proposed the addition of precompiled contracts for a pairing function on a specific pairing-friendly elliptic curve. This complements [EIP-196](https://eips.ethereum.org/EIPS/eip-196) in enabling zkSNARKs verification within Ethereum smart contracts. zkSNARKs (Zero-Knowledge Succinct Non-Interactive Argument of Knowledge) technology can enhance privacy for Ethereum users due to its Zero-Knowledge property. Moreover, it may offer a scalability solution because of its succinctness and efficient verifiability property.

Prior to this EIP, Ethereum's smart contract executions were fully transparent, limiting their use in cases involving private information, such as location, identity, or transaction history. While the Ethereum Virtual Machine (EVM) can theoretically use zkSNARKs, their implementation was presently too costly to fit within the block gas limit. [EIP-197](https://eips.ethereum.org/EIPS/eip-197) defines specific parameters for basic primitives that facilitate zkSNARKs. This allows for more efficient implementation, thereby reducing gas costs.

Notably, setting these parameters doesn't restrict zkSNARKs' use-cases but actually enables the integration of zkSNARK research advancements without requiring further hard forks. Pairing functions, which enable a limited form of multiplicatively homomorphic operations necessary for zkSNARKs, could then be executed within the block gas limit through this precompiled contract.

The code consists of three modules: `add`, `mul`, and `pair`. The add and `mul` modules implement elliptic curve point addition and scalar multiplication respectively on the bn128 curve, an elliptic curve utilized within Ethereum. Each module defines two versions of the contract, one for the Istanbul and another for the Byzantium Ethereum network upgrades.

The pair module conducts the pairing check, an operation that enables comparison of two points on the elliptic curve, an essential part of many zero-knowledge proof systems, including zk-SNARKs. Again, two versions for Istanbul and Byzantium are defined. The `run_add`, `run_mul`, and run_pair functions embody the main implementations of the precompiled contracts, with each function accepting an input byte array, executing the appropriate elliptic curve operations, and outputting the results as a byte array.

The code ensures the allocation of sufficient gas for each operation by stipulating gas costs as constants at the start of each module. It employs the bn library to carry out the actual bn128 operations. As the functions operate with byte arrays, the code features significant byte manipulation and conversion. Consequently, the code presents an implementation of specific elliptic curve operations utilized in Ethereum.

## Hash functions

REVM includes precompiled contracts for `SHA256` and `RIPEMD160`, cryptographic hashing functions integral for data integrity and security. The addresses for these precompiled contracts are `0x0000000000000000000000000000000000000002` for `SHA256` and `0x0000000000000000000000000000000000000003` for `RIPEMD160`.

Each function (`sha256_run` and `ripemd160_run`) accepts two arguments, the input data to be hashed and the gas_limit representing the maximum amount of computational work permissible for the function. They both calculate the gas cost of the operation based on the input data length. If the computed cost surpasses the `gas_limit`, an `Error::OutOfGas` is triggered.

The `sha256_run` function, corresponding to the `SHA256` precompiled contract, computes the `SHA256` hash of the input data. The `ripemd160_run` function computes the `RIPEMD160` hash of the input and pads it to match Ethereum's 256-bit word size. These precompiled contracts offer a computationally efficient way for Ethereum contracts to perform necessary cryptographic operations.

## Identity function

This precompiled contract performs the identity function. In mathematics, an identity function is a function that always returns the same value as its argument. In this context, the contract takes the input data and returns it as is. This precompiled contract resides at the hardcoded Ethereum address `0x0000000000000000000000000000000000000004`.

The `identity_run` function takes two arguments: input data, which it returns unaltered, and `gas_limit` which defines the maximum computational work the function is allowed to do. A linear gas cost calculation based on the size of the input data and two constants, `IDENTITY_BASE` (the base cost of the operation) and `IDENTITY_PER_WORD` (the cost per word), is performed. If the calculated gas cost exceeds the `gas_limit`, an `Error::OutOfGas` is returned.

This identity function can be useful in various scenarios such as forwarding data or acting as a data validation check within a contract. Despite its simplicity, it contributes to the flexibility and broad utility of the Ethereum platform.

## Modexp

REVM also implements two versions of a precompiled contract (Modular Exponential operation), each corresponding to different Ethereum hard forks: Byzantium and Berlin. The contract addresses are `0x0000000000000000000000000000000000000005` for both versions, as they replaced each other in subsequent network upgrades. This operation is used for cryptographic computations and is a crucial part of Ethereum's toolkit.

The byzantium_run and berlin_run functions each run the modular exponential operation using the `run_inner` function, but each uses a different gas calculation method: `byzantium_gas_calc` for Byzantium and `berlin_gas_calc` for Berlin. The gas calculation method used is chosen based on the Ethereum network's current version. The `run_inner` function is a core function that reads the inputs and performs the modular exponential operation. If the calculated gas cost is higher than the gas limit, an error `Error::OutOfGas` is returned. If all computations are successful, the function returns the result of the operation and the gas cost.

The calculate_iteration_count function calculates the number of iterations required to compute the operation, based on the length and value of the exponent. The `read_u64_with_overflow` macro reads input data and checks for potential overflows.

The byzantium_gas_calc function calculates the gas cost for the modular exponential operation as defined in the Byzantium version of the Ethereum protocol. The `berlin_gas_calc` function calculates the gas cost according to the Berlin version, as defined in [EIP-2565](https://eips.ethereum.org/EIPS/eip-2565). These two versions have different formulas to calculate the gas cost of the operation, reflecting the evolution of the Ethereum network.

## Secp256k1

This implementation Ethereum's precompiled contract `ECRECOVER`, an elliptic curve digital signature algorithm (ECDSA) recovery function that recovers the Ethereum address (public key hash) associated with a given signature. The implementation features two versions, each contingent on whether the secp256k1 cryptographic library is enabled, which depends on the build configuration.

Both versions define a `secp256k1` module that includes an `ecrecover` function. This function takes a digital signature and a message as input, both represented as byte arrays, and returns the recovered Ethereum address. It performs this operation by using the signature to recover the original public key used for signing, then hashing this public key with `Keccak256`, Ethereum's chosen hash function. The hash is then truncated to match Ethereum's 20-byte address size.

When `secp256k1` is not enabled, the ecrecover function uses the `k256` library to parse the signature, recover the public key, and perform the hashing. When `secp256k1` is enabled, the function uses the `secp256k1` library for these operations. Although both versions perform the same fundamental operation, they use different cryptographic libraries, which can offer different optimizations and security properties.

The `ec_recover_run` function is the primary entry point for this precompiled contract. It parses the input to extract the message and signature, checks if enough gas is provided for execution, and calls the appropriate ecrecover function. The result of the recovery operation is returned as a `PrecompileResult`, a type that represents the outcome of a precompiled contract execution in Ethereum.
