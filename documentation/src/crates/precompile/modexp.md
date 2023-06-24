# Modular Exponentiation

REVM also implements two versions of a precompiled contract (Modular Exponential operation), each corresponding to different Ethereum hard forks: Byzantium and Berlin. The contract addresses are `0x0000000000000000000000000000000000000005` for both versions, as they replaced each other in subsequent network upgrades. This operation is used for cryptographic computations and is a crucial part of Ethereum's toolkit.

The byzantium_run and berlin_run functions each run the modular exponential operation using the `run_inner` function, but each uses a different gas calculation method: `byzantium_gas_calc` for Byzantium and `berlin_gas_calc` for Berlin. The gas calculation method used is chosen based on the Ethereum network's current version. The `run_inner` function is a core function that reads the inputs and performs the modular exponential operation. If the calculated gas cost is higher than the gas limit, an error `Error::OutOfGas` is returned. If all computations are successful, the function returns the result of the operation and the gas cost.

The calculate_iteration_count function calculates the number of iterations required to compute the operation, based on the length and value of the exponent. The `read_u64_with_overflow` macro reads input data and checks for potential overflows.

The byzantium_gas_calc function calculates the gas cost for the modular exponential operation as defined in the Byzantium version of the Ethereum protocol. The `berlin_gas_calc` function calculates the gas cost according to the Berlin version, as defined in [EIP-2565](https://eips.ethereum.org/EIPS/eip-2565). These two versions have different formulas to calculate the gas cost of the operation, reflecting the evolution of the Ethereum network.
