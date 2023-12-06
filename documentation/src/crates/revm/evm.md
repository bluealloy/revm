# EVM

`Evm` is the primary structure that implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

It is consisting of two main parts `Context` and `Handler`. `Context` represent the state that is needed for execution and `Handler` contains list of functions that act as a logic.

`Context` is additionally split between `EvmContext` and `External` context. `EvmContext` is internal and contains `Database`, `Environment`, `JournaledState` and `Precompiles`. And `External` context is fully generic without any trait restrains and its purpose is to allow custom handlers to have access to internal state (For example external contexts can be a Inspector), more on its usage can be seen in [`EvmBuilder`](./builder.md)


`Evm` implements the [`Host`](./host.md) trait, which defines an interface for the interaction of the EVM interpreter with its environment (or "host"), encompassing essential operations such as account and storage access, creating logs, and invoking transactions.

revm in runtime runs two main loops. First loop is Interpreter loop that loops over bytecode opcodes and executes instruction. Second loop is call loop that everything starts with, it creates call frames, handles subcalls and its return outputs and call Interpreter loop to execute bytecode instructions.





## Methods

- `run_interpreter`

    This method is responsible for setting up and running the interpreter for a specific contract.

    - `contract`: A `Contract` instance that the interpreter will execute.
    - `gas_limit`: A `u64` that determines the maximum amount of gas that the execution can consume.
    - `is_static`: A boolean flag indicating if the execution is static. Static executions cannot modify the state.

    The method returns a tuple containing the result of the execution and the interpreter instance. The result is an `InstructionResult` enumeration value that indicates if the execution was successful or if an error occurred.

    This creates a contract with a specific bytecode and a gas price, then runs the interpreter on this contract with a specified gas limit. The is_static flag is set to false which means the execution can modify the state. The stack machine implements the following instructions:

- `call_precompile`

    This method handles the execution of precompiled contracts. These are a special set of contracts that are part of the Ethereum protocol and implemented in native code for efficiency.

    - `gas`: A `Gas` instance representing the amount of gas available for execution.
    - `contract`: The address of the precompiled contract in the form of a `B160` instance.
    - `input_data`: The input data for the contract as a `Bytes` instance.

    The method returns a tuple containing the result of the contract execution, the remaining gas, and any output data as a `Bytes` instance. 

- `call_inner`

    This method performs a contract call within the EVM.

    - `inputs`: A mutable reference to a `CallInputs` instance, which contains all the necessary information for the contract call.

    The method returns a tuple containing the result of the call (as an `InstructionResult`), the remaining gas (as a `Gas` instance), and any output data from the call (as a `Bytes` instance).
