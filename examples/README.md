# examples:
* `contract_deployment`: Example of deployment of the contract from solidity compilation and calling deployed contract.
* `my_evm`: Example and tutorial on how to create your custom evm.
* `erc20_gas`: Example of custom EVM that uses ERC20 token to pay for Gas.
* `uniswap_get_reserves`: Example of using alloy to fetch state and sol! to call a function of the contract.
* `uniswap_v2_usdc_swap`: Similar to `uniswap_get_reserves` with more examples of usage.
* `block_traces`: Uses Alloy to fetch blocks transaction and state from provider to execute full block. It uses Eip3155 opcode tracer and saves output to the file.
* `custom_opcodes`: Example of introducing a custom instruction to the mainnet Evm.
* `database_components`: Example of decouples Database in `State` and `BlockHash` and how to use it inside Revm.