use super::{
    reverts::AccountInfoRevert, AccountRevert, AccountStatus, RevertToSlot, Storage,
    TransitionAccount,
};
use revm_interpreter::primitives::{AccountInfo, StorageSlot, U256};
use revm_precompile::HashMap;

/// Account information focused on creating of database changesets
/// and Reverts.
///
/// Status is needed to know from what state we are applying the TransitionAccount.
///
/// Original account info is needed to know if there was a change.
/// Same thing for storage where original.
///
/// On selfdestruct storage original value should be ignored.
#[derive(Clone, Debug)]
pub struct BundleAccount {
    pub info: Option<AccountInfo>,
    pub original_info: Option<AccountInfo>,
    /// Contain both original and present state.
    /// When extracting changeset we compare if original value is different from present value.
    /// If it is different we add it to changeset.
    ///
    /// If Account was destroyed we ignore original value and comprate present state with U256::ZERO.
    pub storage: Storage,
    pub status: AccountStatus,
}

impl BundleAccount {
    pub fn storage_slot(&self, slot: U256) -> Option<U256> {
        self.storage.get(&slot).map(|s| s.present_value)
    }

    /// Fetch account info if it exist.
    pub fn account_info(&self) -> Option<AccountInfo> {
        self.info.clone()
    }

    /// Was this account destroyed.
    pub fn was_destroyed(&self) -> bool {
        self.status.was_destroyed()
    }

    /// Return true of account info was changed.
    pub fn is_info_changed(&self) -> bool {
        self.info != self.original_info
    }

    /// Return true if contract was changed
    pub fn is_contract_changed(&self) -> bool {
        self.info.as_ref().map(|a| a.code_hash) != self.original_info.as_ref().map(|a| a.code_hash)
    }

    /// Revert account to previous state and return true if account can be removed.
    pub fn revert(&mut self, revert: AccountRevert) -> bool {
        match revert.account {
            AccountInfoRevert::DoNothing => (),
            AccountInfoRevert::DeleteIt => {
                self.info = None;
                self.status = revert.original_status;
                self.storage = HashMap::new();
                return true;
            }
            AccountInfoRevert::RevertTo(info) => self.info = Some(info),
        };
        self.status = revert.original_status;
        // revert stoarge
        for (key, slot) in revert.storage {
            match slot {
                RevertToSlot::Some(value) => {
                    // Dont overwrite original values if present
                    // if storage is not present set original values as currect value.
                    self.storage
                        .entry(key)
                        .or_insert(StorageSlot::new_cleared_value(value))
                        .present_value = value;
                }
                RevertToSlot::Destroyed => {
                    // if it was destroyed this means that storage was created and we need to remove it.
                    self.storage.remove(&key);
                }
            }
        }
        false
    }

    /// Update to new state and generate AccountRevert that if applied to new state will
    /// revert it to previous state. If not revert is present, update is noop.
    pub fn update_and_create_revert(
        &mut self,
        transition: TransitionAccount,
    ) -> Option<AccountRevert> {
        let updated_info = transition.info;
        let updated_storage = transition.storage;
        let updated_status = transition.status;

        let extend_storage = |this_storage: &mut Storage, storage_update: Storage| {
            for (key, value) in storage_update {
                this_storage.entry(key).or_insert(value).present_value = value.present_value;
            }
        };

        // Helper function that exploads account and returns revert state.
        let make_it_explode = |original_status: AccountStatus,
                               info: AccountInfo,
                               mut storage: Storage|
         -> Option<AccountRevert> {
            let previous_account = info;
            // Zero all present storage values and save present values to AccountRevert.
            let previous_storage = storage
                .iter_mut()
                .map(|(key, value)| {
                    // take previous value and set ZERO as storage got destroyed.
                    let previous_value = core::mem::take(&mut value.present_value);
                    (*key, RevertToSlot::Some(previous_value))
                })
                .collect();
            Some(AccountRevert {
                account: AccountInfoRevert::RevertTo(previous_account),
                storage: previous_storage,
                original_status,
                wipe_storage: true,
            })
        };
        // Very similar to make_it_explode but it will add additional zeros (RevertToSlot::Destroyed)
        // for the storage that are set if account is again created.
        //
        // Example is of going from New (state: 1: 10) -> DestroyedNew (2:10)
        // Revert of that needs to be list of key previous values.
        // [1:10,2:0]
        let make_it_expload_with_aftereffect = |original_status: AccountStatus,
                                                previous_info: AccountInfo,
                                                mut previous_storage: Storage,
                                                destroyed_storage: HashMap<U256, RevertToSlot>|
         -> Option<AccountRevert> {
            // Take present storage values as the storages that we are going to revert to.
            // As those values got destroyed.
            let mut previous_storage: HashMap<U256, RevertToSlot> = previous_storage
                .drain()
                .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                .collect();
            for (key, _) in destroyed_storage {
                previous_storage
                    .entry(key)
                    .or_insert(RevertToSlot::Destroyed);
            }
            Some(AccountRevert {
                account: AccountInfoRevert::RevertTo(previous_info),
                storage: previous_storage,
                original_status,
                wipe_storage: true,
            })
        };

        // Helper to extract storage from plain state and convert it to RevertToSlot::Destroyed.
        let destroyed_storage = |updated_storage: &Storage| -> HashMap<U256, RevertToSlot> {
            updated_storage
                .iter()
                .map(|(key, _)| (*key, RevertToSlot::Destroyed))
                .collect()
        };

        // handle it more optimal in future but for now be more flexible to set the logic.
        let previous_storage_from_update = updated_storage
            .iter()
            .filter(|s| s.1.original_value != s.1.present_value)
            .map(|(key, value)| (*key, RevertToSlot::Some(value.original_value)))
            .collect();

        // Missing update is for Destroyed,DestroyedAgain,DestroyedNew,DestroyedChange.
        // as those update are different between each other.
        // It updated from state before destroyed. And that is NewChanged,New,Changed,LoadedEmptyEIP161.
        // take a note that is not updating LoadedNotExisting.
        let update_part_of_destroyed =
            |this: &mut Self, updated_storage: &Storage| -> Option<AccountRevert> {
                match this.status {
                    AccountStatus::InMemoryChange => make_it_expload_with_aftereffect(
                        AccountStatus::InMemoryChange,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(updated_storage),
                    ),
                    AccountStatus::Changed => make_it_expload_with_aftereffect(
                        AccountStatus::Changed,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(updated_storage),
                    ),
                    AccountStatus::LoadedEmptyEIP161 => make_it_expload_with_aftereffect(
                        AccountStatus::LoadedEmptyEIP161,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(updated_storage),
                    ),
                    _ => None,
                }
            };

        match updated_status {
            AccountStatus::Changed => {
                match self.status {
                    AccountStatus::Changed => {
                        // extend the storage. original values is not used inside bundle.
                        let revert_info = if self.info != updated_info {
                            AccountInfoRevert::RevertTo(self.info.clone().unwrap_or_default())
                        } else {
                            AccountInfoRevert::DoNothing
                        };
                        extend_storage(&mut self.storage, updated_storage);
                        self.info = updated_info;
                        Some(AccountRevert {
                            account: revert_info,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Changed,
                            wipe_storage: false,
                        })
                    }
                    AccountStatus::Loaded => {
                        let info_revert = if self.info != updated_info {
                            AccountInfoRevert::RevertTo(self.info.clone().unwrap_or_default())
                        } else {
                            AccountInfoRevert::DoNothing
                        };
                        self.status = AccountStatus::Changed;
                        self.info = updated_info;
                        extend_storage(&mut self.storage, updated_storage);

                        Some(AccountRevert {
                            account: info_revert,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Loaded,
                            wipe_storage: false,
                        })
                    }
                    AccountStatus::LoadedEmptyEIP161 => {
                        // Only change that can happen from LoadedEmpty to Changed
                        // is if balance is send to account. So we are only checking account change here.
                        let info_revert = if self.info != updated_info {
                            AccountInfoRevert::RevertTo(self.info.clone().unwrap_or_default())
                        } else {
                            AccountInfoRevert::DoNothing
                        };
                        self.status = AccountStatus::Changed;
                        self.info = updated_info;
                        Some(AccountRevert {
                            account: info_revert,
                            storage: HashMap::default(),
                            original_status: AccountStatus::Loaded,
                            wipe_storage: false,
                        })
                    }
                    _ => unreachable!("Invalid state transfer to Changed from {self:?}"),
                }
            }
            AccountStatus::InMemoryChange => match self.status {
                AccountStatus::LoadedEmptyEIP161 => {
                    let revert_info = if self.info != updated_info {
                        AccountInfoRevert::RevertTo(AccountInfo::default())
                    } else {
                        AccountInfoRevert::DoNothing
                    };
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::InMemoryChange;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedEmptyEIP161,
                        wipe_storage: false,
                    })
                }
                AccountStatus::Loaded => {
                    // from loaded to InMemoryChange can happen if there is balance change
                    // or new created account but Loaded didn't have contract.
                    let revert_info =
                        AccountInfoRevert::RevertTo(self.info.clone().unwrap_or_default());
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::InMemoryChange;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::Loaded,
                        wipe_storage: false,
                    })
                }
                AccountStatus::LoadedNotExisting => {
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::InMemoryChange;
                    self.info = updated_info;
                    self.storage = updated_storage;

                    Some(AccountRevert {
                        account: AccountInfoRevert::DeleteIt,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedNotExisting,
                        wipe_storage: false,
                    })
                }
                AccountStatus::InMemoryChange => {
                    let revert_info = if self.info != updated_info {
                        AccountInfoRevert::RevertTo(self.info.clone().unwrap_or_default())
                    } else {
                        AccountInfoRevert::DoNothing
                    };
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::InMemoryChange;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::InMemoryChange,
                        wipe_storage: false,
                    })
                }
                _ => unreachable!("Invalid change to InMemoryChange from {self:?}"),
            },
            AccountStatus::Loaded => {
                // No changeset, maybe just update data
                // Do nothing for now.
                None
            }
            AccountStatus::LoadedNotExisting => {
                // Not changeset, maybe just update data.
                // Do nothing for now.
                None
            }
            AccountStatus::LoadedEmptyEIP161 => {
                // No changeset maybe just update data.
                // Do nothing for now
                None
            }
            AccountStatus::Destroyed => {
                let this_info = self.info.take().unwrap_or_default();
                let this_storage = self.storage.drain().collect();
                let ret = match self.status {
                    AccountStatus::InMemoryChange => {
                        make_it_explode(AccountStatus::InMemoryChange, this_info, this_storage)
                    }
                    AccountStatus::Changed => {
                        make_it_explode(AccountStatus::Changed, this_info, this_storage)
                    }
                    AccountStatus::LoadedEmptyEIP161 => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this_info, this_storage)
                    }
                    AccountStatus::Loaded => {
                        make_it_explode(AccountStatus::Loaded, this_info, this_storage)
                    }
                    AccountStatus::LoadedNotExisting => {
                        // Do nothing as we have LoadedNotExisting -> Destroyed (It is noop)
                        return None;
                    }
                    _ => unreachable!("Invalid transition to Destroyed account from: {self:?} to {updated_info:?} {updated_status:?}"),
                };
                self.status = AccountStatus::Destroyed;
                // set present to destroyed.
                ret
            }
            AccountStatus::DestroyedChanged => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &updated_storage) {
                    // set to destroyed and revert state.
                    self.status = AccountStatus::DestroyedChanged;
                    self.info = updated_info;
                    self.storage = updated_storage;

                    return Some(revert_state);
                }

                let ret = match self.status {
                    AccountStatus::Destroyed => {
                        // from destroyed state new account is made
                        Some(AccountRevert {
                            account: AccountInfoRevert::RevertTo(AccountInfo::default()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Destroyed,
                            wipe_storage: false,
                        })
                    }
                    AccountStatus::DestroyedChanged => {
                        let revert_info = if self.info != updated_info {
                            AccountInfoRevert::RevertTo(AccountInfo::default())
                        } else {
                            AccountInfoRevert::DoNothing
                        };
                        // Stays same as DestroyedNewChanged
                        Some(AccountRevert {
                            // empty account
                            account: revert_info,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedChanged,
                            wipe_storage: false,
                        })
                    }
                    AccountStatus::LoadedNotExisting => {
                        // we can make self to be New
                        //
                        // Example of this transition is loaded empty -> New -> destroyed -> New.
                        // Is same as just loaded empty -> New.
                        //
                        // This will devour the Selfdestruct as it is not needed.
                        self.status = AccountStatus::DestroyedChanged;

                        Some(AccountRevert {
                            // empty account
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                            wipe_storage: false,
                        })
                    }
                    AccountStatus::DestroyedAgain => make_it_expload_with_aftereffect(
                        // destroyed again will set empty account.
                        AccountStatus::DestroyedAgain,
                        AccountInfo::default(),
                        HashMap::default(),
                        destroyed_storage(&updated_storage),
                    ),
                    _ => unreachable!("Invalid state transfer to DestroyedNew from {self:?}"),
                };
                self.status = AccountStatus::DestroyedChanged;
                self.info = updated_info;
                self.storage = updated_storage;

                ret
            }
            AccountStatus::DestroyedAgain => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &HashMap::default()) {
                    // set to destroyed and revert state.
                    self.status = AccountStatus::DestroyedAgain;
                    self.info = None;
                    self.storage.clear();

                    return Some(revert_state);
                }
                let ret = match self.status {
                    AccountStatus::Destroyed
                    | AccountStatus::DestroyedAgain
                    | AccountStatus::LoadedNotExisting => {
                        // From destroyed to destroyed again. is noop
                        //
                        // DestroyedAgain to DestroyedAgain is noop
                        //
                        // From LoadedNotExisting to DestroyedAgain
                        // is noop as account is destroyed again
                        None
                    }
                    AccountStatus::DestroyedChanged => {
                        // From destroyed new to destroyed again.
                        let ret = AccountRevert {
                            // empty account
                            account: AccountInfoRevert::RevertTo(
                                self.info.clone().unwrap_or_default(),
                            ),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedChanged,
                            wipe_storage: false,
                        };
                        self.info = None;
                        Some(ret)
                    }
                    _ => unreachable!("Invalid state to DestroyedAgain from {self:?}"),
                };
                self.info = None;
                self.status = AccountStatus::DestroyedAgain;
                ret
            }
        }
    }
}
