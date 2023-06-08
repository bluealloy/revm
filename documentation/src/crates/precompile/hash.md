## SHA256 and RIPEMD160

REVM includes precompiled contracts for `SHA256` and `RIPEMD160`, cryptographic hashing functions integral for data integrity and security. The addresses for these precompiled contracts are `0x0000000000000000000000000000000000000002` for `SHA256` and `0x0000000000000000000000000000000000000003` for `RIPEMD160`.

Each function (`sha256_run` and `ripemd160_run`) accepts two arguments, the input data to be hashed and the gas_limit representing the maximum amount of computational work permissible for the function. They both calculate the gas cost of the operation based on the input data length. If the computed cost surpasses the `gas_limit`, an `Error::OutOfGas` is triggered.

The `sha256_run` function, corresponding to the `SHA256` precompiled contract, computes the `SHA256` hash of the input data. The `ripemd160_run` function computes the `RIPEMD160` hash of the input and pads it to match Ethereum's 256-bit word size. These precompiled contracts offer a computationally efficient way for Ethereum contracts to perform necessary cryptographic operations.
