
# Evm Builder

It creates the EVM and applies different handler, and allows setting external context and custom logic.

`Evm` inside revm consist of the few parts `Context` and `Handler`. `Context` is additionally split between `EvmContext` and `External` context. Read here for more information on [`Evm`](./evm.md) internals.

Builder ties dependencies between generic `Database`, `External` context and `Spec` and allows handle registers to be added. As there is a generic dependency between them setting `Database` will reset `External` context and `Handler` field while setting `External` field would reset just a `Handler` as it will become invalid. Note that Database will never be reset.


Simple example of using `EvmBuilder` is

```
Evm::build().with_empty_db().without_external_context()
```

`EvmBuilder` has three stages that will gradually unlock more functionality, first is setting the database, second is setting external context and third is setting/registering the handler registers. Environment can be modified in anytime with `modify_*`' functions that take a closure.

