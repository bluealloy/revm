# blake2 hash

This module represents a Rust implementation of the `Blake2b` cryptographic hash function, a vital component of Ethereum's broader EIP-152 proposal. The primary purpose of this module is to integrate the `Blake2b` function into Ethereum's precompiled contract mechanism, providing a consistent and efficient way to perform the cryptographic hashing that underpins Ethereum's functionality.

In [EIP-152](https://eips.ethereum.org/EIPS/eip-152) introduced a new precompiled contract that implements the `BLAKE2` cryptographic hashing algorithm's compression function. The purpose of this is to enhance the interoperability between Ethereum and Zcash, as well as to introduce more versatile cryptographic hash primitives to the Ethereum Virtual Machine (EVM).

BLAKE2 is not just a powerful cryptographic hash function and SHA3 contender, but it also allows for the efficient validation of the Equihash Proof of Work (PoW) used in Zcash. This could make a Bitcoin Relay-style Simplified Payment Verification (SPV) client feasible on Ethereum, as it enables the verification of Zcash block headers without excessive computational cost. `BLAKE2b`, a common 64-bit `BLAKE2` variant, is highly optimized and performs faster than MD5 on modern processors.

The rationale behind incorporating `Blake2b` into Ethereum's suite of precompiled contracts is multifaceted:

- Performance: The `Blake2b` hash function offers excellent performance, particularly when processing large inputs.
- Security: `Blake2b` also provides a high degree of security, making it a suitable choice for cryptographic operations.
- Interoperability: This function is widely used in various parts of the ecosystem, making it a prime candidate for inclusion in Ethereum's precompiled contracts.
- Gas Cost: The gas cost per round (F_ROUND) is specified as 1. This number was decided considering the computational complexity and the necessity to keep the blockchain efficient and prevent spamming.

## Core Components

Two primary constants provide the framework for the precompiled contract:

`F_ROUND: u64`: This is the cost of each round of computation in gas units. Currently set to 1.
`INPUT_LENGTH: usize`: This specifies the required length of the input data, 213 bytes in this case.

## Precompile Function - run

The `run` function is the main entry point for the precompiled contract. It consumes an input byte slice and a gas limit, returning a `PrecompileResult`. This function handles input validation, gas cost computation, data manipulation, and the compression algorithm.

It checks for correct input length and reads the final `block` flag. It then calculates the gas cost based on the number of rounds to be executed. If the gas cost exceeds the provided gas limit, it immediately returns an error.

Once the validation and gas cost computation are complete, it parses the input into three components: state vector `h`, message `block` vector `m`, and offset counter `t`.

Following this, it calls the `compress` function from the algo module, passing in the parsed input data and the final `block` flag.

Finally, it constructs and returns the `PrecompileResult` containing the gas used and the output data.

## Algorithm Module - algo

The algo module encapsulates the technical implementation of the `Blake2b` hash function. It includes several key elements:

Constants:

- `SIGMA`: This 2D array represents the message word selection permutation used in each round of the algorithm.

- `IV`: These are the initialization vectors for the `Blake2b` algorithm.

- The `g` Function: This is the core function within each round of the `Blake2b` algorithm. It manipulates the state vector and mixes in the message data.

- The `compress` Function: This is the main function that executes the rounds of the `g` function, handles the last `block` flag, and updates the state vector with the output of each round.
