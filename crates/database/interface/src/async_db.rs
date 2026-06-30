//! Async database interface.
use crate::{DBErrorMarker, Database, DatabaseCommit, DatabaseRef};
use core::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll},
};
use corosensei::{stack::DefaultStack, Coroutine, CoroutineResult, Yielder};
use primitives::{Address, AddressMap, StorageKey, StorageValue, B256};
use state::{Account, AccountId, AccountInfo, Bytecode};
use std::{cell::Cell, fmt, io};
use tokio::{
    runtime::{Handle, Runtime},
    task,
};

type Resume = AsyncResult<NonNull<Context<'static>>>;
type Yield = ();
type Complete<R> = AsyncResult<R>;
type DatabaseFiber<R> = Coroutine<Resume, Yield, Complete<R>, DefaultStack>;

const DEFAULT_STACK_SIZE: usize = 1024 * 1024;

/// Reusable async EVM fiber stack storage.
#[derive(Default)]
pub struct FiberStack {
    stack: Option<DefaultStack>,
}

impl Clone for FiberStack {
    #[inline]
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl fmt::Debug for FiberStack {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FiberStack").finish_non_exhaustive()
    }
}

impl FiberStack {
    #[inline]
    fn take_or_new(&mut self) -> AsyncResult<DefaultStack> {
        match self.stack.take() {
            Some(stack) => Ok(stack),
            None => DefaultStack::new(DEFAULT_STACK_SIZE).map_err(AsyncError::Io),
        }
    }

    #[inline]
    fn put(&mut self, stack: DefaultStack) {
        debug_assert!(self.stack.is_none());
        self.stack = Some(stack);
    }
}

thread_local! {
    static CURRENT: Cell<Option<NonNull<CurrentFiber>>> = const { Cell::new(None) };
}

/// Result type used by async database execution helpers.
pub type AsyncResult<T, E = Infallible> = Result<T, AsyncError<E>>;

/// Error returned by async database execution helpers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AsyncError<E = Infallible> {
    /// The async EVM fiber was cancelled before execution completed.
    #[error("async EVM execution was cancelled")]
    Cancelled,
    /// An async host operation was called outside an async EVM fiber.
    #[error("async host operation requires EVM async fiber execution")]
    NotOnFiber,
    /// Blocking async I/O was requested outside a supported Tokio runtime.
    #[error("async host operation requires a Tokio multi-thread runtime")]
    Runtime,
    /// Async fiber stack setup failed.
    #[error(transparent)]
    Io(io::Error),
    /// The wrapped operation returned an error.
    #[error(transparent)]
    Inner(#[from] E),
}

impl AsyncError {
    fn with_inner_error<E>(self) -> AsyncError<E> {
        match self {
            Self::Cancelled => AsyncError::Cancelled,
            Self::NotOnFiber => AsyncError::NotOnFiber,
            Self::Runtime => AsyncError::Runtime,
            Self::Io(error) => AsyncError::Io(error),
            Self::Inner(error) => match error {},
        }
    }
}

impl<E: DBErrorMarker> DBErrorMarker for AsyncError<E> {
    #[inline]
    fn is_fatal(&self) -> bool {
        match self {
            Self::Inner(error) => error.is_fatal(),
            _ => true,
        }
    }
}

struct CurrentFiber {
    suspend: NonNull<Yielder<Resume, Yield>>,
    future_cx: NonNull<Context<'static>>,
    cancelled: bool,
}

impl CurrentFiber {
    #[inline]
    fn context(&mut self) -> &mut Context<'_> {
        unsafe { restore_context_lifetime(self.future_cx.as_mut()) }
    }

    #[inline]
    fn suspend(&mut self) -> AsyncResult<()> {
        match unsafe { self.suspend.as_ref() }.suspend(()) {
            Ok(cx) => {
                self.future_cx = cx;
                Ok(())
            }
            Err(error) => {
                self.cancelled = true;
                Err(error)
            }
        }
    }

    #[inline]
    const fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

struct ResetCurrentFiber(Option<NonNull<CurrentFiber>>);

impl Drop for ResetCurrentFiber {
    fn drop(&mut self) {
        CURRENT.set(self.0);
    }
}

/// Runs `func` on a native fiber and awaits its completion.
///
/// Synchronous code running inside `func` may call [`block_on_current`] to wait for async host
/// operations without blocking the executor thread.
#[cfg(test)]
pub(crate) fn on_fiber_result<'a, R, E>(
    func: impl FnOnce() -> Result<R, E> + 'a,
) -> impl Future<Output = AsyncResult<R, E>> + Send + 'a
where
    R: Send + 'a,
    E: Send + 'a,
{
    OnFiber::new(func)
}

/// Runs `func` on a native fiber backed by a reusable EVM stack slot.
///
/// # Safety
///
/// `stack` must point to valid stack storage for the lifetime of the returned future. That storage
/// must not be accessed by anything else until the returned future is dropped.
pub unsafe fn on_fiber_result_with_stack<'a, R, E>(
    stack: NonNull<FiberStack>,
    func: impl FnOnce() -> Result<R, E> + 'a,
) -> impl Future<Output = AsyncResult<R, E>> + Send + 'a
where
    R: Send + 'a,
    E: Send + 'a,
{
    OnFiber::with_stack(stack, func)
}

#[cfg(test)]
pub(crate) fn on_fiber<'a, R>(
    func: impl FnOnce() -> R + 'a,
) -> impl Future<Output = AsyncResult<R>> + Send + 'a
where
    R: Send + 'a,
{
    on_fiber_result(move || Ok::<_, Infallible>(func()))
}

/// Runs `func` on a native fiber backed by a reusable EVM stack slot.
///
/// # Safety
///
/// See [`on_fiber_result_with_stack`].
pub unsafe fn on_fiber_with_stack<'a, R>(
    stack: NonNull<FiberStack>,
    func: impl FnOnce() -> R + 'a,
) -> impl Future<Output = AsyncResult<R>> + Send + 'a
where
    R: Send + 'a,
{
    unsafe { on_fiber_result_with_stack(stack, move || Ok::<_, Infallible>(func())) }
}

enum OnFiber<'a, R, E> {
    Running(FiberFuture<'a, Result<R, E>>),
    Error(Option<AsyncError>),
    Done,
}

impl<'a, R, E> OnFiber<'a, R, E> {
    #[cfg(test)]
    fn new(func: impl FnOnce() -> Result<R, E> + 'a) -> Self {
        Self::new_inner(None, func)
    }

    fn with_stack(stack: NonNull<FiberStack>, func: impl FnOnce() -> Result<R, E> + 'a) -> Self {
        Self::new_inner(Some(stack), func)
    }

    fn new_inner(
        stack: Option<NonNull<FiberStack>>,
        func: impl FnOnce() -> Result<R, E> + 'a,
    ) -> Self {
        match FiberFuture::new(stack, func) {
            Ok(fiber) => Self::Running(fiber),
            Err(error) => Self::Error(Some(error)),
        }
    }
}

impl<R, E> Future for OnFiber<'_, R, E> {
    type Output = AsyncResult<R, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this {
            Self::Running(fiber) => match Pin::new(fiber).poll(cx) {
                Poll::Ready(Ok(Ok(value))) => {
                    *this = Self::Done;
                    Poll::Ready(Ok(value))
                }
                Poll::Ready(Ok(Err(error))) => {
                    *this = Self::Done;
                    Poll::Ready(Err(AsyncError::Inner(error)))
                }
                Poll::Ready(Err(error)) => {
                    *this = Self::Done;
                    Poll::Ready(Err(error.with_inner_error()))
                }
                Poll::Pending => Poll::Pending,
            },
            Self::Error(error) => {
                let error = error
                    .take()
                    .expect("async EVM fiber error already returned");
                Poll::Ready(Err(error.with_inner_error()))
            }
            Self::Done => panic!("async EVM fiber polled after completion"),
        }
    }
}

struct FiberFuture<'a, R> {
    fiber: Option<DatabaseFiber<R>>,
    stack: Option<NonNull<FiberStack>>,
    _marker: PhantomData<&'a ()>,
}

// SAFETY: The future may move between polls, but the coroutine stack itself is heap allocated and
// is only resumed through `poll` with a fresh task context. Values that can remain on the coroutine
// stack across suspension are required to be `Send` by the blocking boundary.
unsafe impl<R: Send> Send for FiberFuture<'_, R> {}

impl<'a, R> FiberFuture<'a, R> {
    fn new(
        mut stack: Option<NonNull<FiberStack>>,
        func: impl FnOnce() -> R + 'a,
    ) -> AsyncResult<Self> {
        let fiber_stack = match &mut stack {
            Some(stack) => unsafe { stack.as_mut() }.take_or_new()?,
            None => DefaultStack::new(DEFAULT_STACK_SIZE).map_err(AsyncError::Io)?,
        };
        let body = move |suspend: &Yielder<Resume, Yield>, resume| {
            let future_cx = resume?;
            let mut current = CurrentFiber {
                suspend: NonNull::from(suspend),
                future_cx,
                cancelled: false,
            };
            let current = NonNull::from(&mut current);
            let previous = CURRENT.replace(Some(current));
            let _reset = ResetCurrentFiber(previous);
            Ok(func())
        };
        // SAFETY: The coroutine is stored inside `FiberFuture<'a, R>`, which is tied to the
        // borrowed state lifetime and dropped before those borrows can expire.
        let fiber = unsafe { Coroutine::with_stack_unchecked(fiber_stack, body) };
        Ok(Self {
            fiber: Some(fiber),
            stack,
            _marker: PhantomData,
        })
    }

    fn recycle_stack(&mut self) {
        let Some(fiber) = self.fiber.take() else {
            return;
        };
        debug_assert!(fiber.done());
        let stack = fiber.into_stack();
        if let Some(mut slot) = self.stack {
            unsafe { slot.as_mut() }.put(stack);
        }
    }
}

impl<R> Future for FiberFuture<'_, R> {
    type Output = AsyncResult<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let cx = NonNull::from(unsafe { change_context_lifetime(cx) });
        let fiber = this
            .fiber
            .as_mut()
            .expect("async EVM fiber polled after completion");
        match fiber.resume(Ok(cx)) {
            CoroutineResult::Return(result) => {
                this.recycle_stack();
                Poll::Ready(result)
            }
            CoroutineResult::Yield(()) => Poll::Pending,
        }
    }
}

impl<R> Drop for FiberFuture<'_, R> {
    fn drop(&mut self) {
        let Some(fiber) = self.fiber.as_mut() else {
            return;
        };
        if fiber.done() {
            self.recycle_stack();
        } else if matches!(
            fiber.resume(Err(AsyncError::Cancelled)),
            CoroutineResult::Yield(())
        ) {
            // SAFETY: Cancellation already gave the coroutine a chance to return normally. If it
            // yields again, the stack is no longer useful to this future.
            unsafe { fiber.force_reset() };
        } else {
            self.recycle_stack();
        }
    }
}

/// Polls `future` to completion from inside an async EVM fiber.
///
/// If `future` returns `Poll::Pending`, the current EVM fiber is suspended and the outer async EVM
/// future returns `Poll::Pending`. When the executor wakes and polls the outer future again, the
/// EVM fiber resumes and continues polling `future`.
///
/// # Errors
///
/// Returns [`AsyncError::NotOnFiber`] if called outside async EVM execution, or
/// [`AsyncError::Cancelled`] if the outer async EVM execution was dropped.
pub fn block_on_current<F: Future>(future: F) -> AsyncResult<F::Output> {
    let mut future = core::pin::pin!(future);
    loop {
        match with_current(|current| {
            if current.is_cancelled() {
                return Err(AsyncError::Cancelled);
            }
            let poll = future.as_mut().poll(current.context());
            if poll.is_pending() {
                current.suspend()?;
            }
            Ok(poll)
        })? {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => {}
        }
    }
}

fn current_tokio_handle() -> Option<Handle> {
    match Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::CurrentThread => None,
            _ => Some(handle),
        },
        Err(_) => None,
    }
}

fn block_on_handle<F>(handle: &Handle, future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    let should_use_block_in_place = Handle::try_current()
        .ok()
        .map(|current| {
            !matches!(
                current.runtime_flavor(),
                tokio::runtime::RuntimeFlavor::CurrentThread
            )
        })
        .unwrap_or(false);

    if should_use_block_in_place {
        task::block_in_place(move || handle.block_on(future))
    } else {
        handle.block_on(future)
    }
}

fn block_on_runtime<F>(runtime: Option<&Handle>, future: F) -> AsyncResult<F::Output>
where
    F: Future + Send,
    F::Output: Send,
{
    if CURRENT.get().is_some() {
        return block_on_current(future);
    }

    if let Some(runtime) = runtime {
        return Ok(block_on_handle(runtime, future));
    }

    Err(AsyncError::Runtime)
}

fn block_on_runtime_result<F, T, E>(runtime: Option<&Handle>, future: F) -> AsyncResult<T, E>
where
    F: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Send,
{
    match block_on_runtime(runtime, future).map_err(AsyncError::with_inner_error)? {
        Ok(value) => Ok(value),
        Err(error) => Err(AsyncError::Inner(error)),
    }
}

fn with_current<R>(f: impl FnOnce(&mut CurrentFiber) -> AsyncResult<R>) -> AsyncResult<R> {
    let mut current = CURRENT.get().ok_or(AsyncError::NotOnFiber)?;
    f(unsafe { current.as_mut() })
}

unsafe fn change_context_lifetime<'a>(cx: &'a mut Context<'_>) -> &'a mut Context<'static> {
    unsafe { core::mem::transmute::<&'a mut Context<'_>, &'a mut Context<'static>>(cx) }
}

unsafe fn restore_context_lifetime<'a>(cx: &'a mut Context<'static>) -> &'a mut Context<'a> {
    unsafe { core::mem::transmute::<&'a mut Context<'static>, &'a mut Context<'a>>(cx) }
}

/// The async EVM database interface.
///
/// Contains the same methods as [Database], but it returns [Future] type instead.
///
/// Use [AsyncDb] to provide [Database] implementation for a type that only implements this trait.
pub trait DatabaseAsync {
    /// The database error type.
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
        account_id: AccountId,
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

/// The async EVM database interface.
///
/// Contains the same methods as [DatabaseRef], but it returns [Future] type instead.
///
/// Use [AsyncDb] to provide [DatabaseRef] implementation for a type that only implements this trait.
pub trait DatabaseAsyncRef {
    /// The database error type.
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
        account_id: AccountId,
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

/// Adapter that exposes an async database through the synchronous [`Database`] interface.
#[derive(Debug)]
pub struct AsyncDb<T> {
    db: T,
    rt: Option<HandleOrRuntime>,
}

impl<T> AsyncDb<T> {
    /// Creates a new async database adapter.
    ///
    /// This captures the current Tokio runtime handle when one is available.
    #[inline]
    pub fn new(db: T) -> Self {
        Self {
            db,
            rt: current_tokio_handle().map(HandleOrRuntime::Handle),
        }
    }

    /// Creates a new async database adapter using the current Tokio runtime handle.
    ///
    /// Returns `None` if no Tokio runtime is available or the current runtime is current-threaded.
    #[inline]
    pub fn blocking(db: T) -> Option<Self> {
        Some(Self {
            db,
            rt: Some(HandleOrRuntime::Handle(current_tokio_handle()?)),
        })
    }

    /// Creates a new async database adapter with a Tokio runtime.
    #[inline]
    pub const fn with_runtime(db: T, runtime: Runtime) -> Self {
        Self {
            db,
            rt: Some(HandleOrRuntime::Runtime(runtime)),
        }
    }

    /// Creates a new async database adapter with a Tokio runtime handle.
    #[inline]
    pub const fn with_handle(db: T, handle: Handle) -> Self {
        Self {
            db,
            rt: Some(HandleOrRuntime::Handle(handle)),
        }
    }

    /// Returns the wrapped database.
    #[inline]
    pub const fn inner(&self) -> &T {
        &self.db
    }

    /// Returns the wrapped database mutably.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut T {
        &mut self.db
    }

    /// Consumes the adapter and returns the wrapped database.
    #[inline]
    pub fn into_inner(self) -> T {
        self.db
    }
}

impl<T: DatabaseAsync> Database for AsyncDb<T> {
    type Error = AsyncError<T::Error>;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let Self { db, rt } = self;
        block_on_runtime_result(
            rt.as_ref().map(HandleOrRuntime::handle),
            db.basic_async(address),
        )
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        let Self { db, rt } = self;
        block_on_runtime_result(
            rt.as_ref().map(HandleOrRuntime::handle),
            db.code_by_hash_async(code_hash),
        )
    }

    #[inline]
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let Self { db, rt } = self;
        block_on_runtime_result(
            rt.as_ref().map(HandleOrRuntime::handle),
            db.storage_async(address, index),
        )
    }

    #[inline]
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let Self { db, rt } = self;
        block_on_runtime_result(
            rt.as_ref().map(HandleOrRuntime::handle),
            db.storage_by_account_id_async(address, account_id, storage_key),
        )
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        let Self { db, rt } = self;
        block_on_runtime_result(
            rt.as_ref().map(HandleOrRuntime::handle),
            db.block_hash_async(number),
        )
    }
}

impl<T: DatabaseAsyncRef> DatabaseRef for AsyncDb<T> {
    type Error = AsyncError<T::Error>;

    #[inline]
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        block_on_runtime_result(
            self.rt.as_ref().map(HandleOrRuntime::handle),
            self.db.basic_async_ref(address),
        )
    }

    #[inline]
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        block_on_runtime_result(
            self.rt.as_ref().map(HandleOrRuntime::handle),
            self.db.code_by_hash_async_ref(code_hash),
        )
    }

    #[inline]
    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        block_on_runtime_result(
            self.rt.as_ref().map(HandleOrRuntime::handle),
            self.db.storage_async_ref(address, index),
        )
    }

    #[inline]
    fn storage_by_account_id_ref(
        &self,
        address: Address,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        block_on_runtime_result(
            self.rt.as_ref().map(HandleOrRuntime::handle),
            self.db
                .storage_by_account_id_async_ref(address, account_id, storage_key),
        )
    }

    #[inline]
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        block_on_runtime_result(
            self.rt.as_ref().map(HandleOrRuntime::handle),
            self.db.block_hash_async_ref(number),
        )
    }
}

impl<T: DatabaseAsync + DatabaseCommit> DatabaseCommit for AsyncDb<T> {
    #[inline]
    fn commit(&mut self, changes: AddressMap<Account>) {
        self.db.commit(changes);
    }

    #[inline]
    fn commit_iter(&mut self, changes: &mut dyn Iterator<Item = (Address, Account)>) {
        self.db.commit_iter(changes);
    }
}

/// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] to provide a [`Database`] implementation.
#[derive(Debug)]
pub struct WrapDatabaseAsync<T>(AsyncDb<T>);

impl<T> WrapDatabaseAsync<T> {
    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance.
    ///
    /// Returns `None` if no tokio runtime is available or if the current runtime is a current-thread runtime.
    #[inline]
    pub fn new(db: T) -> Option<Self> {
        AsyncDb::blocking(db).map(Self)
    }

    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime.
    ///
    /// Refer to [tokio::runtime::Builder] on how to create a runtime if you are in synchronous world.
    ///
    /// If you are already using something like [tokio::main], call [`WrapDatabaseAsync::new`] instead.
    #[inline]
    pub const fn with_runtime(db: T, runtime: Runtime) -> Self {
        Self(AsyncDb::with_runtime(db, runtime))
    }

    /// Wraps a [DatabaseAsync] or [DatabaseAsyncRef] instance, with a runtime handle.
    ///
    /// This generally allows you to pass any valid runtime handle, refer to [tokio::runtime::Handle] on how
    /// to obtain a handle.
    ///
    /// If you are already in asynchronous world, like [tokio::main], use [`WrapDatabaseAsync::new`] instead.
    #[inline]
    pub const fn with_handle(db: T, handle: Handle) -> Self {
        Self(AsyncDb::with_handle(db, handle))
    }

    /// Returns the wrapped database.
    #[inline]
    pub const fn inner(&self) -> &T {
        self.0.inner()
    }

    /// Returns the wrapped database mutably.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut T {
        self.0.inner_mut()
    }

    /// Consumes the adapter and returns the wrapped database.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: DatabaseAsync> Database for WrapDatabaseAsync<T> {
    type Error = AsyncError<T::Error>;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic(address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash(code_hash)
    }

    #[inline]
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0.storage(address, index)
    }

    #[inline]
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0
            .storage_by_account_id(address, account_id, storage_key)
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash(number)
    }
}

impl<T: DatabaseAsyncRef> DatabaseRef for WrapDatabaseAsync<T> {
    type Error = AsyncError<T::Error>;

    #[inline]
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic_ref(address)
    }

    #[inline]
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash_ref(code_hash)
    }

    #[inline]
    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0.storage_ref(address, index)
    }

    #[inline]
    fn storage_by_account_id_ref(
        &self,
        address: Address,
        account_id: AccountId,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0
            .storage_by_account_id_ref(address, account_id, storage_key)
    }

    #[inline]
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash_ref(number)
    }
}

impl<T: DatabaseAsync + DatabaseCommit> DatabaseCommit for WrapDatabaseAsync<T> {
    #[inline]
    fn commit(&mut self, changes: AddressMap<Account>) {
        self.0.commit(changes);
    }

    #[inline]
    fn commit_iter(&mut self, changes: &mut dyn Iterator<Item = (Address, Account)>) {
        self.0.commit_iter(changes);
    }
}

// Hold a tokio runtime handle or full runtime.
#[derive(Debug)]
enum HandleOrRuntime {
    Handle(Handle),
    Runtime(Runtime),
}

impl HandleOrRuntime {
    #[inline]
    fn handle(&self) -> &Handle {
        match self {
            Self::Handle(handle) => handle,
            Self::Runtime(runtime) => runtime.handle(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{block_on_current, on_fiber, AsyncDb, AsyncError, DatabaseAsync};
    use crate::Database;
    use core::{convert::Infallible, fmt, future::Future, pin::Pin, task::Poll};
    use primitives::{Address, StorageKey, StorageValue, B256};
    use state::{AccountInfo, Bytecode};
    use std::task::{Context, Waker};

    #[test]
    fn block_on_requires_fiber() {
        assert!(matches!(
            block_on_current(core::future::ready(())),
            Err(AsyncError::NotOnFiber)
        ));
    }

    #[test]
    fn fiber_suspends_and_resumes_pending_future() {
        let mut state = 1;
        let mut future = core::pin::pin!(on_fiber(|| {
            state += block_on_current(PendingOnce { pending: true }).unwrap();
            state
        }));
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
        assert!(matches!(future.as_mut().poll(&mut cx), Poll::Ready(Ok(3))));
    }

    #[test]
    fn fiber_reuses_stack_slot() {
        let mut stack = super::FiberStack::default();
        let stack_ptr = core::ptr::NonNull::from(&mut stack);

        poll_ready(unsafe {
            super::on_fiber_result_with_stack(stack_ptr, || Ok::<_, Infallible>(1))
        })
        .unwrap();
        assert!(stack.stack.is_some());
        poll_ready(unsafe {
            super::on_fiber_result_with_stack(stack_ptr, || Ok::<_, Infallible>(2))
        })
        .unwrap();
        assert!(stack.stack.is_some());
    }

    #[test]
    fn async_database_adapts_to_database() {
        let mut db = AsyncDb::new(TestDb);

        let value = poll_ready(on_fiber(|| {
            Database::storage(&mut db, Address::ZERO, StorageKey::from(7)).unwrap()
        }))
        .unwrap();

        assert_eq!(value, StorageValue::from(9));
    }

    #[test]
    fn async_database_suspends_until_ready() {
        let mut db = AsyncDb::new(PendingDb { pending: true });
        let mut future = core::pin::pin!(on_fiber(|| {
            Database::storage(&mut db, Address::ZERO, StorageKey::from(7)).unwrap()
        }));
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
        assert!(
            matches!(future.as_mut().poll(&mut cx), Poll::Ready(Ok(value)) if value == StorageValue::from(9))
        );
    }

    #[test]
    fn async_database_returns_database_error() {
        let mut db = AsyncDb::new(FailingDb);
        let result = poll_ready(on_fiber(|| {
            Database::storage(&mut db, Address::ZERO, StorageKey::from(7))
        }));

        assert!(matches!(result, Ok(Err(AsyncError::Inner(TestError)))));
    }

    #[test]
    fn synchronous_database_blocks_with_current_tokio_runtime() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let mut db = AsyncDb::new(TokioDb);

        let value = Database::storage(&mut db, Address::ZERO, StorageKey::from(7)).unwrap();

        assert_eq!(value, StorageValue::from(9));
    }

    #[test]
    fn blocking_constructor_uses_current_tokio_runtime() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let mut db = AsyncDb::blocking(TokioDb).unwrap();

        let value = Database::storage(&mut db, Address::ZERO, StorageKey::from(7)).unwrap();

        assert_eq!(value, StorageValue::from(9));
    }

    #[test]
    fn synchronous_database_blocks_with_stored_tokio_handle() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut db = AsyncDb::with_handle(TokioDb, runtime.handle().clone());

        let value = Database::storage(&mut db, Address::ZERO, StorageKey::from(7)).unwrap();

        assert_eq!(value, StorageValue::from(9));
    }

    #[test]
    fn synchronous_database_requires_runtime_handle() {
        let mut db = AsyncDb::new(TestDb);

        let result = Database::storage(&mut db, Address::ZERO, StorageKey::from(7));

        assert!(matches!(result, Err(AsyncError::Runtime)));
    }

    #[test]
    fn dropping_fiber_cancels_blocked_future() {
        let mut saw_cancel = false;
        {
            let mut future = core::pin::pin!(on_fiber(|| {
                saw_cancel = matches!(block_on_current(PendingForever), Err(AsyncError::Cancelled));
            }));
            let waker = Waker::noop();
            let mut cx = Context::from_waker(waker);

            assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
        }
        assert!(saw_cancel);
    }

    fn poll_ready<F: Future + Send>(future: F) -> F::Output {
        let mut future = core::pin::pin!(future);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("future unexpectedly pending"),
        }
    }

    struct PendingOnce {
        pending: bool,
    }

    impl Future for PendingOnce {
        type Output = i32;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.pending {
                self.pending = false;
                Poll::Pending
            } else {
                Poll::Ready(2)
            }
        }
    }

    struct PendingForever;

    impl Future for PendingForever {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }

    struct TestDb;

    impl DatabaseAsync for TestDb {
        type Error = Infallible;

        async fn basic_async(
            &mut self,
            _address: Address,
        ) -> Result<Option<AccountInfo>, Self::Error> {
            Ok(None)
        }

        async fn code_by_hash_async(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
            Ok(Bytecode::default())
        }

        async fn storage_async(
            &mut self,
            _address: Address,
            _index: StorageKey,
        ) -> Result<StorageValue, Self::Error> {
            Ok(StorageValue::from(9))
        }

        async fn block_hash_async(&mut self, _number: u64) -> Result<B256, Self::Error> {
            Ok(B256::ZERO)
        }
    }

    struct PendingDb {
        pending: bool,
    }

    impl DatabaseAsync for PendingDb {
        type Error = Infallible;

        async fn basic_async(
            &mut self,
            _address: Address,
        ) -> Result<Option<AccountInfo>, Self::Error> {
            Ok(None)
        }

        async fn code_by_hash_async(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
            Ok(Bytecode::default())
        }

        async fn storage_async(
            &mut self,
            _address: Address,
            _index: StorageKey,
        ) -> Result<StorageValue, Self::Error> {
            PendingStorage {
                pending: &mut self.pending,
            }
            .await;
            Ok(StorageValue::from(9))
        }

        async fn block_hash_async(&mut self, _number: u64) -> Result<B256, Self::Error> {
            Ok(B256::ZERO)
        }
    }

    struct PendingStorage<'a> {
        pending: &'a mut bool,
    }

    impl Future for PendingStorage<'_> {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if *self.pending {
                *self.pending = false;
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    struct FailingDb;

    impl DatabaseAsync for FailingDb {
        type Error = TestError;

        async fn basic_async(
            &mut self,
            _address: Address,
        ) -> Result<Option<AccountInfo>, Self::Error> {
            Ok(None)
        }

        async fn code_by_hash_async(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
            Ok(Bytecode::default())
        }

        async fn storage_async(
            &mut self,
            _address: Address,
            _index: StorageKey,
        ) -> Result<StorageValue, Self::Error> {
            Err(TestError)
        }

        async fn block_hash_async(&mut self, _number: u64) -> Result<B256, Self::Error> {
            Ok(B256::ZERO)
        }
    }

    struct TokioDb;

    impl DatabaseAsync for TokioDb {
        type Error = Infallible;

        async fn basic_async(
            &mut self,
            _address: Address,
        ) -> Result<Option<AccountInfo>, Self::Error> {
            tokio::task::yield_now().await;
            Ok(None)
        }

        async fn code_by_hash_async(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
            tokio::task::yield_now().await;
            Ok(Bytecode::default())
        }

        async fn storage_async(
            &mut self,
            _address: Address,
            _index: StorageKey,
        ) -> Result<StorageValue, Self::Error> {
            tokio::task::yield_now().await;
            Ok(StorageValue::from(9))
        }

        async fn block_hash_async(&mut self, _number: u64) -> Result<B256, Self::Error> {
            tokio::task::yield_now().await;
            Ok(B256::ZERO)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("storage read failed")
        }
    }

    impl core::error::Error for TestError {}

    impl crate::DBErrorMarker for TestError {}
}
