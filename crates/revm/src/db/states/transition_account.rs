use super::{BundleAccount, PlainAccount, Storage};
use crate::db::AccountStatus;
use revm_interpreter::primitives::{AccountInfo, HashMap};

/// Account Created when EVM state is merged to cache state.
/// And it is send to Block state.
///
/// It is used when block state gets merged to bundle state to
/// create needed Reverts.
#[derive(Clone, Debug, Default)]
pub struct TransitionAccount {
    pub info: Option<AccountInfo>,
    pub status: AccountStatus,
    /// Previous account info is needed for account that got initialy loaded.
    /// Initialy loaded account are not present inside bundle and are needed
    /// to generate Reverts.
    pub previous_info: Option<AccountInfo>,
    /// Mostly needed when previous status Loaded/LoadedEmpty.
    pub previous_status: AccountStatus,
    /// Storage contains both old and new account
    pub storage: Storage,
}

impl TransitionAccount {
    /// Update new values of transition. Dont override old values
    /// both account info and old storages need to be left intact.
    pub fn update(&mut self, other: Self) {
        self.info = other.info.clone();
        self.status = other.status;

        // update changed values to this transition.
        for (key, slot) in other.storage.into_iter() {
            self.storage.entry(key).or_insert(slot).present_value = slot.present_value;
        }
    }

    /// Set previous values of transition. Override old values.
    pub fn update_previous(
        &mut self,
        info: Option<AccountInfo>,
        status: AccountStatus,
        storage: Storage,
    ) {
        self.previous_info = info;
        self.previous_status = status;

        // update original value of storage.
        for (key, slot) in storage.into_iter() {
            self.storage.entry(key).or_insert(slot).original_value = slot.original_value;
        }
    }

    /// Return previous account without any storage set.
    pub fn previous_bundle_account(&self) -> BundleAccount {
        BundleAccount {
            account: self.previous_info.as_ref().map(|info| PlainAccount {
                info: info.clone(),
                storage: HashMap::new(),
            }),
            status: self.previous_status,
        }
    }
}
