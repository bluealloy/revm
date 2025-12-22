//! Database implementation for BAL.
use core::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};
use primitives::{Address, StorageKey, StorageValue, B256};
use state::{
    bal::{alloy::AlloyBal, Bal, BalError},
    Account, AccountInfo, Bytecode, EvmState,
};
use std::sync::Arc;

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
    pub bal_index: u64,
}

impl BalState {
    /// Create a new BAL manager.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset BAL index.
    #[inline]
    pub fn reset_bal_index(&mut self) {
        self.bal_index = 0;
    }

    /// Bump BAL index.
    #[inline]
    pub fn bump_bal_index(&mut self) {
        self.bal_index += 1;
    }

    /// Get BAL index.
    #[inline]
    pub fn bal_index(&self) -> u64 {
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

    /// Set BAL.
    #[inline]
    pub fn with_bal(mut self, bal: Arc<Bal>) -> Self {
        self.bal = Some(bal);
        self
    }

    /// Set BAL builder.
    #[inline]
    pub fn with_bal_builder(mut self) -> Self {
        self.bal_builder = Some(Bal::new());
        self
    }

    /// Take BAL builder.
    #[inline]
    pub fn take_built_bal(&mut self) -> Option<Bal> {
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
    /// Return Error if BAL is not found and Account is not
    #[inline]
    pub fn get_account_id(&self, address: &Address) -> Result<Option<usize>, BalError> {
        self.bal
            .as_ref()
            .map(|bal| {
                bal.accounts
                    .get_full(address)
                    .map(|i| i.0)
                    .ok_or(BalError::AccountNotFound)
            })
            .transpose()
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
        Ok(self.basic_by_account_id(account_id, basic))
    }

    /// Fetch account from database and apply bal changes to it by account id.
    ///
    /// Panics if account_id is invalid
    #[inline]
    pub fn basic_by_account_id(&self, account_id: usize, basic: &mut Option<AccountInfo>) -> bool {
        if let Some(bal) = &self.bal {
            let is_none = basic.is_none();
            let mut bal_basic = core::mem::take(basic).unwrap_or_default();
            bal.populate_account_info(account_id, self.bal_index, &mut bal_basic)
                .expect("Invalid account id");

            // if it is not changed, check if it is none and return it.
            if is_none {
                return true;
            }

            *basic = Some(bal_basic);
            return true;
        }
        false
    }

    /// Get storage value from BAL.
    ///
    /// Return Err if bal is present but account or storage is not found inside BAL.
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
            return Err(BalError::AccountNotFound);
        };

        Ok(bal_account
            .storage
            .get_bal_writes(storage_key)?
            .get(self.bal_index))
    }

    /// Get the storage value by account id.
    ///
    /// Return Err if bal is present but account or storage is not found inside BAL.
    ///
    ///
    #[inline]
    pub fn storage_by_account_id(
        &self,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<Option<StorageValue>, BalError> {
        let Some(bal) = &self.bal else {
            return Ok(None);
        };

        let Some((_, bal_account)) = bal.accounts.get_index(account_id) else {
            return Err(BalError::AccountNotFound);
        };

        Ok(bal_account
            .storage
            .get_bal_writes(storage_key)?
            .get(self.bal_index))
    }

    /// Apply changed from EvmState to the bal_builder
    #[inline]
    pub fn commit(&mut self, changes: &EvmState) {
        if let Some(bal_builder) = &mut self.bal_builder {
            for (address, account) in changes.iter() {
                bal_builder.update_account(self.bal_index, *address, account);
            }
        }
    }

    /// Commit one account to the BAL builder.
    #[inline]
    pub fn commit_one(&mut self, address: Address, account: &Account) {
        if let Some(bal_builder) = &mut self.bal_builder {
            bal_builder.update_account(self.bal_index, address, account);
        }
    }
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
    pub fn with_bal_option(self, bal: Option<Arc<Bal>>) -> Self {
        Self {
            bal_state: BalState {
                bal,
                ..self.bal_state
            },
            ..self
        }
    }

    /// With BAL builder.
    #[inline]
    pub fn with_bal_builder(self) -> Self {
        Self {
            bal_state: self.bal_state.with_bal_builder(),
            ..self
        }
    }

    /// Reset BAL index.
    #[inline]
    pub fn reset_bal_index(mut self) -> Self {
        self.bal_state.reset_bal_index();
        self
    }

    /// Bump BAL index.
    #[inline]
    pub fn bump_bal_index(&mut self) {
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

impl<ERROR: core::error::Error + Send + Sync + 'static> DBErrorMarker for EvmDatabaseError<ERROR> {}

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
            self.bal_state.basic_by_account_id(account_id, &mut account);
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
        account_id: usize,
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
        self.bal_state.commit(&changes);
        self.db.commit(changes);
    }
}
