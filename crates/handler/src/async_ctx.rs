use async_trait::async_trait;
use context_interface::journaled_state::{AccountLoad, StateLoad, TransferError};
use context_interface::{context::ContextTr, journaled_state::JournalTr};
use database_interface::Database;
use primitives::{Address, U256};
use state::Account;

/// Async façade for common journal/database operations that are currently polled
/// synchronously by `poll_to_ready`.  In later patches every call-site will be
/// migrated to these `async fn`s and the dummy waker helper can be removed.
#[async_trait(?Send)]
pub trait AsyncCtx {
    /// Low-level database / journal error returned by the underlying storage.
    type Error;

    /// Ensure the account is present in the journal, loading it from the DB if
    /// necessary, but do **not** mark it as warm.  This is a thin wrapper around
    /// `JournalTr::load_account_delegated`.
    async fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, Self::Error>;

    /// Load the full mutable account into the journal and mark it warm.
    async fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, Self::Error>;

    /// Transfer a balance between two accounts, returning a `TransferError` if
    /// the operation is invalid.
    async fn transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<Option<TransferError>, Self::Error>;
}

#[async_trait(?Send)]
impl<CTX> AsyncCtx for CTX
where
    CTX: ContextTr,
    // No Send bounds required because Handler/Frame are `?Send`.
    CTX::Journal: JournalTr,
{
    type Error = <<CTX::Journal as JournalTr>::Database as Database>::Error;

    async fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, Self::Error> {
        self.journal_mut().load_account_delegated(address).await
    }

    async fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, Self::Error> {
        self.journal_mut().load_account(address).await
    }

    async fn transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<Option<TransferError>, Self::Error> {
        self.journal_mut().transfer(from, to, value).await
    }
}
