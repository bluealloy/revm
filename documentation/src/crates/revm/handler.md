# Handler

Is logic part of the Evm, it contains the Specification ID, list of functions that do the logic and list of registers that can change behavior of the Handler when `Handler` is build.

Functions can be grouped in five categories and are marked in that way in the code:
* Validation functions: `ValidateHandler`
* Pre execution functions: `PreExecutionHandler`
* Execution functions: `ExecutionHandler`
* Post execution functions: `PostExecutionHandler`
* Instruction table: `InstructionTable`

### Handle Registers

Simple function that is used to modify handler functions. Amazing thing about them is that they can be done over generic external type. For example this allows to have a register over trait that would allow to add hooks to the any type that implements the trait, that trait can be a GetInspector trait so anyone that implement it would be able to register inspector related functions. It is used inside the `EvmBuilder` to change behavior of the default mainnet Handler.

Handle registers are set in `EvmBuilder`.

Order of the registers is important as they are called in the order they are registered. And it matters if register overrides the previous handle or just wraps it, overriding handle can disrupt the logic of previous registered handles.

Registers are very powerful as they allow modification of any part of the Evm and with additional of the `External` context it becomes a powerful combo. Simple example would be to register new precompiles for the Evm.

### ValidationHandler

Consist of functions that are used to validate transaction and block data. They are called before the execution of the transaction and they are used to check if the data (`Environment`) is valid. They are called in the following order:

* validate_env
    
    That verifies if all data is set in `Environment` and if they are valid, for example if `gas_limit` is smaller than block `gas_limit`.

* validate_initial_tx_gas
    
    It calculated initial gas needed for transaction to be executed and checks if it is less them the transaction gas_limit. Note that this does not touch the `Database` or state

* validate_tx_against_state
    
    It loads the caller account and checks those information. Among them the nonce, if there is enough balance to pay for max gas spent and balance transferred. 

### PreExecutionHandler

Consist of functions that are called before execution. They are called in the following order:

* load
   
    Loads access list and beneficiary from `Database`. Cold load is done here.

* load precompiles
   
    Load precompiles.

* deduct_caller:
  
    Deducts values from the caller to the maximum amount of gas that can be spent on the transaction. This loads the caller account from the `Database`.

### ExecutionHandler

Consist of the function that handle the execution of the transaction and the stack of the call frames.

* call
    Create new call frame or return the frame result. Returning of frame result is done if we are calling precompile or the new frame is not created. If FrameReturn is returned then next function that is called would be `insert_call_outcome`. It is called on every frame.

* call_return
    Called after call frame returns from execution. It is used to calculate the gas that is returned from the frame and create the FrameResult that is used to apply the outcome to parent frame in `insert_call_outcome`

* insert_call_outcome
    Inserts the call outcome to the parent frame. It is called on every frame that is created except the first one. for first frame we use `last_frame_return`.

* create
  
    Create new create call frame, create new account and execute bytecode that outputs the code of the new account.

* create_return

    This handler is called after every frame is executed (Expect first), it will calculate the gas that is returned from the frame and apply output to the parent frame.

* insert_create_outcome

    This handler is called after every frame is executed (Expect first), it will calculate the gas that is returned from the frame and apply output to the parent frame.

* last_frame_return
  
    This handler is called after last frame is returned. It is used to calculate the gas that is returned from the first frame and incorporate transaction gas limit (first frame had gas_limit-initial_gas).

### InstructionTable

Is a list of 256 function pointers that are used to execute instructions. They have two types, first is simple function that is faster and second is BoxedInstraction that has a small performance penalty but allows to capture the data. Look at the Interpreter documentation for more information.

### PostExecutionHandler

Is a list of functions that are called after the execution. They are called in the following order:

* reimburse_caller
    
    Reimburse the caller with gas that was not spent during the execution of the transaction.
    Or balance of gas that needs to be refunded.

* reward_beneficiary
    
    At the end of every transaction beneficiary needs to be rewarded with the fee.

* output

    It returns the changes state and the result of the execution.

* end
  
    It will be called always as the last function of the handler.