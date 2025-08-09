//! State management and tracking for the EVM.

/// Account status tracking.
pub mod account_status;
/// Bundle account representation.
pub mod bundle_account;
/// Bundle state management.
pub mod bundle_state;
/// Cache state implementation.
pub mod cache;
/// Cache account representation.
pub mod cache_account;
/// State changeset tracking.
pub mod changes;
/// Plain account representation.
pub mod plain_account;
/// State revert tracking.
pub mod reverts;
/// Main state implementation.
pub mod state;
/// State builder utilities.
pub mod state_builder;
/// Transition account representation.
pub mod transition_account;
/// Transition state management.
pub mod transition_state;

/// Account status for Block and Bundle states.
pub use account_status::AccountStatus;
pub use bundle_account::BundleAccount;
pub use bundle_state::{BundleBuilder, BundleState, OriginalValuesKnown};
pub use cache::CacheState;
pub use cache_account::CacheAccount;
pub use changes::{PlainStateReverts, PlainStorageChangeset, PlainStorageRevert, StateChangeset};
pub use plain_account::{PlainAccount, StorageSlot, StorageWithOriginalValues};
pub use reverts::{AccountRevert, RevertToSlot};
pub use state::{DBBox, State, StateDBBox};
pub use state_builder::StateBuilder;
pub use transition_account::TransitionAccount;
pub use transition_state::TransitionState;
