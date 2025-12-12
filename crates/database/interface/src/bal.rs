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

    /// Fetch account from database and apply bal changes to it.
    #[inline]
    pub fn basic(
        &self,
        address: Address,
        mut basic: Option<AccountInfo>,
    ) -> Result<Option<AccountInfo>, BalError> {
        if let Some(bal) = &self.bal {
            let is_none = basic.is_none();
            let mut bal_basic = basic.unwrap_or_default();
            if bal.populate_account_info(address, self.bal_index, &mut bal_basic)? {
                // return new basic if it got changed.
                return Ok(Some(bal_basic));
            }

            // if it is not changed, check if it is none and return it.
            if is_none {
                return Ok(None);
            }

            basic = Some(bal_basic);
        }

        Ok(basic)
    }

    /// Fetch storage from database and apply bal changes to it.
    pub fn storage(
        &self,
        address: Address,
        key: StorageKey,
        mut value: StorageValue,
    ) -> Result<StorageValue, BalError> {
        if let Some(bal) = &self.bal {
            bal.populate_storage_slot(address, self.bal_index, key, &mut value)?;
        }
        Ok(value)
    }

    /// Fetch the storage changes from database and apply bal change to it.
    #[inline]
    pub fn storage_by_account_id(
        &self,
        account_id: usize,
        storage_key: StorageKey,
        mut value: StorageValue,
    ) -> Result<StorageValue, BalError> {
        if let Some(bal) = &self.bal {
            bal.populate_storage_slot_by_account_id(
                account_id,
                self.bal_index,
                storage_key,
                &mut value,
            )?;
        }
        Ok(value)
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
    External(ERROR),
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
            Self::External(error) => write!(f, "Database error: {error}"),
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
            Self::External(error) => error,
        }
    }
}

impl<DB: Database> Database for BalDatabase<DB> {
    type Error = EvmDatabaseError<DB::Error>;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.db
            .basic(address)
            .map_err(EvmDatabaseError::External)
            .and_then(|basic| {
                self.bal_state
                    .basic(address, basic)
                    .map_err(EvmDatabaseError::Bal)
            })
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db
            .code_by_hash(code_hash)
            .map_err(EvmDatabaseError::External)
    }

    fn storage(&mut self, address: Address, key: StorageKey) -> Result<StorageValue, Self::Error> {
        self.db
            .storage(address, key)
            .map_err(EvmDatabaseError::External)
            .and_then(|value| {
                self.bal_state
                    .storage(address, key, value)
                    .map_err(EvmDatabaseError::Bal)
            })
    }

    /// Storage id is used to access BAL index not for database index.
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.db
            .storage(address, storage_key)
            .map_err(EvmDatabaseError::External)
            .and_then(|value| {
                self.bal_state
                    .storage_by_account_id(account_id, storage_key, value)
                    .map_err(EvmDatabaseError::Bal)
            })
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.db
            .block_hash(number)
            .map_err(EvmDatabaseError::External)
    }
}

impl<DB: DatabaseCommit> DatabaseCommit for BalDatabase<DB> {
    fn commit(&mut self, changes: EvmState) {
        self.bal_state.commit(&changes);
        self.db.commit(changes);
    }
}
