# The `inner_models.rs` Module in the Rust Ethereum Virtual Machine (EVM)

The `inner_models.rs` module within this Rust EVM implementation encompasses a collection of datastructures used as internal models within the EVM. These models represent various aspects of EVM operations such as call and create inputs, call context, value transfers, and the result of self-destruction operations.

## Data Structures

-  `CallInputs` Struct

```rust
pub struct CallInputs {
    pub contract: Address,
    pub transfer: Transfer,
    pub input: Bytes,
    pub gas_limit: u64,
    pub context: CallContext,
    pub is_static: bool,
}
```
    The `CallInputs` struct is used to encapsulate the inputs to a smart contract call in the EVM. This struct includes the target contract address, the value to be transferred (if any), the input data, the gas limit for the call, the call context, and a boolean indicating if the call is a static call (a read-only operation).

- `CreateInputs` Struct

```rust
pub struct CreateInputs {
    pub caller: Address,
    pub scheme: CreateScheme,
    pub value: U256,
    pub init_code: Bytes,
    pub gas_limit: u64,
}
```
    The `CreateInputs` struct encapsulates the inputs for creating a new smart contract. This includes the address of the creator, the creation scheme, the value to be transferred, the initialization code for the new contract, and the gas limit for the creation operation.

- `CallScheme` Enum

    The `CallScheme` enum represents the type of call being made to a smart contract. The different types of calls (`CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`) represent different modes of interaction with a smart contract, each with its own semantics concerning the treatment of the message sender, value transfer, and the context in which the called code executes.

- `CallContext` Struct

```rust
pub struct CallContext {
    pub address: Address,
    pub caller: Address,
    pub code_address: Address,
    pub apparent_value: U256,
    pub scheme: CallScheme,
}
```
    The `CallContext` struct encapsulates the context of a smart contract call. This includes the executing contract's address, the caller's address, the address from which the contract code was loaded, the apparent value of the call (for `DELEGATECALL` and `CALLCODE`), and the call scheme.

- `Transfer` Struct

```rust
pub struct Transfer {
    pub source: Address,
    pub target: Address,
    pub value: U256,
}
```
The `Transfer` struct represents a value transfer between two accounts.


- `SelfDestructResult` Struct

```rust
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}
```

Finally, the `SelfDestructResult` struct captures the result of a self-destruction operation on a contract.

In summary, the `inner_models.rs` module provides several crucial data structures that facilitate the representation and handling of various EVM operations and their associated data within this Rust EVM implementation.
