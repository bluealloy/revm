
use revm_interpreter::primitives::{AccountInfo, HashMap, U256};

use super::{AccountStatus, PlainAccount};

/// Assumption is that Revert can return full state from any future state to any past state.
///
/// It is created when new account state is applied to old account state.
/// And it is used to revert new account state to the old account state.
///
/// RevertAccountState is structured in this way as we need to save it inside database.
/// And we need to be able to read it from database.
#[derive(Clone, Default)]
pub struct RevertAccountState {
    pub account: Option<AccountInfo>,
    pub storage: HashMap<U256, RevertToSlot>,
    pub original_status: AccountStatus,
}

/// So storage can have multiple types:
/// * Zero, on revert remove plain state.
/// * Value, on revert set this value
/// * Destroyed, IF it is not present already in changeset set it to zero.
///     on remove it from plainstate.
///
/// BREAKTHROUGHT: It is completely different state if Storage is Zero or Some or if Storage was
/// Destroyed. Because if it is destroyed, previous values can be found in database or can be zero.
#[derive(Clone)]
pub enum RevertToSlot {
    Some(U256),
    Destroyed,
}