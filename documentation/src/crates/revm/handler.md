# Handler

Is logic part of the Evm, it contains the Specification ID, and list of functions that do the logic and list of registers that can change the behaviour of the Handler.

Functions can be grouped in four categories and are marked in that way in the code:
* Validation functions: `ValidateHandler`
* Main functions: `MainHandler`
* Frame functions: `FrameHandler`
* Internal functions: `InstractionTable`

### ValidationHandler

Consist of functions that are used to validate transaction and block data. They are called before the execution of the transaction and they are used to check if the data (`Environment`) is valid. They are called in the following order:

* validate_env
    
    That verifies if all data is set in `Environment` and if they are valid, for example if `gas_limit` is smaller than block `gas_limit`.

* validate_initial_tx_gas
    
    It calculated initial gas needed for transaction to be executed and checks if it is less them the transaction gas_limit. Note that this does not touch the `Database` or state

* validate_tx_against_state
    
    It loads the caller account and checks those information. Among them the nonce, if there is enough balance to pay for max gas spent and balance transferred. 

### MainHandler

Consist of functions that are used to execute transaction. They are called in the following order:

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


### FrameHandler

Consist of the function that handles the call stack and the first loop. They are called in the following order:

TODO

### InstructionTable

Is a list of 256 function pointers that are used to execute instructions. They have two types, first is simple function that is faster and second is BoxedInstraction that has a small performance penalty but allows to capture the data. Look at the Interpreter documentation for more information.

### Handle Registers

Simple function that is used to modify handler functions. Amazing thing about them is that they can be done over generic external type. For example this allows to have a register over trait that would allow to add hooks to the any type that implements the trait, that trait can be a GetInspector trait so anyone that implement it would be able to register inspector related functions. It is used inside the `EvmBuilder` to change behaviour of the default mainnet Handler.

Order of the registers is important as they are called in the order they are registered. And it matters if register overrides the previous handle or just wraps it, overriding handle can disrupt the logic of previous registered handles.