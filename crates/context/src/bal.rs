use core::sync::atomic::AtomicBool;
use std::sync::Arc;

///Block access index (0 for pre-execution, 1..n for transactions, n+1 for post-execution)
pub type BalIndex = u64;

/// Contains BalIndex and storage value. AtomicBool for second approch.
pub type BalWrites<T> = [(BalIndex, T)];

/// BalValue of already created BAL.
///
/// BalRead will fetch value from DB while BalReadWithValue will use this value later
/// in execution if some other tx needs it.
///
/// For BalWrite we need to know this transaction BalIndex so we could fetch proper value.
#[derive(Debug, Clone)]
pub struct CompiledBal<T> {
    /// Value already read from db.
    read_value: T,
    /// It is behind Arc as it is shared between transactions.
    writes: Arc<BalWrites<T>>,
    /// Helps access BalWrites faster as we only search for BalIndex after last_index of writes.
    /// We can linearly search for next index
    last_index: Option<usize>,
}
// Database

// fetches all accounts and storages from DB. Prios the requested accounts and storages to not block execution.
// READS ARE IMPORTANT AND BLOCKING.

// Database can return ERROR if not part of BAL. Database decides the way how it is executed.
//

/*


let evm = emv.build().evm().with_bal(Bal::new());

evm.transact_bal(tx);

db.fetch_account() -> read Account and its storage in one go!

db.fetch_storage() -> called only if storage is not part of BAL.


*/

// Would assume that loading happens from DB.

/*

Ordinary execution where we know bal


*/

/// USe to store values
#[derive(Debug, Clone)]
pub struct BalBuilder<T> {
    is_read: bool,
    bal_index: u64,
    writes: T,
}
#[derive(Debug, Clone)]
pub struct Bal<T> {
    items: Option<CompiledBal<T>>,
    builder: BalBuilder<T>,
}
