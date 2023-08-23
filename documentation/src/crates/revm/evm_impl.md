# EVM Implementation

This module implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

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

## Host Implementation

The `Host` trait provides an interface that allows the EVM to interact with the external world. It contains methods to access environmental information, manipulate account balances, and interact with contract code and storage.

The `EVMImpl` struct implements this `Host` trait.

- `step` & `step_end`

    These methods are used to control the interpreter's execution. They move the interpreter forward one step, allowing the user to inspect the state of the interpreter after each individual operation.
    These control the execution of the interpreter, allowing step-by-step execution and inspection.

- `env`

    This method returns a mutable reference to the environment information that the EVM uses for its execution. The `Env` struct contains details about the current block, such as the timestamp, block number, difficulty, and gas limit.

- `block_hash`

    This method retrieves the hash of a block given its number. It's typically used within smart contracts for actions like random number generation.

- `load_account`

    This method loads the account associated with a given address and returns information about the account's existence and if it's a contract.

- `balance`

    This method retrieves the balance of an Ethereum account given its address. It returns a tuple containing the balance and a boolean indicating whether the account was "cold" (accessed for the first time in the current transaction).

-  `code`

    This method retrieves the bytecode of a given address. It returns a tuple containing the bytecode and a boolean indicating whether the account was "cold".

-  `code_hash`

    This method retrieves the code_hash at a given address. It returns a tuple containing the hash and a boolean indicating whether the account was "cold".

- `sload` & `sstore`

    These methods interact with the contract storage. The `sload` method retrieves a value from contract storage, while `sstore` sets a value in contract storage.

- `tload` & `tstore`

    As defined in [EIP1153](https://eips.ethereum.org/EIPS/eip-1153), for transiant storage reads and writes. 

- `log`

    This method is used to create log entries, which are a way for contracts to produce output that external observers (like dapps or the frontend of a blockchain explorer) can listen for and react to.

-  `selfdestruct`

    The selfdestruct method attempts to terminate the specified address, transferring its remaining balance to a given target address. If the INSPECT constant is true, the self-destruction event is observed or logged via an inspector. The method returns an Option<SelfDestructResult>, encapsulating the outcome of the operation: Some(SelfDestructResult) on success and None if an error occurs, with the error being stored internally for later reference.

- `create`

    The create method initiates the creation of a contract with the provided CreateInputs. If the INSPECT constant is true, the creation process is observed or logged using an inspector, both at the start and end of the creation. The method returns a tuple consisting of the operation's result (InstructionResult), the optional address (Option<B160>) of the newly created contract, the amount of gas consumed (Gas), and the output data (Bytes). If the inspector intervenes and determines the instruction shouldn't continue, an early return occurs with the observed outcomes.

- `call`

    The call method manages a contract invocation using the provided CallInputs. If the INSPECT constant is active, the call event is observed or logged via an inspector before execution. The method yields a tuple representing the outcome of the call: the result status (InstructionResult), the consumed gas (Gas), and the output data (Bytes). If the inspector suggests early termination, the method returns immediately with the observed results. Otherwise, the main call execution is processed, and the outcomes, either raw or observed, are returned accordingly.