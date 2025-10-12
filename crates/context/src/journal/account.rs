use core::ops::Deref;

use primitives::{B256, U256};
use state::{Account, Bytecode};

use crate::JournalEntry;

/// Journaled account contains both mutable account and journal entries.
///
/// Useful when we want to maka a change to the account and add a journal entry for it.
#[derive(Debug, PartialEq, Eq)]
pub struct JournalAccount<'a> {
    /// Mutable account.
    account: &'a mut Account,
    /// Journal entries.
    journal_entries: &'a mut Vec<JournalEntry>,
}

impl<'a> JournalAccount<'a> {
    /// Creates a new journaled account.
    #[inline]
    pub fn new(account: &'a mut Account, journal_entries: &'a mut Vec<JournalEntry>) -> Self {
        Self {
            account,
            journal_entries,
        }
    }

    pub fn balance(&self) -> &U256 {
        &self.account.info.balance
    }

    pub fn nonce(&self) -> u64 {
        self.account.info.nonce
    }

    pub fn code_hash(&self) -> &B256 {
        &self.account.info.code_hash
    }

    pub fn code(&self) -> Option<&Bytecode> {
        self.account.info.code.as_ref()
    }

    
}

impl<'a> Deref for JournalAccount<'a> {
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
