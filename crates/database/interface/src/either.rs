//! Database implementations for `either::Either` type.

use crate::{Database, DatabaseCommit, DatabaseRef};
use either::Either;
use primitives::{Address, HashMap, StorageKey, StorageValue, B256};
use state::{Account, AccountInfo, Bytecode};

impl<L, R> Database for Either<L, R>
where
    L: Database,
    R: Database<Error = L::Error>,
{
    type Error = L::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        match self {
            Self::Left(db) => db.basic(address),
            Self::Right(db) => db.basic(address),
        }
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self {
            Self::Left(db) => db.code_by_hash(code_hash),
            Self::Right(db) => db.code_by_hash(code_hash),
        }
    }

    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self {
            Self::Left(db) => db.storage(address, index),
            Self::Right(db) => db.storage(address, index),
        }
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        match self {
            Self::Left(db) => db.block_hash(number),
            Self::Right(db) => db.block_hash(number),
        }
    }

    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self {
            Self::Left(db) => db.storage_by_account_id(address, account_id, storage_key),
            Self::Right(db) => db.storage_by_account_id(address, account_id, storage_key),
        }
    }
}

impl<L, R> DatabaseCommit for Either<L, R>
where
    L: DatabaseCommit,
    R: DatabaseCommit,
{
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        match self {
            Self::Left(db) => db.commit(changes),
            Self::Right(db) => db.commit(changes),
        }
    }
}

impl<L, R> DatabaseRef for Either<L, R>
where
    L: DatabaseRef,
    R: DatabaseRef<Error = L::Error>,
{
    type Error = L::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        match self {
            Self::Left(db) => db.basic_ref(address),
            Self::Right(db) => db.basic_ref(address),
        }
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self {
            Self::Left(db) => db.code_by_hash_ref(code_hash),
            Self::Right(db) => db.code_by_hash_ref(code_hash),
        }
    }

    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self {
            Self::Left(db) => db.storage_ref(address, index),
            Self::Right(db) => db.storage_ref(address, index),
        }
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        match self {
            Self::Left(db) => db.block_hash_ref(number),
            Self::Right(db) => db.block_hash_ref(number),
        }
    }

    fn storage_by_account_id_ref(
        &self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        match self {
            Self::Left(db) => db.storage_by_account_id_ref(address, account_id, storage_key),
            Self::Right(db) => db.storage_by_account_id_ref(address, account_id, storage_key),
        }
    }
}
