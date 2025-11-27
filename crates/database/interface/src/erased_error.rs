//! Erased error type.

/// Erased error type.
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct ErasedError(Box<dyn core::error::Error + Send + Sync + 'static>);

impl ErasedError {
    /// Creates a new erased error.
    pub fn new(error: impl core::error::Error + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }

    /// Consumes the erased error and returns the inner error.
    #[inline]
    pub fn into_inner(self) -> Box<dyn core::error::Error + Send + Sync + 'static> {
        self.0
    }
}
