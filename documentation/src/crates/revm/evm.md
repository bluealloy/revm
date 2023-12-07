# EVM

`Evm` is the primary structure that implements the Ethereum Virtual Machine (EVM), a stack-based virtual machine that executes Ethereum smart contracts.

## Structure

It is consisting of two main parts the `Context` and the `Handler`. `Context` represent the state that is needed for execution and `Handler` contains list of functions that act as a logic.

`Context` is additionally split between `EvmContext` and `External` context. `EvmContext` is internal and contains `Database`, `Environment`, `JournaledState` and `Precompiles`. And `External` context is fully generic without any trait restrains and its purpose is to allow custom handlers to save state in runtime or allows hooks to be added (For example external contexts can be a Inspector), more on its usage can be seen in [`EvmBuilder`](./builder.md)

`Evm` implements the [`Host`](./../interpreter/host.md) trait, which defines an interface for the interaction of the EVM Interpreter with its environment (or "host"), encompassing essential operations such as account and storage access, creating logs, and invoking sub calls and selfdestruct.

Data structures of block and transaction can be found inside `Environment`

## Runtime

Runtime consist of two parts first is verification and second is execution.

Handlers logic that does verification are:

* validate_env
    
    That verifies if all data is set in `Environment` and if they are valid, for example if `gas_limit` is smaller than block `gas_limit`.

* validate_initial_tx_gas
    
    It calculated initial gas needed for transaction to be executed and checks if it is less them the transaction gas_limit. Note that this does not touch the `Database` or state

* validate_tx_against_state
    
    It loads the caller account and checks those information. Among them the nonce, if there is enough balance to pay for max gas spent and balance transferred. 

Logic when running transaction consist of few stages that are implemented as a handle calls:
* main_load
   
    Loads access list and beneficiary from `Database`. Cold load is done here.

* deduct_caller:
  
    Deducts values from the caller to the maximum amount of gas that can be spent on the transaction. This loads the caller account from the `Database`.

* create_first_frame and start_the_loop
    
    These two handles main call loop that creates and handles stack of frames. It is responsible for handling subcalls and its return outputs and call Interpreter loop to execute bytecode instructions.

* call_return
  
    Handler that allows processing of the returned output from the call. It calculated refunded gas and final spent gas.

* reimburse_caller
    
    Reimburse the caller with gas that was not spent during the execution of the transaction.
    Or balance of gas that needs to be refunded.

* reward_beneficiary
    
    At the end of every transaction beneficiary needs to be rewarded with the fee.

* main_return

  It returns the changes state and the result of the execution.

`Evm` in runtime runs **two** loops.First loop is call loop that everything starts with, it creates call frames, handles subcalls and its return outputs and call Interpreter loop to execute bytecode instructions. Second loop is Interpreter loop that loops over bytecode opcodes and executes instruction.

First loop, the call loop, implements stack of `Frames` and it is responsible for handling sub calls and its return outputs. At the start Evm creates first `Frame` that contains `Interpreter` and starts the loop. `Interpreter` returns the `InterpreterAction` and action can be a `Return` of a call this means this interpreter finished its run or `SubCall`/`SubCreate` that means that new `Frame` needs to be created and pushed to the stack. When `Interpreter` returns `Return` action `Frame` is popped from the stack and its return value is pushed to the parent `Frame` stack. When `Interpreter` returns `SubCall`/`SubCreate` action new `Frame` is created and pushed to the stack and the loop continues. When the stack is empty the loop finishes.

Second loop is `Interpreter` loop that loops over bytecode opcodes and executes instruction. It is called from the call loop and it is responsible for executing bytecode instructions. It is implemented in [`Interpreter`](../revm_interpreter/interpreter.md) crate.


# Functions

`Evm` is build with a `EvmBuilder` that allows setting of `Database`, `External` context and `Handler`. Builder is created with `Evm::builder()` function. For more information on building check [`EvmBuilder`](./builder.md) documentation.

After building `Evm` it can be used to execute transactions. There are three functions that can be used to execute transactions:
* preverify
* transaction preverified
* transact

If we want to modify `Evm` for example change the specification we can use `.modify()` function that would give us the `EvmBuilder` back and we can set new specification and build new `Evm` from it, as setting new specification would need to reset the `Handler` functions.

