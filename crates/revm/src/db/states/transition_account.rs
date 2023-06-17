use super::{AccountRevert, BundleAccount, Storage};
use crate::db::AccountStatus;
use revm_interpreter::primitives::{AccountInfo, Bytecode, B256};

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
    /// Return new contract bytecode if it is changed or newly created.
    pub fn has_new_contract(&self) -> Option<(B256, &Bytecode)> {
        let present_new_codehash = self.info.as_ref().map(|info| &info.code_hash);
        let previous_codehash = self.previous_info.as_ref().map(|info| &info.code_hash);
        if present_new_codehash != previous_codehash {
            return self
                .info
                .as_ref()
                .and_then(|info| info.code.as_ref().map(|c| (info.code_hash, c)));
        }
        None
    }

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

    /// Consume Self and create account revert from it.
    pub fn create_revert(self) -> Option<AccountRevert> {
        let mut previous_account = self.original_bundle_account();
        previous_account.update_and_create_revert(self)
    }

    /// Present bundle account
    pub fn present_bundle_account(&self) -> BundleAccount {
        BundleAccount {
            info: self.info.clone(),
            original_info: self.previous_info.clone(),
            storage: self.storage.clone(),
            status: self.status,
        }
    }

    /// Original bundle account
    pub fn original_bundle_account(&self) -> BundleAccount {
        BundleAccount {
            info: self.previous_info.clone(),
            original_info: self.previous_info.clone(),
            storage: Storage::new(),
            status: self.previous_status,
        }
    }
}
