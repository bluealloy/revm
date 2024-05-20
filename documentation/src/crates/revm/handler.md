# Handler

This is the logic part of the Evm.
It contains the Specification ID, list of functions that do the logic and list of registers that can change behavior of the Handler when it is build.

Functions can be grouped in five categories and are marked in that way in the code:
* Validation functions: [`ValidationHandler`](https://github.com/bluealloy/revm/blob/main/crates/revm/src/handler/handle_types/validation.rs)
* Pre-execution functions: [`PreExecutionHandler`](https://github.com/bluealloy/revm/blob/main/crates/revm/src/handler/handle_types/pre_execution.rs)
* Execution functions: [`ExecutionHandler`](https://github.com/bluealloy/revm/blob/main/crates/revm/src/handler/handle_types/execution.rs)
* Post-execution functions: [`PostExecutionHandler`](https://github.com/bluealloy/revm/blob/main/crates/revm/src/handler/handle_types/post_execution.rs)
* Instruction table: [`InstructionTable`](https://github.com/bluealloy/revm/blob/main/crates/interpreter/src/opcode.rs)

### Handle Registers

This is a simple function that is used to modify handler functions.
The amazing thing about them is that they can be done over generic external type.
For example, this allows to have a register over trait that allows to add hooks to any type that implements the trait.
That trait can be a `GetInspector` trait, so any implementation is able to register inspector-related functions.
`GetInspector` is implemented on every `Inspector` and it is used inside the `EvmBuilder` to change behavior of the default mainnet Handler.

Handle registers are set in `EvmBuilder`.
The order of the registers is important as they are called in the order they are registered.
It matters if register overrides the previous handle or just wraps it, overriding handle can disrupt the logic of previous registered handles.

Registers are very powerful as they allow modification of any part of the Evm and with additional of the `External` context it becomes a powerful combo.
A simple example is to register new pre-compiles for the Evm.

### ValidationHandler

Consists of functions that are used to validate transaction and block data.
They are called before the execution of the transaction, to check whether the (`Environment`) data is valid.
They are called in the following order:
* `validate_env`:
  Verifies if all data is set in `Environment` and if valid, for example if `gas_limit` is smaller than block `gas_limit`.
* `validate_initial_tx_gas`:
  Calculates initial gas needed for the transaction to be executed and checks if it is less than the transaction gas_limit.
  Note that this does not touch the `Database` or state.
* `validate_tx_against_state`:
  Loads the caller account and checks their information.
  Among them the nonce, if there is enough balance to pay for max gas spent and balance transferred. 

### PreExecutionHandler

Consists of functions that are called before execution.
They are called in the following order:
* `load`:
  Loads access list and beneficiary from `Database`. Cold load is done here.
* `load_precompiles`:
  Retrieves the precompiles for the given spec ID.
  More info: [precompile](../precompile.md). 
* `deduct_caller`:
   Deducts values from the caller to calculate the maximum amount of gas that can be spent on the transaction.
   This loads the caller account from the `Database`.

### ExecutionHandler

Consists of functions that handle the execution of the transaction and the stack of the call frames.

* `call`:
    Called on every frame.
    It creates a new call frame or returns the frame result (the frame result is only returned when calling `precompile`).
    If `FrameReturn` is returned, then the next function that is called is `insert_call_outcome`.

* `call_return`:
    Called after call frame returns from execution.
    It is used to calculate the gas that is returned from the frame and create the `FrameResult` that is used to apply the outcome to parent frame in `insert_call_outcome`.

* `insert_call_outcome`:
    Inserts the call outcome to the parent frame.
    It is called on every frame that is created except the first one.
    For the first frame we use `last_frame_return`.

* `create`:
    Creates new create call frame, create new account and execute bytecode that outputs the code of the new account.

* `create_return`:
    This handler is called after every frame is executed (Expect first).
    It will calculate the gas that is returned from the frame and apply output to the parent frame.

* `insert_create_outcome`:
  Inserts the outcome of a call into the virtual machine's state.

* `last_frame_return`:
    This handler is called after last frame is returned.
    It is used to calculate the gas that is returned from the first frame and incorporate transaction gas limit (the first frame has limit `gas_limit - initial_gas`).

### InstructionTable

This is a list of 256 function pointers that are used to execute instructions.
They have two types, first is simple function that is faster and second is `BoxedInstruction` that has a small performance penalty but allows to capture the data.
Look at the Interpreter documentation for more information.

### PostExecutionHandler

Is a list of functions that are called after the execution. They are called in the following order:

* `reimburse_caller`:
    Reimburse the caller with gas that was not spent during the execution of the transaction.
    Or balance of gas that needs to be refunded.

* `reward_beneficiary`:
    Reward the beneficiary with the fee that was paid for the transaction.

* `output`:
    Returns the state changes and the result of the execution.

* `end`:
    Always called after transaction. End handler will not be called if validation fails.

* `clear`:
    Clears journal state and error and it is always called for the cleanup.