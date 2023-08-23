use revm_interpreter::primitives::{AccountInfo, HashMap, U256};

use super::{AccountStatus, BundleAccount, StorageWithOriginalValues};

/// Assumption is that Revert can return full state from any future state to any past state.
///
/// It is created when new account state is applied to old account state.
/// And it is used to revert new account state to the old account state.
///
/// AccountRevert is structured in this way as we need to save it inside database.
/// And we need to be able to read it from database.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct AccountRevert {
    pub account: AccountInfoRevert,
    pub storage: HashMap<U256, RevertToSlot>,
    pub previous_status: AccountStatus,
    pub wipe_storage: bool,
}

impl AccountRevert {
    /// Very similar to new_selfdestructed but it will add additional zeros (RevertToSlot::Destroyed)
    /// for the storage that are set if account is again created.
    pub fn new_selfdestructed_again(
        status: AccountStatus,
        account: AccountInfoRevert,
        mut previous_storage: StorageWithOriginalValues,
        updated_storage: StorageWithOriginalValues,
    ) -> Self {
        // Take present storage values as the storages that we are going to revert to.
        // As those values got destroyed.
        let mut previous_storage: HashMap<U256, RevertToSlot> = previous_storage
            .drain()
            .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
            .collect();
        for (key, _) in updated_storage {
            previous_storage
                .entry(key)
                .or_insert(RevertToSlot::Destroyed);
        }
        AccountRevert {
            account,
            storage: previous_storage,
            previous_status: status,
            wipe_storage: false,
        }
    }

    /// Create revert for states that were before selfdestruct.
    pub fn new_selfdestructed_from_bundle(
        account_info_revert: AccountInfoRevert,
        bundle_account: &mut BundleAccount,
        updated_storage: &StorageWithOriginalValues,
    ) -> Option<Self> {
        match bundle_account.status {
            AccountStatus::InMemoryChange
            | AccountStatus::Changed
            | AccountStatus::LoadedEmptyEIP161
            | AccountStatus::Loaded => {
                let mut ret = AccountRevert::new_selfdestructed_again(
                    bundle_account.status,
                    account_info_revert,
                    bundle_account.storage.drain().collect(),
                    updated_storage.clone(),
                );
                ret.wipe_storage = true;
                Some(ret)
            }
            _ => None,
        }
    }

    /// Create new selfdestruct revert.
    pub fn new_selfdestructed(
        status: AccountStatus,
        account: AccountInfoRevert,
        mut storage: StorageWithOriginalValues,
    ) -> Self {
        // Zero all present storage values and save present values to AccountRevert.
        let previous_storage = storage
            .iter_mut()
            .map(|(key, value)| {
                // take previous value and set ZERO as storage got destroyed.
                (*key, RevertToSlot::Some(value.present_value))
            })
            .collect();

        Self {
            account,
            storage: previous_storage,
            previous_status: status,
            wipe_storage: true,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum AccountInfoRevert {
    #[default]
    /// Nothing changed
    DoNothing,
    /// Account was created and on revert we need to remove it with all storage.
    DeleteIt,
    /// Account was changed and on revert we need to put old state.
    RevertTo(AccountInfo),
}

/// So storage can have multiple types:
/// * Zero, on revert remove plain state.
/// * Value, on revert set this value
/// * Destroyed, should be removed on revert but on Revert set it as zero.
///
/// Note: It is completely different state if Storage is Zero or Some or if Storage was
/// Destroyed. Because if it is destroyed, previous values can be found in database or it can be zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevertToSlot {
    Some(U256),
    Destroyed,
}

impl RevertToSlot {
    pub fn to_previous_value(self) -> U256 {
        match self {
            RevertToSlot::Some(value) => value,
            RevertToSlot::Destroyed => U256::ZERO,
        }
    }
}
