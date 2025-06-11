//! Local context trait [`LocalContextTr`] and related types.
use core::{
    cell::{Ref, RefCell},
    fmt,
    mem::ManuallyDrop,
    ops::Range,
};
use primitives::{Bytes, B256, U256};
use std::{rc::Rc, vec::Vec};

/// Pool for reusable values that can be pulled and returned.
#[derive(Debug)]
pub struct LocalPool<T>(Rc<RefCell<Vec<T>>>);

impl<T> Clone for LocalPool<T> {
    fn clone(&self) -> Self {
        LocalPool(self.0.clone())
    }
}

impl<T> Default for LocalPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LocalPool<T> {
    /// Creates a new local pool.
    pub fn new() -> Self {
        LocalPool(Rc::new(RefCell::new(Vec::new())))
    }

    /// Pulls a value from the pool, or initializes a new one using the provided function.
    pub fn pull(&self, init: impl FnOnce() -> T) -> PoolGuard<T> {
        let pool = self.clone();
        let val = match pool.0.borrow_mut().pop() {
            Some(val) => val,
            None => init(),
        };
        PoolGuard::new(pool, val)
    }

    /// Attaches a value back to the pool.
    pub fn attach(&self, val: T) {
        self.0.borrow_mut().push(val);
    }
}

/// A pooled value that will be returned to the pool when dropped.
#[repr(C)] // Ensures `Deref` is zero-cost.
pub struct PoolGuard<T> {
    inner: ManuallyDrop<T>,
    pool: ManuallyDrop<LocalPool<T>>,
}

impl<T: fmt::Debug> fmt::Debug for PoolGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> core::ops::Deref for PoolGuard<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for PoolGuard<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for PoolGuard<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for PoolGuard<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|val| Self::new(LocalPool::new(), val))
    }
}

impl<T> PoolGuard<T> {
    fn new(pool: LocalPool<T>, inner: T) -> Self {
        Self {
            pool: ManuallyDrop::new(pool),
            inner: ManuallyDrop::new(inner),
        }
    }

    /// Returns the inner value and consumes the guard.
    pub fn into_inner(mut self) -> (LocalPool<T>, T) {
        unsafe { self.take() }
    }

    unsafe fn take(&mut self) -> (LocalPool<T>, T) {
        let pool = unsafe { ManuallyDrop::take(&mut self.pool) };
        let inner = unsafe { ManuallyDrop::take(&mut self.inner) };
        (pool, inner)
    }
}

impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        let (pool, val) = unsafe { self.take() };
        // Avoid attaching if this instance contains the only reference to the pool.
        if Rc::strong_count(&pool.0) > 1 {
            pool.attach(val);
        }
    }
}

/// Local context used for caching initcode from Initcode transactions.
pub trait LocalContextTr {
    /// Get the local context
    fn insert_initcodes(&mut self, initcodes: &[Bytes]);

    /// Get validated initcode by hash. if initcode is not validated it is assumed
    /// that validation is going to be performed inside this function.
    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes>;

    /// Pulls a stack from the pool.
    fn pull_stack(&self) -> PoolGuard<Vec<U256>>;

    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    fn shared_memory_buffer(&self) -> &Rc<RefCell<Vec<u8>>>;

    /// Slice of the shared memory buffer returns None if range is not valid or buffer can't be borrowed.
    fn shared_memory_buffer_slice(&self, range: Range<usize>) -> Option<Ref<'_, [u8]>> {
        let buffer = self.shared_memory_buffer();
        buffer.borrow().get(range.clone())?;
        Some(Ref::map(buffer.borrow(), |b| {
            b.get(range).unwrap_or_default()
        }))
    }

    /// Clear the local context.
    fn clear(&mut self);
}
