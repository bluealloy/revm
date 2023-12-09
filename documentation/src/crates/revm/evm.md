# EVM

`Evm` is the primary structure that implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

## What is inside

It is consisting of two main parts the `Context` and the `Handler`. `Context` represent the state that is needed for execution and `Handler` contains list of functions that act as a logic.

`Context` is additionally split between `EvmContext` and `External` context. `EvmContext` is internal and contains `Database`, `Environment`, `JournaledState` and `Precompiles`. And `External` context is fully generic without any trait restrains and its purpose is to allow custom handlers to save state in runtime or allows hooks to be added (For example external contexts can be a Inspector), more on its usage can be seen in [`EvmBuilder`](./builder.md).

`Evm` implements the [`Host`](./../interpreter/host.md) trait, which defines an interface for the interaction of the EVM Interpreter with its environment (or "host"), encompassing essential operations such as account and storage access, creating logs, and invoking sub calls and selfdestruct.

Data structures of block and transaction can be found inside `Environment`. And more information on journaled state can be found in [`JournaledState`](../revm_journaled_state/journaled_state.md) documentation.

## Runtime

Runtime consist of list of functions from `Handler` that are called in predefined order. They are grouped by functionality on `Verification`, `Main`, `Frame` and `Instruction` functions. Verification function are related to the preverification of set `Environment` data. Main function is the main logic of the transaction execution. Frame function handles call and creates and sub calls. And Instruction functions are instruction table that executes opcodes.

`Evm` execution runs **two** loops. First loop is call loop that everything starts with, it creates call frames, handles subcalls and its return outputs and call Interpreter loop to execute bytecode instructions it is handled by `FrameHandler`. Second loop is `Interpreter` loop that loops over bytecode opcodes and executes instruction from `InstructionTable`.

First loop, the call loop, implements stack of `Frames` and it is responsible for handling sub calls and its return outputs. At the start Evm creates first `Frame` that contains `Interpreter` and starts the loop. `Interpreter` returns the `InterpreterAction` and action can be a `Return` of a call this means this interpreter finished its run or `SubCall`/`SubCreate` that means that new `Frame` needs to be created and pushed to the stack. When `Interpreter` returns `Return` action `Frame` is popped from the stack and its return value is pushed to the parent `Frame` stack. When `Interpreter` returns `SubCall`/`SubCreate` action new `Frame` is created and pushed to the stack and the loop continues. When the stack is empty the loop finishes.

Second loop is `Interpreter` loop that loops over bytecode opcodes and executes instruction. It is called from the call loop and it is responsible for executing bytecode instructions. It is implemented in [`Interpreter`](../revm_interpreter/interpreter.md) crate.

To dive deeper into the `Evm` logic  check [`Handler`](../handler.md) documentation.

# Functionalities

`Evm` is build with a `EvmBuilder` that allows setting of `Database`, `External` context and `Handler`. Builder is created with `Evm::builder()` function. For more information on building check [`EvmBuilder`](./builder.md) documentation.

After building `Evm` it can be used to execute transactions. There are three functions that can be used to execute transactions:
* preverify - that only preverifies transaction information.
* transact preverified - is next step after preverification that executes transaction.
* transact - it calls both preverifies and it executes transaction.

If we want to modify `Evm` for example change the specification ID we can use `.modify()` function that would give us the `EvmBuilder` and we can set new specification that would build new `Evm` from it, as setting new specification would need to reset the `Handler` functions.

More on the builder can be found in [`EvmBuilder`](./builder.md) documentation. 