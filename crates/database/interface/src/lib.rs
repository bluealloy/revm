//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(async_fn_in_trait)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

use core::convert::Infallible;

use auto_impl::auto_impl;
use core::error::Error;
use primitives::{address, Address, HashMap, StorageKey, StorageValue, B256, U256};
use state::{Account, AccountInfo, Bytecode};
use std::string::String;

// keep dependency recognition
use async_trait as _;

/// Address with all `0xff..ff` in it. Used for testing.
pub const FFADDRESS: Address = address!("0xffffffffffffffffffffffffffffffffffffffff");
/// BENCH_TARGET address
pub const BENCH_TARGET: Address = FFADDRESS;
/// BENCH_TARGET_BALANCE balance
pub const BENCH_TARGET_BALANCE: U256 = U256::from_limbs([10_000_000_000_000_000, 0, 0, 0]);
/// Address with all `0xee..ee` in it. Used for testing.
pub const EEADDRESS: Address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
/// BENCH_CALLER address
pub const BENCH_CALLER: Address = EEADDRESS;
/// BENCH_CALLER_BALANCE balance
pub const BENCH_CALLER_BALANCE: U256 = U256::from_limbs([10_000_000_000_000_000, 0, 0, 0]);

pub mod empty_db;

pub use empty_db::{EmptyDB, EmptyDBTyped};

/// Database error marker is needed to implement From conversion for Error type.
pub trait DBErrorMarker {}

/// Implement marker for `()`.
impl DBErrorMarker for () {}
impl DBErrorMarker for Infallible {}
impl DBErrorMarker for String {}

/// EVM database interface.
#[auto_impl(&mut, Box)]
pub trait Database {
    /// The database error type.
    type Error: DBErrorMarker + Error;

    /// Gets basic account information.
    async fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    async fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    async fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error>;

    /// Gets block hash by block number.
    async fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

/// EVM database commit interface.
#[auto_impl(&mut, Box)]
pub trait DatabaseCommit {
    /// Commit changes to the database.
    async fn commit(&mut self, changes: HashMap<Address, Account>);
}

/// EVM database interface.
///
/// Contains the same methods as [`Database`], but with `&self` receivers instead of `&mut self`.
///
/// Use [`WrapDatabaseRef`] to provide [`Database`] implementation for a type
/// that only implements this trait.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait DatabaseRef {
    /// The database error type.
    type Error: DBErrorMarker + Error;

    /// Gets basic account information.
    async fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    async fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    async fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error>;

    /// Gets block hash by block number.
    async fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error>;
}

/// Wraps a [`DatabaseRef`] to provide a [`Database`] implementation.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WrapDatabaseRef<T: DatabaseRef>(pub T);

impl<F: DatabaseRef> From<F> for WrapDatabaseRef<F> {
    #[inline]
    fn from(f: F) -> Self {
        WrapDatabaseRef(f)
    }
}

impl<T: DatabaseRef> Database for WrapDatabaseRef<T> {
    type Error = T::Error;

    #[inline]
    async fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic_ref(address).await
    }

    #[inline]
    async fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash_ref(code_hash).await
    }

    #[inline]
    async fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0.storage_ref(address, index).await
    }

    #[inline]
    async fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash_ref(number).await
    }
}

impl<T: DatabaseRef + DatabaseCommit> DatabaseCommit for WrapDatabaseRef<T> {
    #[inline]
    async fn commit(&mut self, changes: HashMap<Address, Account>) {
        self.0.commit(changes).await
    }
}
