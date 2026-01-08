//! Async database interface.
use crate::{DBErrorMarker, Database, DatabaseRef};
use core::future::Future;
use primitives::{Address, StorageKey, StorageValue, B256};
use state::{AccountInfo, Bytecode};
use tokio::runtime::{Handle, Runtime};

/// The async EVM database interface
///
/// Contains the same methods as [Database], but it returns [Future] type instead.
///
/// Use [WrapDatabaseAsync] to provide [Database] implementation for a type that only implements this trait.
pub trait DatabaseAsync {
    /// The database error type
    type Error: DBErrorMarker;

    /// Gets basic account information.
    fn basic_async(
        &mut self,
        address: Address,
    ) -> impl Future<Output = Result<Option<AccountInfo>, Self::Error>> + Send;

    /// Gets account code by its hash.
    fn code_by_hash_async(
        &mut self,
        code_hash: B256,
    ) -> impl Future<Output = Result<Bytecode, Self::Error>> + Send;

    /// Gets storage value of address at index.
    fn storage_async(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> impl Future<Output = Result<StorageValue, Self::Error>> + Send;

    /// Gets storage value of account by its id.
    ///
    /// Default implementation is to call [`DatabaseAsync::storage_async`] method.
    #[inline]
    fn storage_by_account_id_async(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> impl Future<Output = Result<StorageValue, Self::Error>> + Send {
        let _ = account_id;
        self.storage_async(address, storage_key)
    }

    /// Gets block hash by block number.
    fn block_hash_async(
        &mut self,
        number: u64,
    ) -> impl Future<Output = Result<B256, Self::Error>> + Send;
}

/// The async EVM database interface
///
/// Contains the same methods as [DatabaseRef], but it returns [Future] type instead.
///
/// Use [WrapDatabaseAsync] to provide [DatabaseRef] implementation for a type that only implements this trait.
pub trait DatabaseAsyncRef {
    /// The database error type
    type Error: DBErrorMarker;

    /// Gets basic account information.
    fn basic_async_ref(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<Option<AccountInfo>, Self::Error>> + Send;

    /// Gets account code by its hash.
    fn code_by_hash_async_ref(
        &self,
        code_hash: B256,
    ) -> impl Future<Output = Result<Bytecode, Self::Error>> + Send;

    /// Gets storage value of address at index.
    fn storage_async_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> impl Future<Output = Result<StorageValue, Self::Error>> + Send;

    /// Gets storage value of account by its id.
    ///
    /// Default implementation is to call [`DatabaseAsyncRef::storage_async_ref`] method.
    #[inline]
    fn storage_by_account_id_async_ref(
        &self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> impl Future<Output = Result<StorageValue, Self::Error>> + Send {
        let _ = account_id;
        self.storage_async_ref(address, storage_key)
    }

    /// Gets block hash by block number.
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
    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance.
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

    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime.
    ///
    /// Refer to [tokio::runtime::Builder] on how to create a runtime if you are in synchronous world.
    ///
    /// If you are already using something like [tokio::main], call [`WrapDatabaseAsync::new`] instead.
    pub fn with_runtime(db: T, runtime: Runtime) -> Self {
        let rt = HandleOrRuntime::Runtime(runtime);
        Self { db, rt }
    }

    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime handle.
    ///
    /// This generally allows you to pass any valid runtime handle, refer to [tokio::runtime::Handle] on how
    /// to obtain a handle.
    ///
    /// If you are already in asynchronous world, like [tokio::main], use [`WrapDatabaseAsync::new`] instead.
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
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.rt.block_on(self.db.storage_async(address, index))
    }

    /// Gets storage value of account by its id.
    ///
    /// Wraps [`DatabaseAsync::storage_by_account_id_async`] in a blocking call.
    #[inline]
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.rt.block_on(
            self.db
                .storage_by_account_id_async(address, account_id, storage_key),
        )
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
    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.rt.block_on(self.db.storage_async_ref(address, index))
    }

    #[inline]
    fn storage_by_account_id_ref(
        &self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.rt.block_on(
            self.db
                .storage_by_account_id_async_ref(address, account_id, storage_key),
        )
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
            Self::Handle(handle) => {
                // We use a conservative approach: if we're currently in a multi-threaded
                // tokio runtime context, we use block_in_place. This works because:
                // 1. If we're in the SAME runtime, block_in_place prevents deadlock
                // 2. If we're in a DIFFERENT runtime, block_in_place still works safely
                //    (it just moves the work off the current worker thread before blocking)
                //
                // This approach is compatible with all tokio versions and doesn't require
                // runtime identity comparison (Handle::id() is unstable feature).
                let should_use_block_in_place = Handle::try_current()
                    .ok()
                    .map(|current| {
                        // Only use block_in_place for multi-threaded runtimes
                        // (block_in_place panics on current-thread runtime)
                        !matches!(
                            current.runtime_flavor(),
                            tokio::runtime::RuntimeFlavor::CurrentThread
                        )
                    })
                    .unwrap_or(false);

                if should_use_block_in_place {
                    // We're in a multi-threaded runtime context.
                    // Use block_in_place to:
                    // 1. Move the blocking operation off the async worker thread
                    // 2. Prevent potential deadlock if this is the same runtime
                    // 3. Allow other tasks to continue executing
                    tokio::task::block_in_place(move || handle.block_on(f))
                } else {
                    // Safe to call block_on directly in these cases:
                    // - We're outside any runtime context
                    // - We're in a current-thread runtime (where block_in_place doesn't work)
                    handle.block_on(f)
                }
            }
            Self::Runtime(rt) => rt.block_on(f),
        }
    }
}
