
# Evm Builder

It build or modifies the EVM and applies different handler, and allows setting external context and registering handler custom logic.

`Evm` inside revm consist of the few parts `Context` and `Handler`. `Context` is additionally split between `EvmContext` that contains generic `Database` and `External` context that is generic without restrain. Read here for more information on [`Evm`](./evm.md) internals.

Builder ties dependencies between generic `Database`, `External` context and `Spec` and allows handle registers to be added that implement logic on those generics. As they are interconected setting `Database` or `ExternalContext` wound reset handle registers, and builder stages are introduced to mitigate those misuses.

Simple example of using `EvmBuilder`:

```rust,ignore
  use crate::evm::Evm;

  // build Evm with default values.
  let mut evm = Evm::builder().build();
  let output = evm.transact();
```

## Builder Stages

There are two builder stages that are used to mitigate potential misuse of the builder:

* `SetGenericStage`: Initial stage that allows setting the database and external context.
* `HandlerStage`: Allows setting the handler registers but is explicit about setting new generic type as it will void the handler registers. 

Functions from one stage are just a renamed function from other stage, it is made so that user is more aware of what underlying function does. Example of this, in `SettingDbStage` we have `with_db` function while in `HandlerStage` we have `reset_handler_with_db`, both of them set the database but the latter also resets the handler. There are multiple functions that are common to both stages as in `build`.

There is naming convention for the functions that can be found in builder:
* In both stages we have:
  * `build` is to create the Evm.
  * `spec_id` is used to set the spec id, this is common to both stages and will create new mainnet handler and reapply all the handler registers.
  * `modify_*` are used to modify the database, external context or Env.
  * `clear_*` allows setting default values for Environment.
  * `append_handler_register_*` are used to push handler registers. This will transition the builder to the `HandlerStage`.
* `SetGenericStage` have:
  * `with_*` are found in `SetGenericStage` and are used to set the generics.
* `HandlerStage` have:
  * `reset_handler_with_*` is used if we want to change some of the generic types this will reset the handler registers. This will transition the builder to the `SetGenericStage`.

# Creating and modification of Evm

Evm implements function that allows using of EvmBuilder without even knowing that it exist. Most obvious one is `Evm::builder()` that would create a new builder with default values.

Additionally function that is very important is `evm.modify()` that allows modifying the Evm. It will return the builder and will allow user to modify the Evm.

# Examples

Example of using builder to create Evm with inspector:
```rust,ignore
  use crate::{
      db::EmptyDB, Context, EvmContext, inspector::inspector_handle_register, inspectors::NoOpInspector, Evm,
  };

  // Create the evm.
  let evm = Evm::builder()
      .with_db(EmptyDB::default())
      .with_external_context(NoOpInspector)
      // Register will modify Handler and call NoOpInspector.
      .append_handler_register(inspector_handle_register)
      // .with_db(..) would not compile as we already locked the builder generics,
      // alternative fn is reset_handler_with_db(..)
      .build();
  
  // Execute the evm.
  let output = evm.transact();
  
  // Extract evm context.
  let Context {
      external,
      evm: EvmContext { db, .. },
  } = evm.into_context();
```

Example of changing spec id and Environment of already build evm.
```rust,ignore
  use crate::{Evm,SpecId::BERLIN};

  // Create default evm.
  let evm = Evm::builder().build();

  // Modify evm spec.
  let evm = evm.modify().spec_id(BERLIN).build();

  // Shortcut for above.
  let mut evm = evm.modify_spec_id(BERLIN);

  // Execute the evm.
  let output1 = evm.transact();

  // Example of modifying the tx env.
  let mut evm = evm.modify().modify_tx_env(|env| env.gas_price = 0.into()).build();

  // Execute the evm with modified tx env.
  let output2 = evm.transact();
```

## Appending handler registers

Handler registers are simple function that allow modifying the `Handler` logic by replacing 
the handler functions. They are used to add custom logic to the evm execution but as they are free to modify the `Handler` in any form they want there can be conflicts if handlers that override the same function are added.

Most common use case for adding new logic to `Handler` is `Inspector` that is used to inspect the execution of the evm. Example of this can be found in [`Inspector`](./inspector.md) documentation.