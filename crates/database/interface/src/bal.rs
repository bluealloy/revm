//! Database implementation for BAL.
use core::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};
use primitives::{Address, StorageKey, StorageKeyMap, StorageValue, B256};
use state::{
    bal::{alloy::AlloyBal, AccountBal, Bal, BalError, BlockAccessIndex},
    Account, AccountId, AccountInfo, Bytecode, EvmState,
};
use std::{sync::Arc, vec::Vec};

use crate::{DBErrorMarker, Database, DatabaseCommit};

/// Contains both the BAL for reads and BAL builders.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalState {
    /// BAL used to execute transactions.
    pub bal: Option<Arc<Bal>>,
    /// BAL builder that is used to build BAL.
    /// It is create from State output of transaction execution.
    pub bal_builder: Option<Bal>,
    /// BAL index, used by bal to fetch appropriate values and used by bal_builder on commit
    /// to submit changes.
    pub bal_index: BlockAccessIndex,
    /// Whether reads not covered by the BAL fall back to the underlying database instead of
    /// returning an error.
    ///
    /// During block validation an access outside the BAL means the BAL is invalid, so this
    /// defaults to `false`. Enabling it allows executing transactions that are not part of the
    /// block (e.g. RPC calls) on top of BAL-positioned state: state not covered by the BAL is
    /// untouched by the block, so the database values are correct.
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow_db_fallback: bool,
    /// Verification state for checking committed accesses against the attached BAL.
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub bal_verifier: Option<BalVerificationState>,
}

impl BalState {
    /// Create a new BAL manager.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset BAL index to pre-execution.
    #[inline]
    pub const fn reset_bal_index(&mut self) {
        self.bal_index = BlockAccessIndex::PRE_EXECUTION;
    }

    /// Bump BAL index.
    #[inline]
    pub const fn bump_bal_index(&mut self) {
        self.bal_index.increment();
    }

    /// Get BAL index.
    #[inline]
    pub const fn bal_index(&self) -> BlockAccessIndex {
        self.bal_index
    }

    /// Get BAL.
    #[inline]
    pub fn bal(&self) -> Option<Arc<Bal>> {
        self.bal.clone()
    }

    /// Get BAL builder.
    #[inline]
    pub fn bal_builder(&self) -> Option<Bal> {
        self.bal_builder.clone()
    }

    /// Returns true if commits need to update BAL state.
    #[inline]
    pub const fn tracks_commits(&self) -> bool {
        self.bal_builder.is_some() || self.bal_verifier.is_some()
    }

    /// Set BAL.
    #[inline]
    pub fn with_bal(mut self, bal: Arc<Bal>) -> Self {
        self.set_bal(Some(bal));
        self
    }

    /// Set BAL builder.
    #[inline]
    pub fn with_bal_builder(mut self) -> Self {
        self.bal_builder = Some(Bal::new());
        self
    }

    /// Enable BAL verification against the attached BAL.
    #[inline]
    pub fn with_bal_verifier(mut self) -> Self {
        self.enable_bal_verifier();
        self
    }

    /// Set BAL.
    #[inline]
    pub fn set_bal(&mut self, bal: Option<Arc<Bal>>) {
        self.bal = bal;
        if self.bal_verifier.is_some() {
            self.enable_bal_verifier();
        }
    }

    /// Enable BAL verification against the attached BAL.
    #[inline]
    pub fn enable_bal_verifier(&mut self) {
        self.bal_verifier = Some(match self.bal.as_deref() {
            Some(bal) => BalVerificationState::new(bal),
            None => BalVerificationState::missing_bal(),
        });
    }

    /// Disable BAL verification.
    #[inline]
    pub fn disable_bal_verifier(&mut self) {
        self.bal_verifier = None;
    }

    /// Verify that all declared BAL entries were observed by execution.
    #[inline]
    pub fn verify_bal(&self) -> Result<(), BalVerificationError> {
        match (&self.bal_verifier, self.bal.as_deref()) {
            (Some(verifier), Some(bal)) => verifier.verify(bal),
            (Some(_), None) => Err(BalVerificationError::MissingBal),
            (None, _) => Ok(()),
        }
    }

    /// Set whether reads not covered by the BAL fall back to the underlying database.
    ///
    /// See [`Self::allow_db_fallback`].
    #[inline]
    pub const fn with_allow_db_fallback(mut self, allow: bool) -> Self {
        self.allow_db_fallback = allow;
        self
    }

    /// Set whether reads not covered by the BAL fall back to the underlying database.
    ///
    /// See [`Self::allow_db_fallback`].
    #[inline]
    pub const fn set_allow_db_fallback(&mut self, allow: bool) {
        self.allow_db_fallback = allow;
    }

    /// Take BAL builder.
    #[inline]
    pub const fn take_built_bal(&mut self) -> Option<Bal> {
        self.reset_bal_index();
        self.bal_builder.take()
    }

    /// Take built BAL as AlloyBAL.
    #[inline]
    pub fn take_built_alloy_bal(&mut self) -> Option<AlloyBal> {
        self.take_built_bal().map(|bal| bal.into_alloy_bal())
    }

    /// Get account id from BAL.
    ///
    /// Returns `Ok(None)` if no BAL is attached, or if [`Self::allow_db_fallback`] is enabled and the
    /// account is not covered by the BAL.
    ///
    /// Return Error if the BAL is attached but does not contain the account and fallback is
    /// disabled.
    #[inline]
    pub fn get_account_id(&self, address: &Address) -> Result<Option<AccountId>, BalError> {
        let Some(bal) = self.bal.as_ref() else {
            return Ok(None);
        };
        match bal.accounts.get_full(address) {
            Some(i) => Ok(Some(AccountId::new(i.0).expect("too many bals"))),
            None if self.allow_db_fallback => Ok(None),
            None => Err(BalError::AccountNotFound { address: *address }),
        }
    }

    /// Fetch account from database and apply bal changes to it.
    ///
    /// Return Some if BAL is existing, None if not.
    /// Return Err if Accounts is not found inside BAL.
    /// And return true
    #[inline]
    pub fn basic(
        &self,
        address: Address,
        basic: &mut Option<AccountInfo>,
    ) -> Result<bool, BalError> {
        let Some(account_id) = self.get_account_id(&address)? else {
            return Ok(false);
        };
        self.basic_by_account_id(account_id, basic)
    }

    /// Fetch account from database and apply bal changes to it by account id.
    #[inline]
    pub fn basic_by_account_id(
        &self,
        account_id: AccountId,
        basic: &mut Option<AccountInfo>,
    ) -> Result<bool, BalError> {
        let Some(bal) = &self.bal else {
            return Ok(false);
        };
        let is_none = basic.is_none();
        let mut bal_basic = core::mem::take(basic).unwrap_or_default();
        let changed = bal.populate_account_info(account_id, self.bal_index, &mut bal_basic)?;

        // If account was not in DB and BAL has no changes, keep it as None.
        if !changed && is_none {
            return Ok(true);
        }

        *basic = Some(bal_basic);
        Ok(true)
    }

    /// Get storage value from BAL.
    ///
    /// Returns `Ok(None)` if no BAL is attached, or if [`Self::allow_db_fallback`] is enabled and the
    /// account or slot is not covered by the BAL.
    ///
    /// Return Err if bal is present but account or storage is not found inside BAL and fallback
    /// is disabled.
    #[inline]
    pub fn storage(
        &self,
        account: &Address,
        storage_key: StorageKey,
    ) -> Result<Option<StorageValue>, BalError> {
        let Some(bal) = &self.bal else {
            return Ok(None);
        };

        let Some(bal_account) = bal.accounts.get(account) else {
            if self.allow_db_fallback {
                return Ok(None);
            }
            return Err(BalError::AccountNotFound { address: *account });
        };

        match bal_account.storage.get_bal_writes(account, storage_key) {
            Ok(writes) => Ok(writes.get(self.bal_index)),
            Err(BalError::SlotNotFound { .. }) if self.allow_db_fallback => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Get the storage value by account id.
    ///
    /// Returns `Ok(None)` if no BAL is attached, or if [`Self::allow_db_fallback`] is enabled and the
    /// slot is not covered by the BAL.
    ///
    /// Return Err if the account id is invalid, or if the slot is not found inside BAL and
    /// fallback is disabled.
    #[inline]
    pub fn storage_by_account_id(
        &self,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<Option<StorageValue>, BalError> {
        let Some(bal) = &self.bal else {
            return Ok(None);
        };

        let Some((address, bal_account)) = bal.accounts.get_index(account_id.get()) else {
            return Err(BalError::InvalidAccountId { account_id });
        };

        match bal_account.storage.get_bal_writes(address, storage_key) {
            Ok(writes) => Ok(writes.get(self.bal_index)),
            Err(BalError::SlotNotFound { .. }) if self.allow_db_fallback => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Apply changed from EvmState to the bal_builder
    #[inline]
    pub fn commit(&mut self, changes: &EvmState) {
        if let Some(bal_builder) = &mut self.bal_builder {
            for (address, account) in changes.iter() {
                bal_builder.update_account(self.bal_index, *address, account);
            }
        }

        if let Some(verifier) = &mut self.bal_verifier {
            match self.bal.as_deref() {
                Some(bal) => verifier.commit(self.bal_index, bal, changes),
                None => verifier.record_error(BalVerificationError::MissingBal),
            }
        }
    }

    /// Commit one account to the BAL builder.
    #[inline]
    pub fn commit_one(&mut self, address: Address, account: &Account) {
        if let Some(bal_builder) = &mut self.bal_builder {
            bal_builder.update_account(self.bal_index, address, account);
        }

        if let Some(verifier) = &mut self.bal_verifier {
            match self.bal.as_deref() {
                Some(bal) => {
                    if let Err(error) = verifier.commit_one(self.bal_index, bal, address, account) {
                        verifier.record_error(error);
                    }
                }
                None => verifier.record_error(BalVerificationError::MissingBal),
            }
        }
    }
}

/// Tracks which declared BAL entries were actually observed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BalVerificationState {
    accounts: Vec<BalAccountVerification>,
    error: Option<BalVerificationError>,
}

impl BalVerificationState {
    fn new(bal: &Bal) -> Self {
        Self {
            accounts: bal
                .accounts
                .iter()
                .map(|(_, account)| BalAccountVerification::new(account))
                .collect(),
            error: None,
        }
    }

    const fn missing_bal() -> Self {
        Self {
            accounts: Vec::new(),
            error: Some(BalVerificationError::MissingBal),
        }
    }

    const fn record_error(&mut self, error: BalVerificationError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn commit(&mut self, index: BlockAccessIndex, bal: &Bal, changes: &EvmState) {
        if self.error.is_some() {
            return;
        }

        for (address, account) in changes {
            if let Err(error) = self.commit_one(index, bal, *address, account) {
                self.record_error(error);
                break;
            }
        }
    }

    fn commit_one(
        &mut self,
        index: BlockAccessIndex,
        bal: &Bal,
        address: Address,
        account: &Account,
    ) -> Result<(), BalVerificationError> {
        let (account_index, bal_account) = bal_account_by_id_or_address(bal, address, account)?;
        let seen_account = self
            .accounts
            .get_mut(account_index)
            .ok_or(BalError::AccountNotFound { address })?;

        seen_account.seen = true;

        let original = account.original_info();
        let empty = AccountInfo::default();
        let present = if account.is_selfdestructed_locally() {
            &empty
        } else {
            &account.info
        };

        verify_account_writes(index, &original, present, bal_account, seen_account)?;
        verify_storage_writes(index, address, account, bal_account, seen_account)
    }

    fn verify(&self, bal: &Bal) -> Result<(), BalVerificationError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }

        for (account_index, (_, bal_account)) in bal.accounts.iter().enumerate() {
            let Some(seen_account) = self.accounts.get(account_index) else {
                return Err(BalVerificationError::UnusedEntry);
            };

            if !seen_account.seen {
                return Err(BalVerificationError::UnusedEntry);
            }

            if seen_account.balance_next != bal_account.account_info.balance.writes.len()
                || seen_account.nonce_next != bal_account.account_info.nonce.writes.len()
                || seen_account.code_next != bal_account.account_info.code.writes.len()
                || seen_account.remaining_storage != 0
            {
                return Err(BalVerificationError::UnusedEntry);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BalAccountVerification {
    seen: bool,
    balance_next: usize,
    nonce_next: usize,
    code_next: usize,
    remaining_storage: usize,
    storage: StorageKeyMap<BalSlotVerification>,
}

impl BalAccountVerification {
    fn new(account: &AccountBal) -> Self {
        Self {
            seen: false,
            balance_next: 0,
            nonce_next: 0,
            code_next: 0,
            remaining_storage: account.storage.storage.len(),
            storage: StorageKeyMap::default(),
        }
    }

    fn mark_storage_complete(&mut self, slot: StorageKey) -> Result<(), BalVerificationError> {
        let slot = self.storage.entry(slot).or_default();
        if slot.complete {
            return Ok(());
        }

        slot.complete = true;
        self.remaining_storage = self
            .remaining_storage
            .checked_sub(1)
            .ok_or(BalVerificationError::UnusedEntry)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BalSlotVerification {
    next: usize,
    complete: bool,
}

/// Error returned when committed execution state does not match the submitted BAL.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BalVerificationError {
    /// BAL verification was enabled but no BAL was attached.
    #[error("BAL verification enabled without an attached BAL")]
    MissingBal,
    /// BAL lookup failed.
    #[error(transparent)]
    Bal(#[from] BalError),
    /// Execution changed state without a matching BAL write at the current index.
    #[error("missing BAL write")]
    MissingWrite,
    /// Execution changed state to a value different from the BAL write.
    #[error("mismatched BAL write")]
    MismatchedWrite,
    /// The BAL declared an entry that execution did not observe.
    #[error("unused BAL entry")]
    UnusedEntry,
}

fn verify_account_writes(
    index: BlockAccessIndex,
    original: &AccountInfo,
    present: &AccountInfo,
    bal_account: &AccountBal,
    seen_account: &mut BalAccountVerification,
) -> Result<(), BalVerificationError> {
    if original.balance != present.balance {
        verify_write(
            index,
            &present.balance,
            &bal_account.account_info.balance.writes,
            &mut seen_account.balance_next,
        )?;
    }

    if original.nonce != present.nonce {
        verify_write(
            index,
            &present.nonce,
            &bal_account.account_info.nonce.writes,
            &mut seen_account.nonce_next,
        )?;
    }

    if original.code_hash != present.code_hash {
        let code = (present.code_hash, present.code.clone().unwrap_or_default());
        verify_write(
            index,
            &code,
            &bal_account.account_info.code.writes,
            &mut seen_account.code_next,
        )?;
    }

    Ok(())
}

fn verify_storage_writes(
    index: BlockAccessIndex,
    address: Address,
    account: &Account,
    bal_account: &AccountBal,
    seen_account: &mut BalAccountVerification,
) -> Result<(), BalVerificationError> {
    for (slot, value) in &account.storage {
        let writes = bal_account
            .storage
            .storage
            .get(slot)
            .ok_or(BalError::SlotNotFound {
                address,
                slot: *slot,
            })?;

        if account.is_selfdestructed_locally() {
            if value.original_value != StorageValue::ZERO {
                verify_storage_write(
                    index,
                    *slot,
                    &StorageValue::ZERO,
                    &writes.writes,
                    seen_account,
                )?;
            } else if writes.writes.is_empty() {
                seen_account.mark_storage_complete(*slot)?;
            }
        } else if value.is_changed() {
            verify_storage_write(
                index,
                *slot,
                &value.present_value,
                &writes.writes,
                seen_account,
            )?;
        } else if writes.writes.is_empty() {
            seen_account.mark_storage_complete(*slot)?;
        }
    }

    Ok(())
}

fn bal_account_by_id_or_address<'a>(
    bal: &'a Bal,
    address: Address,
    account: &Account,
) -> Result<(usize, &'a AccountBal), BalVerificationError> {
    if let Some(account_id) = account.info.account_id {
        if let Some((stored_address, bal_account)) = bal.accounts.get_index(account_id.get()) {
            if *stored_address == address {
                return Ok((account_id.get(), bal_account));
            }
        }

        return Err(BalError::AccountNotFound { address }.into());
    }

    bal.accounts
        .get_full(&address)
        .map(|(index, _, account)| (index, account))
        .ok_or(BalError::AccountNotFound { address }.into())
}

fn verify_storage_write<T: PartialEq>(
    index: BlockAccessIndex,
    slot: StorageKey,
    actual: &T,
    writes: &[(BlockAccessIndex, T)],
    seen_account: &mut BalAccountVerification,
) -> Result<(), BalVerificationError> {
    let slot_seen = seen_account.storage.entry(slot).or_default();
    verify_write(index, actual, writes, &mut slot_seen.next)?;
    if slot_seen.next == writes.len() {
        seen_account.mark_storage_complete(slot)?;
    }
    Ok(())
}

fn verify_write<T: PartialEq>(
    index: BlockAccessIndex,
    actual: &T,
    writes: &[(BlockAccessIndex, T)],
    next: &mut usize,
) -> Result<(), BalVerificationError> {
    let Some((write_index, expected)) = writes.get(*next) else {
        return Err(BalVerificationError::MissingWrite);
    };

    if *write_index < index {
        return Err(BalVerificationError::UnusedEntry);
    }
    if *write_index > index {
        return Err(BalVerificationError::MissingWrite);
    }
    if expected != actual {
        return Err(BalVerificationError::MismatchedWrite);
    }

    *next += 1;
    Ok(())
}

/// Database implementation for BAL.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalDatabase<DB> {
    /// BAL manager.
    pub bal_state: BalState,
    /// Database.
    pub db: DB,
}

impl<DB> Deref for BalDatabase<DB> {
    type Target = DB;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<DB> DerefMut for BalDatabase<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl<DB> BalDatabase<DB> {
    /// Create a new BAL database.
    #[inline]
    pub fn new(db: DB) -> Self {
        Self {
            bal_state: BalState::default(),
            db,
        }
    }

    /// With BAL.
    #[inline]
    pub fn with_bal_option(mut self, bal: Option<Arc<Bal>>) -> Self {
        self.bal_state.set_bal(bal);
        self
    }

    /// With BAL builder.
    #[inline]
    pub fn with_bal_builder(self) -> Self {
        Self {
            bal_state: self.bal_state.with_bal_builder(),
            ..self
        }
    }

    /// Enable BAL verification against the attached BAL.
    #[inline]
    pub fn with_bal_verifier(mut self) -> Self {
        self.bal_state.enable_bal_verifier();
        self
    }

    /// Enable BAL verification against the attached BAL.
    #[inline]
    pub fn enable_bal_verifier(&mut self) {
        self.bal_state.enable_bal_verifier();
    }

    /// Disable BAL verification.
    #[inline]
    pub fn disable_bal_verifier(&mut self) {
        self.bal_state.disable_bal_verifier();
    }

    /// Verify that all declared BAL entries were observed by execution.
    #[inline]
    pub fn verify_bal(&self) -> Result<(), BalVerificationError> {
        self.bal_state.verify_bal()
    }

    /// Set whether reads not covered by the BAL fall back to the underlying database.
    ///
    /// See [`BalState::allow_db_fallback`].
    #[inline]
    pub const fn with_allow_bal_db_fallback(mut self, allow: bool) -> Self {
        self.bal_state.allow_db_fallback = allow;
        self
    }

    /// Reset BAL index.
    #[inline]
    pub const fn reset_bal_index(mut self) -> Self {
        self.bal_state.reset_bal_index();
        self
    }

    /// Bump BAL index.
    #[inline]
    pub const fn bump_bal_index(&mut self) {
        self.bal_state.bump_bal_index();
    }
}

/// Error type from database.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EvmDatabaseError<ERROR> {
    /// BAL error.
    Bal(BalError),
    /// External database error.
    Database(ERROR),
}

impl<ERROR> From<BalError> for EvmDatabaseError<ERROR> {
    fn from(error: BalError) -> Self {
        Self::Bal(error)
    }
}

impl<ERROR: core::error::Error + Send + Sync + 'static> DBErrorMarker for EvmDatabaseError<ERROR> {
    fn is_fatal(&self) -> bool {
        match self {
            Self::Bal(_) => false,
            Self::Database(_) => true,
        }
    }
}

impl<ERROR: Display> Display for EvmDatabaseError<ERROR> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bal(error) => write!(f, "Bal error: {error}"),
            Self::Database(error) => write!(f, "Database error: {error}"),
        }
    }
}

impl<ERROR: Error> Error for EvmDatabaseError<ERROR> {}

impl<ERROR> EvmDatabaseError<ERROR> {
    /// Convert BAL database error to database error.
    ///
    /// Panics if BAL error is present.
    pub fn into_external_error(self) -> ERROR {
        match self {
            Self::Bal(_) => panic!("Expected database error, got BAL error"),
            Self::Database(error) => error,
        }
    }
}

impl<DB: Database> Database for BalDatabase<DB> {
    type Error = EvmDatabaseError<DB::Error>;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let account_id = self.bal_state.get_account_id(&address)?;

        let mut account = self.db.basic(address).map_err(EvmDatabaseError::Database)?;

        if let Some(account_id) = account_id {
            self.bal_state
                .basic_by_account_id(account_id, &mut account)?;
        }

        Ok(account)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db
            .code_by_hash(code_hash)
            .map_err(EvmDatabaseError::Database)
    }

    #[inline]
    fn storage(&mut self, address: Address, key: StorageKey) -> Result<StorageValue, Self::Error> {
        if let Some(storage) = self.bal_state.storage(&address, key)? {
            return Ok(storage);
        }

        self.db
            .storage(address, key)
            .map_err(EvmDatabaseError::Database)
    }

    #[inline]
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        if let Some(value) = self
            .bal_state
            .storage_by_account_id(account_id, storage_key)?
        {
            return Ok(value);
        }

        self.db
            .storage(address, storage_key)
            .map_err(EvmDatabaseError::Database)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.db
            .block_hash(number)
            .map_err(EvmDatabaseError::Database)
    }
}

impl<DB: DatabaseCommit> DatabaseCommit for BalDatabase<DB> {
    fn commit(&mut self, changes: EvmState) {
        if self.bal_state.tracks_commits() {
            self.bal_state.commit(&changes);
        }
        self.db.commit(changes);
    }

    fn commit_iter(&mut self, changes: &mut dyn Iterator<Item = (Address, Account)>) {
        if self.bal_state.tracks_commits() {
            let bal_state = &mut self.bal_state;
            let mut changes = changes.map(|(address, account)| {
                bal_state.commit_one(address, &account);
                (address, account)
            });
            self.db.commit_iter(&mut changes);
        } else {
            self.db.commit_iter(changes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::U256;
    use state::bal::{AccountBal, BalWrites};
    use state::{EvmStorageSlot, TransactionId};

    fn bal_with_account(address: Address, slot: StorageKey) -> Arc<Bal> {
        let mut account = AccountBal::default();
        account.storage.storage.insert(
            slot,
            BalWrites::new(vec![(BlockAccessIndex::new(1), StorageValue::from(42u64))]),
        );
        Arc::new(Bal::from_iter([(address, account)]))
    }

    #[test]
    fn bal_misses_error_without_fallback() {
        let address = Address::with_last_byte(1);
        let missing = Address::with_last_byte(2);
        let slot = U256::from(1);
        let missing_slot = U256::from(2);
        let bal_state = BalState::new().with_bal(bal_with_account(address, slot));

        assert_eq!(
            bal_state.get_account_id(&missing),
            Err(BalError::AccountNotFound { address: missing })
        );
        assert_eq!(
            bal_state.storage(&missing, slot),
            Err(BalError::AccountNotFound { address: missing })
        );
        assert_eq!(
            bal_state.storage(&address, missing_slot),
            Err(BalError::SlotNotFound {
                address,
                slot: missing_slot
            })
        );
    }

    #[test]
    fn bal_misses_fall_back_to_database_with_fallback() {
        let address = Address::with_last_byte(1);
        let missing = Address::with_last_byte(2);
        let slot = U256::from(1);
        let missing_slot = U256::from(2);
        let mut bal_state = BalState::new()
            .with_bal(bal_with_account(address, slot))
            .with_allow_db_fallback(true);

        // Misses fall through to the database instead of erroring.
        assert_eq!(bal_state.get_account_id(&missing), Ok(None));
        assert_eq!(bal_state.storage(&missing, slot), Ok(None));
        assert_eq!(bal_state.storage(&address, missing_slot), Ok(None));

        // Reads covered by the BAL are still served from it.
        bal_state.bal_index = BlockAccessIndex::new(2);
        assert!(bal_state.get_account_id(&address).unwrap().is_some());
        assert_eq!(
            bal_state.storage(&address, slot),
            Ok(Some(StorageValue::from(42u64)))
        );
    }

    fn idx(index: u64) -> BlockAccessIndex {
        BlockAccessIndex::new(index)
    }

    fn evm_state(address: Address, account: Account) -> EvmState {
        EvmState::from_iter([(address, account)])
    }

    #[test]
    fn bal_verifier_accepts_matching_account_and_storage_writes() {
        let address = Address::with_last_byte(1);
        let slot = U256::from(1);
        let mut bal_account = AccountBal::default();
        bal_account.account_info.balance = BalWrites::new(vec![(idx(1), U256::from(7))]);
        bal_account
            .storage
            .storage
            .insert(slot, BalWrites::new(vec![(idx(1), U256::from(42))]));
        let bal = Arc::new(Bal::from_iter([(address, bal_account)]));

        let mut account = Account::from(AccountInfo::default());
        account.info.balance = U256::from(7);
        account.storage.insert(
            slot,
            EvmStorageSlot::new_changed(U256::ZERO, U256::from(42), TransactionId::ZERO),
        );

        let mut bal_state = BalState::new().with_bal(bal).with_bal_verifier();
        bal_state.bal_index = idx(1);
        bal_state.commit(&evm_state(address, account));

        assert_eq!(bal_state.verify_bal(), Ok(()));
    }

    #[test]
    fn bal_verifier_accepts_multiple_storage_writes_in_order() {
        let address = Address::with_last_byte(1);
        let slot = U256::from(1);
        let mut bal_account = AccountBal::default();
        bal_account.storage.storage.insert(
            slot,
            BalWrites::new(vec![(idx(1), U256::from(42)), (idx(2), U256::from(43))]),
        );
        let bal = Arc::new(Bal::from_iter([(address, bal_account)]));

        let mut first = Account::from(AccountInfo::default());
        first.storage.insert(
            slot,
            EvmStorageSlot::new_changed(U256::ZERO, U256::from(42), TransactionId::ZERO),
        );

        let mut second = Account::from(AccountInfo::default());
        second.storage.insert(
            slot,
            EvmStorageSlot::new_changed(U256::from(42), U256::from(43), TransactionId::ZERO),
        );

        let mut bal_state = BalState::new().with_bal(bal).with_bal_verifier();
        bal_state.bal_index = idx(1);
        bal_state.commit(&evm_state(address, first));
        bal_state.bal_index = idx(2);
        bal_state.commit(&evm_state(address, second));

        assert_eq!(bal_state.verify_bal(), Ok(()));
    }

    #[test]
    fn bal_verifier_accepts_declared_storage_read() {
        let address = Address::with_last_byte(1);
        let slot = U256::from(1);
        let mut bal_account = AccountBal::default();
        bal_account
            .storage
            .storage
            .insert(slot, BalWrites::default());
        let bal = Arc::new(Bal::from_iter([(address, bal_account)]));

        let mut account = Account::from(AccountInfo::default());
        account.storage.insert(
            slot,
            EvmStorageSlot::new(U256::from(5), TransactionId::ZERO),
        );

        let mut bal_state = BalState::new().with_bal(bal).with_bal_verifier();
        bal_state.bal_index = idx(1);
        bal_state.commit(&evm_state(address, account));

        assert_eq!(bal_state.verify_bal(), Ok(()));
    }

    #[test]
    fn bal_verifier_rejects_write_to_declared_read_slot() {
        let address = Address::with_last_byte(1);
        let slot = U256::from(1);
        let mut bal_account = AccountBal::default();
        bal_account
            .storage
            .storage
            .insert(slot, BalWrites::default());
        let bal = Arc::new(Bal::from_iter([(address, bal_account)]));

        let mut account = Account::from(AccountInfo::default());
        account.storage.insert(
            slot,
            EvmStorageSlot::new_changed(U256::ZERO, U256::from(42), TransactionId::ZERO),
        );

        let mut bal_state = BalState::new().with_bal(bal).with_bal_verifier();
        bal_state.bal_index = idx(1);
        bal_state.commit(&evm_state(address, account));

        assert_eq!(
            bal_state.verify_bal(),
            Err(BalVerificationError::MissingWrite)
        );
    }

    #[test]
    fn bal_verifier_rejects_unused_declared_write() {
        let address = Address::with_last_byte(1);
        let mut bal_account = AccountBal::default();
        bal_account.account_info.balance = BalWrites::new(vec![(idx(1), U256::from(7))]);
        let bal = Arc::new(Bal::from_iter([(address, bal_account)]));

        let account = Account::from(AccountInfo::default());

        let mut bal_state = BalState::new().with_bal(bal).with_bal_verifier();
        bal_state.bal_index = idx(1);
        bal_state.commit(&evm_state(address, account));

        assert_eq!(
            bal_state.verify_bal(),
            Err(BalVerificationError::UnusedEntry)
        );
    }
}
