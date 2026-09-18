//! Stateful construction of a BAL from sequential execution results.

use super::{alloy::AlloyBal, Bal, BlockAccessIndex};
use crate::{Account, AccountInfo};
use primitives::{Address, AddressMap, StorageKey, StorageValue};
use std::collections::BTreeMap;

/// Builds a [`Bal`] while retaining the initial values of the current index.
///
/// Unlike the serialized BAL output, the builder can merge multiple execution
/// results whose original values have been rebased after each commit. The first
/// original value seen for each account field or storage slot is retained until
/// the index advances. Later commits may use either that initial value or the
/// value before their own execution; only their final values are applied.
///
/// Indices must be nondecreasing, and every state change must be submitted in
/// execution order. Start a new builder (or call [`Self::clear`]) for each block.
/// An output [`Bal`] cannot resume construction: it does not contain the initial
/// values needed to merge further commits at its last index.
///
/// Cloning or serializing the builder preserves its construction state, including
/// the baselines. Use [`Self::into_bal`] for the output without that state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalBuilder {
    bal: Bal,
    index: Option<BlockAccessIndex>,
    originals: AddressMap<AccountOriginals>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct AccountOriginals {
    info: AccountInfo,
    storage: BTreeMap<StorageKey, StorageValue>,
}

impl BalBuilder {
    /// Creates an empty builder for a block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current output, including the latest net changes at this index.
    pub const fn bal(&self) -> &Bal {
        &self.bal
    }

    /// Consumes the builder, discarding construction state.
    pub fn into_bal(self) -> Bal {
        self.bal
    }

    /// Consumes the builder and produces canonical EIP-7928 output.
    pub fn into_alloy_bal(self) -> AlloyBal {
        self.into_bal().into_alloy_bal()
    }

    /// Clears both output and baselines so this builder can start another block.
    pub fn clear(&mut self) {
        self.bal.accounts.clear();
        self.originals.clear();
        self.index = None;
    }

    /// Merges an account's execution result at `index`.
    ///
    /// On the first submission for an account/slot at this index, its original
    /// value must be the value before that execution. Subsequent submissions may
    /// retain that baseline or rebase it to the preceding commit's final value.
    /// Include accessed, unchanged slots as well as changed slots.
    ///
    /// # Panics
    ///
    /// Panics if `index` is less than the last submitted index. Use a fresh
    /// builder or [`Self::clear`] when starting another block.
    pub fn update_account(&mut self, index: BlockAccessIndex, address: Address, account: &Account) {
        if let Some(previous) = self.index {
            assert!(index >= previous, "BAL indices must be nondecreasing");
        }
        if self.index != Some(index) {
            self.originals.clear();
            self.index = Some(index);
        }

        let originals = self.originals.entry(address).or_insert_with(|| {
            let mut info = account.original_info();
            // Only the code hash is needed to compare original and final code.
            info.code = None;
            info.account_id = None;
            AccountOriginals {
                info,
                storage: BTreeMap::new(),
            }
        });
        let output = self.bal.accounts.entry(address).or_default();
        let destroyed = account.is_selfdestructed_locally();
        let empty = AccountInfo::default();
        let present = if destroyed { &empty } else { &account.info };
        output.account_info.update(index, &originals.info, present);

        for (key, slot) in &account.storage {
            let original = originals.storage.entry(*key).or_insert(slot.original_value);
            if !destroyed {
                output.storage.storage.entry(*key).or_default().update(
                    index,
                    original,
                    slot.present_value,
                );
            }
        }
        if destroyed {
            // Destruction also clears slots visited by earlier submissions at
            // this index, even when the final execution result omits them.
            for (key, original) in &originals.storage {
                output.storage.storage.entry(*key).or_default().update(
                    index,
                    original,
                    StorageValue::ZERO,
                );
            }
        }
    }
}
