
# Evm Builder

The builder creates or modifies the EVM and applies different handlers.
It allows setting external context and registering handler custom logic.

The revm `Evm` consists of `Context` and `Handler`.
`Context` is additionally split between `EvmContext` (contains generic `Database`) and `External` context (generic without restrain).
Read [evm](./evm.md) for more information on the internals.

The `Builder` ties dependencies between generic `Database`, `External` context and `Spec`.
It allows handle registers to be added that implement logic on those generics.
As they are interconnected, setting `Database` or `ExternalContext` resets handle registers, so builder stages are introduced to mitigate those misuses.

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

Functions from one stage are just renamed functions from other stage, it is made so that user is more aware of what underlying function does.
For example, in `SettingDbStage` we have `with_db` function while in `HandlerStage` we have `reset_handler_with_db`, both of them set the database but the latter also resets the handler.
There are multiple functions that are common to both stages such as `build`.

### Builder naming conventions
In both stages we have:
  * `build` creates the Evm.
  * `spec_id` creates new mainnet handler and reapplies all the handler registers.
  * `modify_*` functions are used to modify the database, external context or Env.
  * `clear_*` functions allows setting default values for Environment.
  * `append_handler_register_*` functions are used to push handler registers.
    This will transition the builder to the `HandlerStage`.

In `SetGenericStage` we have:
  * `with_*` are found in `SetGenericStage` and are used to set the generics.

In `HandlerStage` we have:
  * `reset_handler_with_*` is used if we want to change some of the generic types this will reset the handler registers.
    This will transition the builder to the `SetGenericStage`.

# Creating and modification of Evm

Evm implements functions that allow using the `EvmBuilder` without even knowing that it exists.
The most obvious one is `Evm::builder()` that creates a new builder with default values.

Additionally, a function that is very important is `evm.modify()` that allows modifying the Evm.
It returns a builder, allowing users to modify the Evm.

# Examples
The following example uses the builder to create an `Evm` with inspector:
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
      // .with_db(..) does not compile as we already locked the builder generics,
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

The next example changes the spec id and environment of an already built evm.
```rust,ignore
  use crate::{Evm,SpecId::BERLIN};

  // Create default evm.
  let evm = Evm::builder().build();

  // Modify evm spec.
  let evm = evm.modify().with_spec_id(BERLIN).build();

  // Shortcut for above.
  let mut evm = evm.modify_spec_id(BERLIN);

  // Execute the evm.
  let output1 = evm.transact();

  // Example of modifying the tx env.
  let mut evm = evm.modify().modify_tx_env(|env| env.gas_price = 0.into()).build();

  // Execute the evm with modified tx env.
  let output2 = evm.transact();
```

Example of adding custom precompiles to Evm.

```rust,ignore
use super::SpecId;
use crate::{
    db::EmptyDB,
    inspector::inspector_handle_register,
    inspectors::NoOpInspector,
    primitives::{Address, Bytes, ContextStatefulPrecompile, ContextPrecompile, PrecompileResult},
    Context, Evm, EvmContext,
};
use std::sync::Arc;

struct CustomPrecompile;

impl ContextStatefulPrecompile<EvmContext<EmptyDB>, ()> for CustomPrecompile {
    fn call(
        &self,
        _input: &Bytes,
        _gas_limit: u64,
        _context: &mut EvmContext<EmptyDB>,
        _extctx: &mut (),
    ) -> PrecompileResult {
        Ok((10, Bytes::new()))
    }
}
fn main() {
    let mut evm = Evm::builder()
        .with_empty_db()
        .with_spec_id(SpecId::HOMESTEAD)
        .append_handler_register(|handler| {
            let precompiles = handler.pre_execution.load_precompiles();
            handler.pre_execution.load_precompiles = Arc::new(move || {
                let mut precompiles = precompiles.clone();
                precompiles.extend([(
                    Address::ZERO,
                    ContextPrecompile::ContextStateful(Arc::new(CustomPrecompile)),
                )]);
                precompiles
            });
        })
        .build();

    evm.transact().unwrap();
}

```

## Appending handler registers

Handler registers are simple functions that allow modifying the `Handler` logic by replacing the handler functions.
They are used to add custom logic to the evm execution but as they are free to modify the `Handler` in any form they want.
There may be conflicts if handlers that override the same function are added.

The most common use case for adding new logic to `Handler` is `Inspector` that is used to inspect the execution of the evm.
Example of this can be found in [`Inspector`](./inspector.md) documentation.
