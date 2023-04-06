use crate::{Bytecode, B160, U256};

/// Journal of changes to state in raw form.
/// Journal is used to revert state changes when transaction execution fails.
///
/// If inner contract is called journal new journal entry would be created and pushed
/// to the end of the vector, after inner call is finished, additional journal entry is appended.
/// so if we have one inner call there will be three journal entries.
pub type Journal = Vec<Vec<JournalEntry>>;

/// Journal entry is used to track changes to state and revert them when needed.
/// It is returned after executing transaction.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalEntry {
    /// Used to mark account that is hot inside EVM in regards to EIP-2929 AccessList.
    /// Action: We will add Account to state.
    /// Revert: we will remove account from state.
    AccountLoaded { address: B160 },
    /// Mark account to be destroyed and journal balance to be reverted
    /// Action: Mark account and transfer the balance
    /// Revert: Unmark the account and transfer balance back
    AccountDestroyed {
        address: B160,
        target: B160,
        had_balance: U256,
        was_destroyed: bool, // if account had already been destroyed before this journal entry
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched { address: B160 },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer { from: B160, to: B160, balance: U256 },
    /// Increment nonce
    /// Action: Increment nonce by one
    /// Revert: Decrement nonce by one
    NonceChange {
        address: B160, //geth has nonce value,
    },
    /// It is used to track both storage change and hot load of storage slot. For hot load in regard
    /// to EIP-2929 AccessList had_value will be None
    /// Action: Storage change or hot load
    /// Revert: Revert to previous value or remove slot from storage
    StorageChange {
        address: B160,
        key: U256,
        had_value: Option<U256>, //if none, storage slot was cold loaded from db and needs to be removed
    },
    /// Code changed
    /// Action: Account code changed
    /// Revert: Revert to previous bytecode.
    CodeChange { address: B160, had_code: Bytecode },
}
