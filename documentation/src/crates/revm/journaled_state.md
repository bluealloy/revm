# Journaled State

The `journaled_state` module of the `revm` crate provides a state management implementation for Ethereum-style accounts. It includes support for various actions such as self-destruction of accounts, initial account loading, account state modification, and logging. It also contains several important utility functions such as `is_precompile`.

This module is built around the `JournaledState` structure, which encapsulates the entire state of the blockchain. `JournaledState` uses an internal state representation (a `HashMap`) that tracks all accounts. Each account is represented by the `Account` structure, which includes fields like balance, nonce, and code hash. For state-changing operations, the module keeps track of all the changes within a "journal" for easy reversion and commitment to the database. This feature is particularly useful for handling reversion of state changes in case of transaction failures or other exceptions. The module interacts with a database through the `Database` trait, which abstracts the operations for fetching and storing data. This design allows for a pluggable backend where different implementations of the `Database` trait can be used to persist the state in various ways (for instance, in-memory or disk-based databases).

## Data Structures

- `JournaledState`

    This structure represents the entire state of the blockchain, including accounts, their
    associated balances, nonces, and code hashes. It maintains a journal of all state changes
    that allows for easy reversion and commitment of changes to the database.

- `Account`

    This structure represents an individual account on the blockchain. It includes the account's
    balance, nonce, and code hash. It also includes a flag indicating if the account is
    self-destructed, and a map representing the account's storage.

- `JournalEntry`

    This structure represents an entry in the `JournaledState`'s journal. Each entry describes
    an operation that changes the state, such as an account loading, an account destruction, or a
    storage change.

## Methods

- `selfdestruct`

    This method marks an account as self-destructed and transfers its balance to a target account.
    If the target account does not exist, it's created. If the self-destructed account and the
    target are the same, the balance will be lost.

-  `initial_account_and_code_load`

    This method initializes an account and loads its associated code from the database. If the
    code does not exist, an empty code is associated with the account.

-  `initial_account_load`

    This method loads an account's basic information from the database without loading the code.
    It also loads specified storage slots into memory.

- `load_account`

    This method loads an account's information into memory and returns whether the account was
    cold or warm accessed.

- `load_account_exist`

    This method checks whether an account exists or not. It returns whether the account was
    cold or warm accessed and whether it exists.

- `load_code`

    This method loads an account's code into memory from the database.

- `sload`

    This method loads a specified storage value of an account. It returns the value and whether
    the storage was cold loaded.

- `sstore`

    This method changes the value of a specified storage slot in an account and returns the
    original value, the present value, the new value, and whether the storage was cold loaded.

- `log`

    This method adds a log entry to the journal.

- `is_precompile`

    This method checks whether an address is a precompiled contract or not.

## Relevant EIPs

The JournaledState module's operations are primarily designed to comply with the Ethereum standards defined in several Ethereum Improvement Proposals (EIPs). Specifically:

### [EIP-161](https://eips.ethereum.org/EIPS/eip-161) State Trie Clearing

Overview
The [EIP-161](https://eips.ethereum.org/EIPS/eip-161) aims to optimize Ethereum's state management by deleting empty accounts. The specification was proposed by Gavin Wood and was activated in the Spurious Dragon hardfork at block number `2,675,000` on the Ethereum mainnet.proposal. The EIP focuses on four main changes:

- Account Creation: During the creation of an account (whether by transactions or the `CREATE` operation), the nonce of the new account is incremented by one before the execution of the initialization code. For most networks, the starting value is 1, but this may vary for test networks with non-zero default starting nonces.

- Call and Suicide Charges: Prior to [EIP-161](https://eips.ethereum.org/EIPS/eip-161), a gas charge of 25,000 was levied for `CALL` and `SUICIDE` operations if the destination account did not exist. With [EIP-161](https://eips.ethereum.org/EIPS/eip-161), this charge is only applied if the operation transfers more than zero value and the destination account is dead (non-existent or empty).

- Existence of Empty Accounts: An account cannot change its state from non-existent to existent-but-empty. If an operation would result in this, the account remains non-existent.

- Removal of Empty Accounts: At the end of a transaction, any account that was involved in potentially state-changing operations and is now empty will be deleted.

An account is considered "empty" if it has no code, and its nonce and balance are both zero. An account is considered "dead" if it is non-existent or it is empty. An account is considered "touched" when it is involved in any potentially state-changing operation.

These rules have an impact on how state is managed within the [EIP-161](https://eips.ethereum.org/EIPS/eip-161) context, and this affects how the JournaledState module functions. For example, operations like `initial_account_and_code_load`, `initial_account_load`, and `selfdestruct` all need to take into account whether an account is empty or dead.

#### Rationale

The rationale behind [EIP-161](https://eips.ethereum.org/EIPS/eip-161) is to optimize the Ethereum state management by getting rid of unnecessary data. Prior to this change, it was possible for the state trie to become bloated with empty accounts. This bloating resulted in increased storage requirements and slower processing times for Ethereum nodes.

By removing these empty accounts, the size of the state trie can be reduced, leading to improvements in the performance of Ethereum nodes. Additionally, the changes regarding the gas costs for `CALL` and `SUICIDE` operations add a new level of nuance to the Ethereum gas model, further optimizing transaction processing.

[EIP-161](https://eips.ethereum.org/EIPS/eip-161) has a significant impact on the state management of Ethereum, and thus is highly relevant to the JournaledState module of the revm crate. The operations defined in this module, such as loading accounts, self-destructing accounts, and changing storage, must all conform to the rules defined in [EIP-161](https://eips.ethereum.org/EIPS/eip-161).

### [EIP-658](https://eips.ethereum.org/EIPS/eip-658): Embedding transaction status code in receipts

This EIP is particularly important because it introduced a way to unambiguously determine whether a transaction was successful or not. Before the introduction of [EIP-658](https://eips.ethereum.org/EIPS/eip-658), it was impossible to determine with certainty if a transaction was successful simply based on its gas consumption. This was because with the introduction of the `REVERT` opcode in [EIP-140](https://eips.ethereum.org/EIPS/eip-140), transactions could fail without consuming all gas.

[EIP-658](https://eips.ethereum.org/EIPS/eip-658) replaced the intermediate state root field in the receipt with a status code that indicates whether the top-level call of the transaction succeeded or failed. The status code is 1 for success and 0 for failure.

This EIP affects the JournaledState module, as the result of executing transactions and their success or failure status directly influences the state of the blockchain. The execution of state-modifying methods like `initial_account_and_code_load`, `selfdestruct`, `sstore`, and `log` can result in success or failure, and the status needs to be properly reflected in the transaction receipt.

#### Rationale

The main motivation behind [EIP-658](https://eips.ethereum.org/EIPS/eip-658) was to provide an unambiguous way to determine the success or failure of a transaction. Before [EIP-658](https://eips.ethereum.org/EIPS/eip-658), users had to rely on checking if a transaction had consumed all gas to guess if it had failed. However, this was not reliable because of the introduction of the `REVERT` opcode in [EIP-140](https://eips.ethereum.org/EIPS/eip-140).

Moreover, although full nodes can replay transactions to get their return status, fast nodes can only do this for transactions after their pivot point, and light nodes cannot do it at all. This means that without [EIP-658](https://eips.ethereum.org/EIPS/eip-658), it would be impractical for a non-full node to reliably determine the status of a transaction.

[EIP-658](https://eips.ethereum.org/EIPS/eip-658) addressed this problem by embedding the status code directly in the transaction receipt, making it easily accessible. This change was minimal and non-disruptive, while it significantly improved the clarity and usability of transaction receipts.

### [EIP-2929](https://eips.ethereum.org/EIPS/eip-2929): Gas cost increases for state access opcodes

[EIP-2929](https://eips.ethereum.org/EIPS/eip-2929) proposes an increase in the gas costs for several opcodes when they're used for the first time in a transaction. The EIP was created to mitigate potential DDoS (Distributed Denial of Service) attacks by increasing the cost of potential attack vectors, and to make the stateless witness sizes in Ethereum more manageable.

[EIP-2929](https://eips.ethereum.org/EIPS/eip-2929) also introduces two sets, `accessed_addresses` and `accessed_storage_keys`, to track the addresses and storage slots that have been accessed within a transaction. This mitigates the additional gas cost for repeated operations on the same address or storage slot within a transaction, as any repeated operation on an already accessed address or storage slot will cost less gas.

In the context of this EIP, "cold" and "warm" (or "hot") refer to whether an address or storage slot has been accessed before during the execution of a transaction. If an address or storage slot is being accessed for the first time in a transaction, it is referred to as a "cold" access. If it has already been accessed within the same transaction, any subsequent access is referred to as "warm" or "hot".

- Parameters: The EIP defines new parameters such as `COLD_SLOAD_COST` (2100 gas) for a "cold" storage read, `COLD_ACCOUNT_ACCESS_COST` (2600 gas) for a "cold" account access, and `WARM_STORAGE_READ_COST` (100 gas) for a "warm" storage read.

- Storage read changes: For `SLOAD` operation, if the (address, storage_key) pair is not yet in `accessed_storage_keys`, `COLD_SLOAD_COST` gas is charged and the pair is added to `accessed_storage_keys`. If the pair is already in `accessed_storage_keys`, `WARM_STORAGE_READ_COST` gas is charged.

- Account access changes: When an address is the target of certain opcodes (`EXTCODESIZE`, `EXTCODECOPY`, `EXTCODEHASH`, `BALANCE`, `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`), if the target is not in `accessed_addresses`, `COLD_ACCOUNT_ACCESS_COST` gas is charged, and the address is added to `accessed_addresses`. Otherwise, `WARM_STORAGE_READ_COST` gas is charged.

- `SSTORE` changes: For `SSTORE` operation, if the (address, storage_key) pair is not in `accessed_storage_keys`, an additional `COLD_SLOAD_COST` gas is charged, and the pair is added to `accessed_storage_keys`.

- `SELFDESTRUCT` changes: If the recipient of `SELFDESTRUCT` is not in `accessed_addresses`, an additional `COLD_ACCOUNT_ACCESS_COST` is charged, and the recipient is added to the set.

This methodology allows Ethereum to maintain an internal record of accessed accounts and storage slots within a transaction, making it possible to charge lower gas fees for repeated operations, thereby reducing the cost for such operations.

#### Rationale

- Security: Previously, these opcodes were underpriced, making them susceptible to DoS attacks where an attacker would simply send transactions that access or call a large number of accounts. By increasing the gas costs, the EIP intends to mitigate these potential security risks.

- Improving stateless witness sizes: Stateless Ethereum clients don't maintain the complete state of the blockchain, but instead rely on block "witnesses" (a list of all the accounts, storage, and contract code accessed during transaction execution) to validate transactions. This EIP helps in reducing the size of these witnesses, thereby making stateless Ethereum more viable.
