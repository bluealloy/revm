//! This module contains [`JournaledAccount`] struct a wrapper around account and journal entries that
//! allow updates to the account and journal entries.
//!
//! Useful to encapsulate account and journal entries together. So when account gets changed, we can add a journal entry for it.

use crate::{
    context::{SStoreResult, StateLoad},
    journaled_state::JournalLoadError,
};

use super::entry::JournalEntryTr;
use core::ops::Deref;
use database_interface::Database;
use primitives::{
    hash_map::Entry, Address, HashMap, HashSet, StorageKey, StorageValue, B256, KECCAK_EMPTY, U256,
};
use state::{Account, Bytecode, EvmStorageSlot};
use std::vec::Vec;

/// Trait that contains database and journal of all changes that were made to the account.
pub trait JournaledAccountTr: Deref<Target = Account> {
    /// Database error type.
    type DatabaseError;
    /// Entry type.
    type Entry: JournalEntryTr;

    /// Creates a new journaled account.
    fn sload(
        &mut self,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<&mut EvmStorageSlot>, JournalLoadError<Self::DatabaseError>>;

    /// Warm loads storage slot and stores the new value
    fn sstore(
        &mut self,
        key: StorageKey,
        new: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, JournalLoadError<Self::DatabaseError>>;

    /// Loads the code of the account. and returns it as reference.
    fn load_code(&mut self) -> Result<&Bytecode, JournalLoadError<Self::DatabaseError>>;

    /// Returns the balance of the account.
    fn balance(&self) -> &U256;

    /// Returns the nonce of the account.
    fn nonce(&self) -> u64;

    /// Returns the code hash of the account.
    fn code_hash(&self) -> &B256;

    /// Returns the code of the account.
    fn code(&self) -> Option<&Bytecode>;

    /// Touches the account.
    fn touch(&mut self);

    /// Marks the account as cold without making a journal entry.
    ///
    /// Changing account without journal entry can be a footgun as reverting of the state change
    /// would not happen without entry. It is the reason why this function has an `unsafe` prefix.
    ///
    /// If account is in access list, it would still be marked as warm if account get accessed again.
    fn unsafe_mark_cold(&mut self);

    /// Sets the balance of the account.
    ///
    /// If balance is the same, we don't add a journal entry.
    ///
    /// Touches the account in all cases.
    fn set_balance(&mut self, balance: U256);

    /// Increments the balance of the account.
    ///
    /// Touches the account in all cases.
    fn incr_balance(&mut self, balance: U256) -> bool;

    /// Decrements the balance of the account.
    ///
    /// Touches the account in all cases.
    fn decr_balance(&mut self, balance: U256) -> bool;

    /// Bumps the nonce of the account.
    ///
    /// Touches the account in all cases.
    ///
    /// Returns true if nonce was bumped, false if nonce is at the max value.
    fn bump_nonce(&mut self) -> bool;

    /// Set the nonce of the account and create a journal entry.
    ///
    /// Touches the account in all cases.
    fn set_nonce(&mut self, nonce: u64);

    /// Set the nonce of the account without creating a journal entry.
    ///
    /// Changing account without journal entry can be a footgun as reverting of the state change
    /// would not happen without entry. It is the reason why this function has an `unsafe` prefix.
    fn unsafe_set_nonce(&mut self, nonce: u64);

    /// Sets the code of the account.
    ///
    /// Touches the account in all cases.
    fn set_code(&mut self, code_hash: B256, code: Bytecode);

    /// Sets the code of the account. Calculates hash of the code.
    ///
    /// Touches the account in all cases.
    fn set_code_and_hash_slow(&mut self, code: Bytecode);

    /// Delegates the account to another address (EIP-7702).
    ///
    /// This touches the account, sets the code to the delegation designation,
    /// and bumps the nonce.
    fn delegate(&mut self, address: Address);
}

/// Journaled account contains both mutable account and journal entries.
///
/// Useful to encapsulate account and journal entries together. So when account gets changed, we can add a journal entry for it.
#[derive(Debug, PartialEq, Eq)]
pub struct JournaledAccount<'a, ENTRY: JournalEntryTr, DB> {
    /// Address of the account.
    address: Address,
    /// Mutable account.
    account: &'a mut Account,
    /// Journal entries.
    journal_entries: &'a mut Vec<ENTRY>,
    /// Access list.
    access_list: &'a HashMap<Address, HashSet<StorageKey>>,
    /// Transaction ID.
    transaction_id: usize,
    /// Database used to load storage.
    db: &'a mut DB,
}

impl<'a, ENTRY: JournalEntryTr, DB: Database> JournaledAccountTr
    for JournaledAccount<'a, ENTRY, DB>
{
    type DatabaseError = <DB as Database>::Error;
    type Entry = ENTRY;

    fn sload(
        &mut self,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<&mut EvmStorageSlot>, JournalLoadError<Self::DatabaseError>> {
        self.sload(key, skip_cold_load)
    }

    fn sstore(
        &mut self,
        key: StorageKey,
        new: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, JournalLoadError<Self::DatabaseError>> {
        self.sstore(key, new, skip_cold_load)
    }

    fn load_code(&mut self) -> Result<&Bytecode, JournalLoadError<Self::DatabaseError>> {
        self.load_code()
    }

    /// Returns the balance of the account.
    #[inline]
    fn balance(&self) -> &U256 {
        &self.account.info.balance
    }

    /// Returns the nonce of the account.
    #[inline]
    fn nonce(&self) -> u64 {
        self.account.info.nonce
    }

    /// Returns the code hash of the account.
    #[inline]
    fn code_hash(&self) -> &B256 {
        &self.account.info.code_hash
    }

    /// Returns the code of the account.
    #[inline]
    fn code(&self) -> Option<&Bytecode> {
        self.account.info.code.as_ref()
    }

    /// Touches the account.
    #[inline]
    fn touch(&mut self) {
        if !self.account.status.is_touched() {
            self.account.mark_touch();
            self.journal_entries
                .push(ENTRY::account_touched(self.address));
        }
    }

    /// Marks the account as cold without making a journal entry.
    ///
    /// Changing account without journal entry can be a footgun as reverting of the state change
    /// would not happen without entry. It is the reason why this function has an `unsafe` prefix.
    ///
    /// If account is in access list, it would still be marked as warm if account get accessed again.
    #[inline]
    fn unsafe_mark_cold(&mut self) {
        self.account.mark_cold();
    }

    /// Sets the balance of the account.
    ///
    /// If balance is the same, we don't add a journal entry.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn set_balance(&mut self, balance: U256) {
        self.touch();
        if self.account.info.balance != balance {
            self.journal_entries.push(ENTRY::balance_changed(
                self.address,
                self.account.info.balance,
            ));
            self.account.info.set_balance(balance);
        }
    }

    /// Increments the balance of the account.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn incr_balance(&mut self, balance: U256) -> bool {
        self.touch();
        let Some(balance) = self.account.info.balance.checked_add(balance) else {
            return false;
        };
        self.set_balance(balance);
        true
    }

    /// Decrements the balance of the account.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn decr_balance(&mut self, balance: U256) -> bool {
        self.touch();
        let Some(balance) = self.account.info.balance.checked_sub(balance) else {
            return false;
        };
        self.set_balance(balance);
        true
    }

    /// Bumps the nonce of the account.
    ///
    /// Touches the account in all cases.
    ///
    /// Returns true if nonce was bumped, false if nonce is at the max value.
    #[inline]
    fn bump_nonce(&mut self) -> bool {
        self.touch();
        let Some(nonce) = self.account.info.nonce.checked_add(1) else {
            return false;
        };
        self.account.info.set_nonce(nonce);
        self.journal_entries.push(ENTRY::nonce_bumped(self.address));
        true
    }

    /// Set the nonce of the account and create a journal entry.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn set_nonce(&mut self, nonce: u64) {
        self.touch();
        let previous_nonce = self.account.info.nonce;
        self.account.info.set_nonce(nonce);
        self.journal_entries
            .push(ENTRY::nonce_changed(self.address, previous_nonce));
    }

    /// Set the nonce of the account without creating a journal entry.
    ///
    /// Changing account without journal entry can be a footgun as reverting of the state change
    /// would not happen without entry. It is the reason why this function has an `unsafe` prefix.
    #[inline]
    fn unsafe_set_nonce(&mut self, nonce: u64) {
        self.account.info.set_nonce(nonce);
    }

    /// Sets the code of the account.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn set_code(&mut self, code_hash: B256, code: Bytecode) {
        self.touch();
        self.account.info.set_code_hash(code_hash);
        self.account.info.set_code(code);
        self.journal_entries.push(ENTRY::code_changed(self.address));
    }

    /// Sets the code of the account. Calculates hash of the code.
    ///
    /// Touches the account in all cases.
    #[inline]
    fn set_code_and_hash_slow(&mut self, code: Bytecode) {
        let code_hash = code.hash_slow();
        self.set_code(code_hash, code);
    }

    /// Delegates the account to another address (EIP-7702).
    ///
    /// This touches the account, sets the code to the delegation designation,
    /// and bumps the nonce.
    #[inline]
    fn delegate(&mut self, address: Address) {
        let (bytecode, hash) = if address.is_zero() {
            (Bytecode::default(), KECCAK_EMPTY)
        } else {
            let bytecode = Bytecode::new_eip7702(address);
            let hash = bytecode.hash_slow();
            (bytecode, hash)
        };
        self.touch();
        self.set_code(hash, bytecode);
        self.bump_nonce();
    }
}

impl<'a, ENTRY: JournalEntryTr, DB: Database> JournaledAccount<'a, ENTRY, DB> {
    /// Creates a new journaled account.
    #[inline(never)]
    pub fn sload(
        &mut self,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<&mut EvmStorageSlot>, JournalLoadError<<DB as Database>::Error>> {
        let is_newly_created = self.account.is_created();
        let (slot, is_cold) = match self.account.storage.entry(key) {
            Entry::Occupied(occ) => {
                let slot = occ.into_mut();
                // skip load if account is cold.
                let is_cold = slot.is_cold_transaction_id(self.transaction_id);
                if is_cold && skip_cold_load {
                    return Err(JournalLoadError::ColdLoadSkipped);
                }
                slot.mark_warm_with_transaction_id(self.transaction_id);
                (slot, is_cold)
            }
            Entry::Vacant(vac) => {
                // is storage cold
                let is_cold = self
                    .access_list
                    .get(&self.address)
                    .and_then(|v| v.get(&key))
                    .is_none();

                if is_cold && skip_cold_load {
                    return Err(JournalLoadError::ColdLoadSkipped);
                }
                // if storage was cleared, we don't need to ping db.
                let value = if is_newly_created {
                    StorageValue::ZERO
                } else {
                    self.db.storage(self.address, key)?
                };

                let slot = vac.insert(EvmStorageSlot::new(value, self.transaction_id));
                (slot, is_cold)
            }
        };

        if is_cold {
            // add it to journal as cold loaded.
            self.journal_entries
                .push(ENTRY::storage_warmed(self.address, key));
        }

        Ok(StateLoad::new(slot, is_cold))
    }

    /// Warm loads storage slot and stores the new value
    #[inline]
    pub fn sstore(
        &mut self,
        key: StorageKey,
        new: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, JournalLoadError<<DB as Database>::Error>> {
        // assume that acc exists and load the slot.
        let slot = self.sload(key, skip_cold_load)?;

        let ret = Ok(StateLoad::new(
            SStoreResult {
                original_value: slot.original_value(),
                present_value: slot.present_value(),
                new_value: new,
            },
            slot.is_cold,
        ));

        // when new value is different from present, we need to add a journal entry and make a change.
        if slot.present_value != new {
            let previous_value = slot.present_value;
            // insert value into present state.
            slot.data.present_value = new;

            // add journal entry.
            self.journal_entries
                .push(ENTRY::storage_changed(self.address, key, previous_value));
        }

        ret
    }

    /// Loads the code of the account. and returns it as reference.
    #[inline]
    pub fn load_code(&mut self) -> Result<&Bytecode, JournalLoadError<<DB as Database>::Error>> {
        if self.account.info.code.is_none() {
            let hash = *self.code_hash();
            let code = if hash == KECCAK_EMPTY {
                Bytecode::default()
            } else {
                self.db.code_by_hash(hash)?
            };
            self.account.info.code = Some(code);
        }

        Ok(self.account.info.code.as_ref().unwrap())
    }
}

impl<'a, ENTRY: JournalEntryTr, DB> JournaledAccount<'a, ENTRY, DB> {
    /// Consumes the journaled account and returns the mutable account.
    #[inline]
    pub fn into_account_ref(self) -> &'a Account {
        self.account
    }

    /// Creates a new journaled account.
    #[inline]
    pub fn new(
        address: Address,
        account: &'a mut Account,
        journal_entries: &'a mut Vec<ENTRY>,
        db: &'a mut DB,
        access_list: &'a HashMap<Address, HashSet<StorageKey>>,
        transaction_id: usize,
    ) -> Self {
        Self {
            address,
            account,
            journal_entries,
            db,
            access_list,
            transaction_id,
        }
    }
}

impl<'a, ENTRY: JournalEntryTr, DB> Deref for JournaledAccount<'a, ENTRY, DB> {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        self.account
    }
}
