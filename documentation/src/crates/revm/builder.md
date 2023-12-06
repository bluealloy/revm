
# Evm Builder

It creates the EVM and applies different handler, and allows setting external context and custom logic.

EVM inside revm consist of the few parts `Context` and `Handler`.

Context represent the state that is needed for execution and handler contains list of functions that act as a logic.

`Context` is additionally split between `EvmContext` and `External` context. `External` context is fully generic without any trait restrains and its purpose is to allow custom handlers to have access to internal state (For example external contexts can be a Inspector). While `EvmContext` is internal and contains `Database`, Environment, JournaledState and Precompiles.

Handler is.. it is not generic but has a specification identification variable. It contains list of function that are wrapped around `Arc`. Functions (aka handles) are grouped by functionality on:
* preverification functions. Are related to the preverification of set Environment data. 
* main function: Deducs caller balance, loads warm accounts/storages, and handler beneficiary rewards. 
* Frame function: Handles call and creates and sub calls.
* Instruction functions: Is instruction table that executes opcodes. 

Builder ties dependencies between generic Database, External context and Spec and allows overriding handlers. As there is a dependency between them setting Database will reset External and Handle field while setting External field would reset Handler. Note that Database will never be reset.


Simple example of using `EvmBuilder` is

```

Evm::build().with_empty_db().with_empty_external()
```