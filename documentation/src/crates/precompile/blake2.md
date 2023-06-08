# blake2 hash

In [EIP-152](https://eips.ethereum.org/EIPS/eip-152) introduced a new precompiled contract that implements the `BLAKE2` cryptographic hashing algorithm's compression function. The purpose of this is to enhance the interoperability between Ethereum and Zcash, as well as to introduce more versatile cryptographic hash primitives to the Ethereum Virtual Machine (EVM).

BLAKE2 is not just a powerful cryptographic hash function and SHA3 contender, but it also allows for the efficient validation of the Equihash Proof of Work (PoW) used in Zcash. This could make a Bitcoin Relay-style Simplified Payment Verification (SPV) client feasible on Ethereum, as it enables the verification of Zcash block headers without excessive computational cost. `BLAKE2b`, a common 64-bit `BLAKE2` variant, is highly optimized and performs faster than MD5 on modern processors.
