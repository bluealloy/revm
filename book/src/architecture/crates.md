
Revm project is split in multiple crates, each crate is responsible for different part of the project. The main crates are:

There is one binary crate `revme`. It is used for running Ethereum state tests json.

### Library crates
* revm:
* primitives:
* interpreter:
* precompile:
* database:
* database/interface:
* bytecode:
* state:
* specification:
* context:
* context/interface:
* handler:

### variants
* optimism
* inspector

### utility
* statetest-types:

### examples
* block_traces:
* cheatcode_inspector:
* contract_deployment:
* database_components:
* uniswap_get_reserves:
* uniswap_v2_usdc_swap:
* erc20_gas:


# Dependency of library crates can be seen here

TODO Add dependency graph here