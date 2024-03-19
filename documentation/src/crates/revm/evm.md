# EVM

`Evm` is the primary structure that implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

## What is inside

It is consisting of two main parts the `Context` and the `Handler`. `Context` represent the state that is needed for execution and `Handler` contains list of functions that act as a logic.

`Context` is additionally split between `EvmContext` and `External` context. `EvmContext` is internal and contains `Database`, `Environment`, `JournaledState` and `Precompiles`. And `External` context is fully generic without any trait restrains and its purpose is to allow custom handlers to save state in runtime or allows hooks to be added (For example external contexts can be a Inspector), more on its usage can be seen in [`EvmBuilder`](./builder.md).

`Evm` implements the [`Host`](./../interpreter/host.md) trait, which defines an interface for the interaction of the EVM Interpreter with its environment (or "host"), encompassing essential operations such as account and storage access, creating logs, and invoking sub calls and selfdestruct.

Data structures of block and transaction can be found inside `Environment`. And more information on journaled state can be found in [`JournaledState`](../revm/journaled_state.md) documentation.

## Runtime

Runtime consists of list of functions from `Handler` that are called in predefined order.
They are grouped by functionality on `Verification`, `PreExecution`, `Execution`, `PostExecution` and `Instruction` functions.
Verification function are related to the pre-verification of set `Environment` data.
Pre-/Post-execution functions deduct and reward caller beneficiary.
And `Execution` functions handle initial call and creates and sub calls.
`Instruction` functions are part of the instruction table that is used inside `Interpreter` to execute opcodes.

The `Evm` execution runs **two** loops:


### Call loop
The first loop is call loop that everything starts with, it creates call frames, handles subcalls, it returns outputs and calls `Interpreter` loop to execute bytecode instructions.
It is handled by `ExecutionHandler`.

The first loop implements a stack of `Frames`.
It is responsible for handling sub calls and its return outputs.
At the start, `Evm` creates `Frame` containing `Interpreter` and starts the loop.

The `Interpreter` returns the `InterpreterAction` which can be:
- `Return`: This interpreter finished its run.
  `Frame` is popped from the stack and its return value is pushed to the parent `Frame` stack.
- `SubCall`/`SubCreate`: A new `Frame` needs to be created and pushed to the stack.
  A new `Frame` is created and pushed to the stack and the loop continues.
  When the stack is empty, the loop finishes.

### Interpreter loop
The second loop is the `Interpreter` loop which is called by the call loop and loops over bytecode opcodes and executes instructions based on the `InstructionTable`.
It is implemented in the [`Interpreter`](../interpreter.md) crate.

To dive deeper into the `Evm` logic  check [`Handler`](./handler.md) documentation.

# Functionalities

The function of `Evm` is to start execution, but setting up what it is going to execute is done by `EvmBuilder`.
The main functions of the builder are:
* `preverify` - that only pre-verifies transaction information.
* `transact preverified` - is next step after pre-verification that executes transactions.
* `transact` - it calls both preverifies and executes transactions.
* `builder` and `modify` functions - allow building or modifying the `Evm`, more on this can be found in [`EvmBuilder`](./builder.md) documentation. `builder` is the main way of creating `Evm` and `modify` allows you to modify parts of it without dissolving `Evm`.
* `into_context` - is used when we want to get the `Context` from `Evm`.
