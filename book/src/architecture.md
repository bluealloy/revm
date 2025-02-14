# Architecture

List of crates can be found in [Crates](./architecture/crates.md) section of the book.

REVM as any EVM implement a list of [EIP's (Ethereum Improvement Protocol)](https://github.com/ethereum/EIPs) changes over time. Those changes are shipped in the form of hardforks. List of hardforks can be found here [Ethereum Hardforks]() and repository that contains all EIPs can be found here [EIPs](https://eips.ethereum.org/).

### Main components/traits:

Revm consist of few traits that implement functionality of the EVM. The main traits are:
* **EvmTrait**: This trait allows as to access main EVM fields and to run interpreter. It defines **Context**, **Precompiles**, **Instructions**. Docs
* **ContextTrait**: is gained from EvmTrait and consist of types needed for execution. It defines environment such as block and transaction, database for runtime fetching of accounts and storage, journal for status changes and revert handling and few more fields. Docs
* **EthHandler**: is a trait that by default implements Ethereum logic, it takes EvmTrait as a input. Entry point is a `run` function. Docs
* **Frame**: is a associate type of EthHandler and contains runtime data of the call and logic of executing the call, default impl is a type is EthFrame. Docs

Inspection for tracing is extensing above traits with:
* **InspectorEvmTrait** is derived from EvmTrait and allows running Evm in Inspection mode. It contains **Inspector** associate type. Docs
* **EthInspectorHandler** is derived from EthHandler and allows running Evm in Inspection mode. Entry point is `inspect_run` function and it calls a alternative functions for execution loop that includes inspector calls. Docs
* **Inspector** is a a user oriented trait that is used for inspection of the EVM. It is used for tracing. It is part of Evm struct and it is called from EthInspectorHandler and InspectorEvmTrait. Docs


### Simplified code

```rust
pub trait EvmTrait {
    type Context: ContextTrait;
    ...
    fn execute_interpreter(..);
}

pub trait EthHandler {
    type Evm: EvmTrait;
    type Frame: Frame;
    ...
    fn run(evm);
}
```

### flow of execution
Execution flow can be found here (TODO Move to codebase to EthHandler trait):
* It starts with creation of new EVM instance
  * Building of the Context
  * Building of the EVM. Inspector/Precompiles are created.
  * Adding of the Inspector if needed.
* transact/inspect. Both inspection and transaction have same flow where the only difference is that inspection includes calls to the inspector.
  * validation of transaction and doing balance gas limit check.
  * pre execution loads all warm accounts and deducts the caller.
  * Execution :
    * Creates first frame with Interpreter or executes precompile.
    * run the frame loop:
      * Calls Evm to exec interpreter with the frame. Interpreter loops is called here
      * Output of Interpreter loop is NextAction that can be a Return of Call.
      * If Return, then the frame is popped and the return value is pushed to the parent frame. If it is new call, then a new frame is created and pushed to the call stack.
      * If call stack is empty the execution loop is done.
    * handles the result of execution.s
  * Post execution deals with halt and revert handling redistrubution of rewards and reimbursment of unspend gas.