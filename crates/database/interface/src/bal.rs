//! Database implementation for BAL.

use core::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};
use primitives::{Address, StorageKey, StorageValue, B256};
use state::{
    bal::{Bal, BalError},
    AccountInfo, Bytecode, EvmState,
};
use std::sync::Arc;

use crate::{DBErrorMarker, Database, DatabaseCommit};

/// Database implementation for BAL.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalDatabase<DB> {
    /// BAL used to execute transactions.
    pub bal: Option<Arc<Bal>>,
    /// BAL builder that is used to build BAL.
    /// It is create from State output of transaction execution.
    pub bal_builder: Option<Bal>,
    /// BAL index.
    pub bal_index: u64,
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
            bal: None,
            bal_builder: None,
            bal_index: 0,
            db,
        }
    }

    /// With BAL.
    #[inline]
    pub fn with_bal_option(self, bal: Option<Arc<Bal>>) -> Self {
        Self { bal, ..self }
    }

    /// With BAL builder.
    #[inline]
    pub fn with_bal_builder(self) -> Self {
        Self {
            bal_builder: Some(Bal::new()),
            ..self
        }
    }

    /// Reset BAL index.
    #[inline]
    pub fn reset_bal_index(self) -> Self {
        Self {
            bal_index: 0,
            ..self
        }
    }

    /// Bump BAL index.
    #[inline]
    pub fn bump_bal_index(&mut self) {
        self.bal_index += 1;
    }
}

/// Error type for BAL database.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BalDatabaseError<ERROR> {
    /// BAL error.
    Bal(BalError),
    /// Database error.
    Database(ERROR),
}

impl<ERROR> From<BalError> for BalDatabaseError<ERROR> {
    fn from(error: BalError) -> Self {
        Self::Bal(error)
    }
}

impl<ERROR> DBErrorMarker for BalDatabaseError<ERROR> {}

impl<ERROR: Display> Display for BalDatabaseError<ERROR> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bal(error) => write!(f, "Bal error: {error}"),
            Self::Database(error) => write!(f, "Database error: {error}"),
        }
    }
}

impl<ERROR: Error> Error for BalDatabaseError<ERROR> {}

impl<DB: Database> Database for BalDatabase<DB> {
    type Error = BalDatabaseError<DB::Error>;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let mut basic = self.db.basic(address).map_err(BalDatabaseError::Database)?;
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

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db
            .code_by_hash(code_hash)
            .map_err(BalDatabaseError::Database)
    }

    #[doc = " Gets storage value of address at index."]
    fn storage(&mut self, address: Address, key: StorageKey) -> Result<StorageValue, Self::Error> {
        let mut value = self
            .db
            .storage(address, key)
            .map_err(BalDatabaseError::Database)?;
        if let Some(bal) = &self.bal {
            bal.populate_storage_slot(address, self.bal_index, key, &mut value)
                .map_err(BalDatabaseError::Bal)?;
        }
        Ok(value)
    }

    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let mut value = self
            .db
            .storage(address, storage_key)
            .map_err(BalDatabaseError::Database)?;
        if let Some(bal) = &self.bal {
            bal.populate_storage_slot_by_account_id(
                account_id,
                self.bal_index,
                storage_key,
                &mut value,
            )
            .map_err(BalDatabaseError::Bal)?;
        }
        Ok(value)
    }

    #[doc = " Gets block hash by block number."]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.db
            .block_hash(number)
            .map_err(BalDatabaseError::Database)
    }
}

impl<DB: DatabaseCommit> DatabaseCommit for BalDatabase<DB> {
    fn commit(&mut self, changes: EvmState) {
        if let Some(bal_builder) = &mut self.bal_builder {
            for (address, account) in changes.iter() {
                bal_builder.update_account(self.bal_index, *address, account);
            }
        }
        self.db.commit(changes);
    }
}
