use revm_interpreter::primitives::{AccountInfo};

use crate::db::{AccountStatus};

use super::Storage;



/// Account Created when EVM state is merged to cache state.
/// And it is send to Block state.
/// 
/// It is used when block state gets merged to bundle state to
/// create needed Reverts.
pub struct TransitionAccount {
    pub info: Option<AccountInfo>,
    pub status: AccountStatus,
    /// Previous account info is needed for account that got initialy loaded.
    /// Initialu loaded account are not present inside bundle and are needed
    /// to generate Reverts.
    pub previous_info: Option<AccountInfo>,
    /// Mostly needed when previous status Loaded/LoadedEmpty.
    pub previous_status: AccountStatus,
    /// Storage contains both old and new account
    pub storage: Storage,
}
