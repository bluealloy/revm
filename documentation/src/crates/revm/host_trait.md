# Host Implementation

The `Host` trait provides an interface that allows the EVM to interact with the external world. It contains methods to access environmental information, manipulate account balances, and interact with contract code and storage.

The [`EVMImpl`](./evm_impl.md) struct implements this `Host` trait.

- `step` & `step_end`

    These methods are used to control the interpreter's execution. They move the interpreter forward one step, allowing the user to inspect the state of the interpreter after each individual operation. These control the execution of the interpreter, allowing step-by-step execution and inspection.

- `env`

    This method returns a mutable reference to the environment information that the EVM uses for its execution. The `Env` struct contains details about the current block, such as the timestamp, block number, difficulty, and gas limit.

- `block_hash`

    This method retrieves the hash of a block given its number. It's typically used within smart contracts for actions like random number generation.

- `load_account`

    This method loads the account associated with a given address and returns information about the account's existence and if it's a contract.

- `balance`

    This method retrieves the balance of an Ethereum account given its address. It returns a tuple containing the balance and a boolean indicating whether the account was "cold" (accessed for the first time in the current transaction).

- `code`

    This method retrieves the bytecode of a given address. It returns a tuple containing the bytecode and a boolean indicating whether the account was "cold".

- `code_hash`

    This method retrieves the code_hash at a given address. It returns a tuple containing the hash and a boolean indicating whether the account was "cold".

- `sload` & `sstore`

    These methods interact with the contract storage. The `sload` method retrieves a value from contract storage, while `sstore` sets a value in contract storage.

- `tload` & `tstore`

    As defined in [EIP1153](https://eips.ethereum.org/EIPS/eip-1153), for transient storage reads and writes. 

- `log`

    This method is used to create log entries, which are a way for contracts to produce output that external observers (like dapps or the frontend of a blockchain explorer) can listen for and react to.

- `selfdestruct`

    The selfdestruct method attempts to terminate the specified address, transferring its remaining balance to a given target address. If the `Inspector` is `Some`, the self-destruction event is observed or logged via an inspector. The method returns an Option<SelfDestructResult>, encapsulating the outcome of the operation: Some(SelfDestructResult) on success and None if an error occurs, with the error being stored internally for later reference.

- `create`

    The create method initiates the creation of a contract with the provided CreateInputs. If the `Inspector` is `Some`, the creation process is observed or logged using an inspector, both at the start and end of the creation. The method returns a tuple consisting of the operation's result (InstructionResult), the optional address (Option<B160>) of the newly created contract, the amount of gas consumed (Gas), and the output data (Bytes). If the inspector intervenes and determines the instruction shouldn't continue, an early return occurs with the observed outcomes.

- `call`

    The call method manages a contract invocation using the provided CallInputs. If the `Inspector` is `Some`, the call event is observed or logged via an inspector before execution. The method yields a tuple representing the outcome of the call: the result status (InstructionResult), the consumed gas (Gas), and the output data (Bytes). If the inspector suggests early termination, the method returns immediately with the observed results. Otherwise, the main call execution is processed, and the outcomes, either raw or observed, are returned accordingly.