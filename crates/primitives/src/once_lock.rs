//! `OnceLock` abstraction that uses [`std::sync::OnceLock`] when available, once_cell otherwise.

#[cfg(not(feature = "std"))]
mod no_std_impl {
    use once_cell::race::OnceBox;
    use std::boxed::Box;

    /// A thread-safe cell which can be written to only once.
    #[derive(Debug)]
    pub struct OnceLock<T> {
        inner: OnceBox<T>,
    }

    impl<T> Default for OnceLock<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> OnceLock<T> {
        /// Creates a new empty OnceLock.
        #[inline]
        pub const fn new() -> Self {
            Self {
                inner: OnceBox::new(),
            }
        }

        /// Gets the contents of the OnceLock, initializing it if necessary.
        #[inline]
        pub fn get_or_init<F>(&self, f: F) -> &T
        where
            F: FnOnce() -> T,
            T: Into<Box<T>>,
        {
            self.inner.get_or_init(|| f().into())
        }

        /// Gets the contents of the OnceLock, returning None if it is not initialized.
        #[inline]
        pub fn get(&self) -> Option<&T> {
            self.inner.get()
        }
    }
}

#[cfg(feature = "std")]
use once_cell as _;
#[cfg(feature = "std")]
pub use std::sync::OnceLock;

#[cfg(not(feature = "std"))]
pub use no_std_impl::OnceLock;
