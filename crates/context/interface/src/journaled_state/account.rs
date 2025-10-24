//! This module contains [`JournaledAccount`] struct a wrapper around account and journal entries that
//! allow updates to the account and journal entries.
//!
//! Useful to encapsulate account and journal entries together. So when account gets changed, we can add a journal entry for it.

use super::entry::JournalEntryTr;
use core::ops::Deref;
use primitives::{Address, B256, KECCAK_EMPTY, U256};
use state::{Account, Bytecode};
use std::vec::Vec;

/// Journaled account contains both mutable account and journal entries.
///
/// Useful to encapsulate account and journal entries together. So when account gets changed, we can add a journal entry for it.
#[derive(Debug, PartialEq, Eq)]
pub struct JournaledAccount<'a, ENTRY: JournalEntryTr> {
    /// Address of the account.
    address: Address,
    /// Mutable account.
    account: &'a mut Account,
    /// Journal entries.
    journal_entries: &'a mut Vec<ENTRY>,
}

impl<'a, ENTRY: JournalEntryTr> JournaledAccount<'a, ENTRY> {
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
    ) -> Self {
        Self {
            address,
            account,
            journal_entries,
        }
    }

    /// Returns the balance of the account.
    #[inline]
    pub fn balance(&self) -> &U256 {
        &self.account.info.balance
    }

    /// Returns the nonce of the account.
    #[inline]
    pub fn nonce(&self) -> u64 {
        self.account.info.nonce
    }

    /// Returns the code hash of the account.
    #[inline]
    pub fn code_hash(&self) -> &B256 {
        &self.account.info.code_hash
    }

    /// Returns the code of the account.
    #[inline]
    pub fn code(&self) -> Option<&Bytecode> {
        self.account.info.code.as_ref()
    }

    /// Touches the account.
    #[inline]
    pub fn touch(&mut self) {
        if !self.account.status.is_touched() {
            self.account.mark_touch();
            self.journal_entries
                .push(ENTRY::account_touched(self.address));
        }
    }

    /// Sets the balance of the account.
    ///
    /// If balance is the same, we don't add a journal entry.
    ///
    /// Touches the account in all cases.
    #[inline]
    pub fn set_balance(&mut self, balance: U256) {
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
    pub fn incr_balance(&mut self, balance: U256) -> bool {
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
    pub fn decr_balance(&mut self, balance: U256) -> bool {
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
    pub fn bump_nonce(&mut self) -> bool {
        self.touch();
        let Some(nonce) = self.account.info.nonce.checked_add(1) else {
            return false;
        };
        self.account.info.set_nonce(nonce);
        self.journal_entries
            .push(ENTRY::nonce_changed(self.address));
        true
    }

    /// Sets the code of the account.
    ///
    /// Touches the account in all cases.
    #[inline]
    pub fn set_code(&mut self, code_hash: B256, code: Bytecode) {
        self.touch();
        self.account.info.set_code_hash(code_hash);
        self.account.info.set_code(code);
        self.journal_entries.push(ENTRY::code_changed(self.address));
    }

    /// Sets the code of the account. Calculates hash of the code.
    ///
    /// Touches the account in all cases.
    #[inline]
    pub fn set_code_and_hash_slow(&mut self, code: Bytecode) {
        let code_hash = code.hash_slow();
        self.set_code(code_hash, code);
    }

    /// Delegates the account to another address (EIP-7702).
    ///
    /// This touches the account, sets the code to the delegation designation,
    /// and bumps the nonce.
    #[inline]
    pub fn delegate(&mut self, address: Address) {
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

impl<'a, ENTRY: JournalEntryTr> Deref for JournaledAccount<'a, ENTRY> {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        self.account
    }
}
