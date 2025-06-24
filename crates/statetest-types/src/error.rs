use revm::primitives::B256;
use thiserror::Error;

/// Errors that can occur during test setup and execution
#[derive(Debug, Error)]
pub enum TestError {
    /// Unknown private key.
    #[error("unknown private key: {0:?}")]
    UnknownPrivateKey(B256),
    /// Invalid transaction type.
    #[error("invalid transaction type")]
    InvalidTransactionType,
    /// Unexpected exception.
    #[error("unexpected exception: got {got_exception:?}, expected {expected_exception:?}")]
    UnexpectedException {
        /// Expected exception.
        expected_exception: Option<String>,
        /// Got exception.
        got_exception: Option<String>,
    },
}
