Crates version and their description:
* ![revm](https://img.shields.io/crates/v/revm?height=50?label=revm) main crate, it reexports all other crates. 
* ![revm-primitives](https://img.shields.io/crates/v/revm-primitives?label=revm-primitives) contains constants and primitives types that revm uses (alloy-primitives)
* ![revm-interpreter](https://img.shields.io/crates/v/revm-interpreter?label=revm-interpreter) biggest crate in the project, it contains all instructions
* ![revm-precompile](https://img.shields.io/crates/v/revm-precompile?label=revm-precompile) Precompiles defined by ethereum
* ![revm-database-interface](https://img.shields.io/crates/v/revm-database-interface?label=revm-database-interface) Interfaces for database implementation, database is used to fetch runtime state data (accounts, storages and block hash) 
* ![revm-database](https://img.shields.io/crates/v/revm-database?label=revm-database) A few structures that implement database interface
* ![revm-bytecode](https://img.shields.io/crates/v/revm-bytecode?label=revm-bytecode) Bytecode legacy analysis and EOF validation. Create contains opcode tables. 
* ![revm-state](https://img.shields.io/crates/v/revm-state?label=revm-state) Small crate with accounts and storage types.
* ![revm-context-interface](https://img.shields.io/crates/v/revm-context-interface?label=revm-context-interface) traits for Block/Transaction/Cfg/Journal.
* ![revm-context](https://img.shields.io/crates/v/revm-context?label=revm-context) default implementation for traits from context interface. 
* ![revm-handler](https://img.shields.io/crates/v/revm-handler?label=revm-handler) Contains logic around validation, pre and post execution and handling of call frames.  
* ![revm-inspector](https://img.shields.io/crates/v/revm-inspector?label=revm-inspector) Adds support for inspector and implements EIP-3155 tracer.
* ![op-revm](https://img.shields.io/crates/v/op-revm?label=op-revm) Uses revm to create Optimism EVM.
* ![revm-statetest-types](https://img.shields.io/crates/v/revm-statetest-types?label=revm-statetest-types) helpful structs for state test usage.
