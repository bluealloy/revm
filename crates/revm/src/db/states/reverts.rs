use revm_interpreter::primitives::{AccountInfo, HashMap, U256};

use super::{AccountStatus, BundleAccount, Storage};

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
    ///
    /// Example is of going from New (state: 1: 10) -> DestroyedNew (2:10)
    /// Revert of that needs to be list of key previous values.
    /// [1:10,2:0]
    pub fn new_selfdestructed_again(
        status: AccountStatus,
        account: AccountInfo,
        mut previous_storage: Storage,
        updated_storage: Storage,
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
            account: AccountInfoRevert::RevertTo(account),
            storage: previous_storage,
            previous_status: status,
            wipe_storage: true,
        }
    }

    /// Missing update is for Destroyed,DestroyedAgain,DestroyedNew,DestroyedChange.
    /// as those update are different between each other.
    /// It updated from state before destroyed. And that is NewChanged,New,Changed,LoadedEmptyEIP161.
    /// take a note that is not updating LoadedNotExisting.
    pub fn new_selfdestructed_from_bundle(
        bundle_account: &mut BundleAccount,
        updated_storage: &Storage,
    ) -> Option<Self> {
        match bundle_account.status {
            AccountStatus::InMemoryChange
            | AccountStatus::Changed
            | AccountStatus::LoadedEmptyEIP161 => Some(AccountRevert::new_selfdestructed_again(
                bundle_account.status,
                bundle_account.info.clone().unwrap_or_default(),
                bundle_account.storage.drain().collect(),
                updated_storage.clone(),
            )),
            _ => None,
        }
    }

    /// Create new selfdestruct revert.
    pub fn new_selfdestructed(
        status: AccountStatus,
        account: AccountInfo,
        mut storage: Storage,
    ) -> Self {
        // Zero all present storage values and save present values to AccountRevert.
        let previous_storage = storage
            .iter_mut()
            .map(|(key, value)| {
                // take previous value and set ZERO as storage got destroyed.
                let previous_value = core::mem::take(&mut value.present_value);
                (*key, RevertToSlot::Some(previous_value))
            })
            .collect();

        Self {
            account: AccountInfoRevert::RevertTo(account),
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
/// * Destroyed, IF it is not present already in changeset set it to zero.
///     on remove it from plainstate.
///
/// BREAKTHROUGHT: It is completely different state if Storage is Zero or Some or if Storage was
/// Destroyed. Because if it is destroyed, previous values can be found in database or can be zero.
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
