
# Evm Builder

It build or modifies the EVM and applies different handler, and allows setting external context and registering handler custom logic.

`Evm` inside revm consist of the few parts `Context` and `Handler`. `Context` is additionally split between `EvmContext` that contains generic `Database` and `External` context that is generic without restrain. Read here for more information on [`Evm`](./evm.md) internals.

Builder ties dependencies between generic `Database`, `External` context and `Spec` and allows handle registers to be added that implement logic on those generics. As they are interconected setting `Database` or `ExternalContext` wound reset handle registers, and builder stages are introduced to mitigate those misuses.

Simple example of using `EvmBuilder`:

```
let evm = Evm::builder().with_empty_db().without_external_context().build();
```

## Builder Stages

List of the builder stages:

* SetGenericStage: Initial stage that allows setting the database and external context.
* HandlerStage: Allows pushing handler registers, requiring both the database and external context to be set.

Stages has functions that are specific to that stage and there are functions that are group common to both stages.

A few functions that are specific for stage are just a renames to make user more aware of what underlying function does. For example in `SettingDbStage` we have `with_db` function while in `HandlerStage` we have `reset_handler_with_db`, both of them set the database but the latter also resets the handler.

There is naming convention for the functions that are common to both stages:
* `with_*` are mostly found in `SetGenericStage` and are used to set the generics.
* `append_*` are used to push handler registers.
* `modify_*` are used to modify the database, external context or Env, those are common to both stages.
* `reset_handler_with_*` is used if we want to change some of the generic types this will reset the handler registers.
* `build` is to create the Evm.
* `with_spec_id` is used to set the spec id, this is common to both stages and will create new handler and reapply all the handler registers.
* `clear_*` allows setting default values for Environment.


# Creating and modification of Evm

Evm implements function that allows using of EvmBuilder without even knowing. Most obvious one is `Evm::builder()` that would create a new builder with default values.

Additionally function that is very important is `evm.modify()` that allows modifying the Evm. It will return the builder and will allow to modify the Evm. This function is used in the `Evm::execute` function to modify the Evm before execution.


# Examples

