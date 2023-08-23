# Result

At the core of this module is the `ExecutionResult` enum, which describes the possible outcomes of an EVM execution: `Success`, `Revert`, and `Halt`. `Success` represents a successful transaction execution, and it holds important information such as the reason for `success` (an Eval enum), the gas used, the gas refunded, a vector of logs (`Vec<Log>`), and the output of the execution. This aligns with the stipulation in [EIP-658](https://eips.ethereum.org/EIPS/eip-658) that introduces a status code in the receipt of a transaction, indicating whether the top-level call was successful or failed.

`Revert` represents a transaction that was reverted by the `REVERT` opcode without spending all of its gas. It stores the gas used and the output. `Halt` represents a transaction that was reverted for various reasons and consumed all its gas. It stores the reason for halting (a `Halt` enum) and the gas used.

The `ExecutionResult` enum provides several methods to extract important data from an execution result, such as `is_success()`, `logs()`, `output()`, `into_output()`, `into_logs()`, and `gas_used()`. These methods facilitate accessing key details of a transaction execution.

The `EVMError` and `InvalidTransaction` enums handle different kinds of errors that can occur in an EVM, including database errors, errors specific to the transaction itself, and errors that occur due to issues with gas, among others.

The `Output` enum handles different kinds of outputs of an EVM execution, including `Call` and `Create`. This is where the output data from a successful execution or a reverted transaction is stored.
