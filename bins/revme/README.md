# Rust EVM executor or short REVME

This is binary crate that executed evm multiple ways. Currently it is used trun ethereum tests:
* statetest: takes path to folder where ethereum statetest json can be found. It recursively searches for all json files and execute them. This is how i run all https://github.com/ethereum/tests to check if revm is compliant. Example `revme statests test/GenericEvmTest/`