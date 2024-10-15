use core::future::Future;

use primitives::{Address, B256, U256};
use state::{AccountInfo, Bytecode};
use tokio::runtime::{Handle, Runtime};

use crate::{Database, DatabaseRef};

/// The async EVM database interface.
///
/// Contains the same methods as [Database], but it returns [Future] type instead.
///
/// Use [WrapDatabaseAsync] to provide [Database] implementation for a type that only implements this trait.
pub trait DatabaseAsync {
    /// The database error type.
    type Error: Send;

    /// Get basic account information.
    fn basic_async(
        &mut self,
        address: Address,
    ) -> impl Future<Output = Result<Option<AccountInfo>, Self::Error>> + Send;

    /// Get account code by its hash.
    fn code_by_hash_async(
        &mut self,
        code_hash: B256,
    ) -> impl Future<Output = Result<Bytecode, Self::Error>> + Send;

    /// Get storage value of address at index.
    fn storage_async(
        &mut self,
        address: Address,
        index: U256,
    ) -> impl Future<Output = Result<U256, Self::Error>> + Send;

    /// Get block hash by block number.
    fn block_hash_async(
        &mut self,
        number: u64,
    ) -> impl Future<Output = Result<B256, Self::Error>> + Send;
}

/// The async EVM database interface.
///
/// Contains the same methods as [DatabaseRef], but it returns [Future] type instead.
///
/// Use [WrapDatabaseAsync] to provide [DatabaseRef] implementation for a type that only implements this trait.
pub trait DatabaseAsyncRef {
    /// The database error type.
    type Error: Send;

    /// Get basic account information.
    fn basic_async_ref(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<Option<AccountInfo>, Self::Error>> + Send;

    /// Get account code by its hash.
    fn code_by_hash_async_ref(
        &self,
        code_hash: B256,
    ) -> impl Future<Output = Result<Bytecode, Self::Error>> + Send;

    /// Get storage value of address at index.
    fn storage_async_ref(
        &self,
        address: Address,
        index: U256,
    ) -> impl Future<Output = Result<U256, Self::Error>> + Send;

    /// Get block hash by block number.
    fn block_hash_async_ref(
        &self,
        number: u64,
    ) -> impl Future<Output = Result<B256, Self::Error>> + Send;
}

/// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] to provide a [`Database`] implementation.
#[derive(Debug)]
pub struct WrapDatabaseAsync<T> {
    db: T,
    rt: HandleOrRuntime,
}

impl<T> WrapDatabaseAsync<T> {
    /// Wrap a [DatabaseAsync] or [DatabaseAsyncRef] instance.
    ///
    /// Returns `None` if no tokio runtime is available or if the current runtime is a current-thread runtime.
    pub fn new(db: T) -> Option<Self> {
        let rt = match Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::CurrentThread => return None,
                _ => HandleOrRuntime::Handle(handle),
            },
            Err(_) => return None,
        };
        Some(Self { db, rt })
    }

    /// Wrap a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime.
    ///
    /// Refer to [tokio::runtime::Builder] on how to create a runtime if you are in synchronous world.
    /// If you are already using something like [tokio::main], call [WrapDatabaseAsync::new] instead.
    pub fn with_runtime(db: T, runtime: Runtime) -> Self {
        let rt = HandleOrRuntime::Runtime(runtime);
        Self { db, rt }
    }

    /// Wrap a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime handle.
    ///
    /// This generally allows you to pass any valid runtime handle, refer to [tokio::runtime::Handle] on how
    /// to obtain a handle. If you are already in asynchronous world, like [tokio::main], use [WrapDatabaseAsync::new]
    /// instead.
    pub fn with_handle(db: T, handle: Handle) -> Self {
        let rt = HandleOrRuntime::Handle(handle);
        Self { db, rt }
    }
}

impl<T: DatabaseAsync> Database for WrapDatabaseAsync<T> {
    type Error = T::Error;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.rt.block_on(self.db.basic_async(address))
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.rt.block_on(self.db.code_by_hash_async(code_hash))
    }

    #[inline]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.rt.block_on(self.db.storage_async(address, index))
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.rt.block_on(self.db.block_hash_async(number))
    }
}

impl<T: DatabaseAsyncRef> DatabaseRef for WrapDatabaseAsync<T> {
    type Error = T::Error;

    #[inline]
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.rt.block_on(self.db.basic_async_ref(address))
    }

    #[inline]
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.rt.block_on(self.db.code_by_hash_async_ref(code_hash))
    }

    #[inline]
    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.rt.block_on(self.db.storage_async_ref(address, index))
    }

    #[inline]
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.rt.block_on(self.db.block_hash_async_ref(number))
    }
}

// Hold a tokio runtime handle or full runtime
#[derive(Debug)]
enum HandleOrRuntime {
    Handle(Handle),
    Runtime(Runtime),
}

impl HandleOrRuntime {
    #[inline]
    fn block_on<F>(&self, f: F) -> F::Output
    where
        F: Future + Send,
        F::Output: Send,
    {
        match self {
            Self::Handle(handle) => tokio::task::block_in_place(move || handle.block_on(f)),
            Self::Runtime(rt) => rt.block_on(f),
        }
    }
}
