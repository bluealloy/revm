# EVM Implementation

This module implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

## `run_interpreter`

This function is responsible for setting up and running the interpreter for a specific contract.

- `contract`: A `Contract` instance that the interpreter will execute.
- `gas_limit`: A `u64` that determines the maximum amount of gas that the execution can consume.
- `is_static`: A boolean flag indicating if the execution is static. Static executions cannot modify the state.

The function returns a tuple containing the result of the execution and the interpreter instance. The result is an `InstructionResult` enumeration value that indicates if the execution was successful or if an error occurred.

For example

```rust
let contract = Contract::new(bytecode, U256::from(10));
let gas_limit = 1000000_u64;
let is_static = false;
let (exit_reason, interpreter) = evm.run_interpreter(contract, gas_limit, is_static);
```

This creates a contract with a specific bytecode and a gas price, then runs the interpreter on this contract with a specified gas limit. The is_static flag is set to false which means the execution can modify the state.

## `call_precompile`

This function handles the execution of precompiled contracts. These are a special set of contracts that are part of the Ethereum protocol and implemented in native code for efficiency.

- `gas`: A `Gas` instance representing the amount of gas available for execution.
- `contract`: The address of the precompiled contract in the form of a `B160` instance.
- `input_data`: The input data for the contract as a `Bytes` instance.

The function returns a tuple containing the result of the contract execution, the remaining gas, and any output data as a `Bytes` instance.

For example

```rust
let gas = Gas::new(1000000);
let contract = B160::zero();
let input_data = Bytes::from("input data");
let (exit_reason, gas, output) = evm.call_precompile(gas, contract, input_data);
```

This executes a precompiled contract with a specified gas limit and input data.

## `call_inner`

This function performs a contract call within the EVM.

- `inputs`: A mutable reference to a `CallInputs` instance, which contains all the necessary information for the contract call.

The function returns a tuple containing the result of the call (as an `InstructionResult`), the remaining gas (as a `Gas` instance), and any output data from the call (as a `Bytes` instance).

for example

```rust
    let mut inputs = CallInputs {
    gas_limit: 1000000,
    // other parameters...
};
let (exit_reason, gas, output) = evm.call_inner(&mut inputs);
```

## Host Implementation

The `Host` trait provides an interface that allows the EVM to interact with the external world. It contains methods to access environmental information, manipulate account balances, and interact with contract code and storage.

The `EVMImpl` struct implements this `Host` trait. The methods provided by this trait interface are as follows:

## `step` & `step_end`

These methods are used to control the interpreter's execution. They move the interpreter forward one step, allowing the user to inspect the state of the interpreter after each individual operation.

for example

```rust
let mut interpreter = Interpreter::new(contract, gas_limit, is_static);
let result = evm.step(&mut interpreter);
let result_end = evm.step_end(&mut interpreter, result);
```

These control the execution of the interpreter, allowing step-by-step execution and inspection.

## `env`

This method returns a mutable reference to the environment information that the EVM uses for its execution. The `Env` struct contains details about the current block, such as the timestamp, block number, difficulty, and gas limit.

## `block_hash`

This method retrieves the hash of a block given its number. It's typically used within smart contracts for actions like random number generation.

## `load_account`

This method loads the account associated with a given address and returns information about the account's existence and if it's a contract.

## `balance`

This method retrieves the balance of an Ethereum account given its address. It returns a tuple containing the balance and a boolean indicating whether the account was "cold" (accessed for the first time in the current transaction).

## `code`

This method retrieves the bytecode of a contract given its address. It returns a tuple containing the bytecode and a boolean indicating whether the account was "cold".

## `sload` & `sstore`

These methods interact with the contract storage. The `sload` method retrieves a value from contract storage, while `sstore` sets a value in contract storage.

## `log`

This method is used to create log entries, which are a way for contracts to produce output that external observers (like dapps or the frontend of a blockchain explorer) can listen for and react to.

## `selfdestruct`
