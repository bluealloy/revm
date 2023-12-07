
# Evm Builder

It creates the EVM and applies different handler, and allows setting external context and custom logic.

`Evm` inside revm consist of the few parts `Context` and `Handler`. `Context` is additionally split between `EvmContext` and `External` context. Read here for more information [`Evm`](./evm.md) internals.

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