# Interpreter

The `interpreter` crate is concerned with the execution of the EVM opcodes and serves as the event loop to step through the opcodes. The interpreter is concerned with attributes like gas, contracts, memory, stack, and returning execution results. It's structured as follows:

Modules:

- [gas](./interpreter/gas.md): This module deals with handling the gas mechanics in the EVM, such as calculating gas costs for operations.
- [host](./interpreter/host.md): This module defines the evm context `Host` trait.
- [inner_models](./interpreter/inner_models.md): This module contains the inner data structures used in the EVM implementation.
- [instruction_result](./interpreter/instruction_result.md): This module contains definitions related to the result of instruction execution.
- [instructions](./interpreter/instructions.md): This module includes the definitions of the EVM opcodes (instructions).


External Crates:

- alloc: The alloc crate is used to provide the ability to allocate memory on the heap. It's a part of Rust's standard library that can be used in environments without a full host OS.
- core: The core crate is the dependency-free foundation of the Rust standard library. It includes fundamental types, macros, and traits.

Constants:

- `USE_GAS`: This constant determines whether gas measurement should be used. It's set to false if the `no_gas_measuring` feature is enabled.

Re-exports:
- Several types and functions are re-exported for easier access by users of this library, such as `Gas`, `Host`, `InstructionResult`, `OpCode`, `Interpreter`, `Memory`, `Stack`, and others. This allows users to import these items directly from the library root instead of from their individual modules.
- revm_primitives: This crate is re-exported, providing primitive types or functionality used in the EVM implementation.
