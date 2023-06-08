use super::{
    reverts::AccountInfoRevert, AccountRevert, AccountStatus, RevertToSlot, Storage,
    TransitionAccount,
};
use revm_interpreter::primitives::{AccountInfo, U256};
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
    /// If Account was destroyed we ignore original value.
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

    /// Return true of account info was changed.
    pub fn is_info_changed(&self) -> bool {
        self.info != self.original_info
    }

    /// Return true if contract was changed
    pub fn is_contract_changed(&self) -> bool {
        self.info.as_ref().map(|a| a.code_hash) != self.original_info.as_ref().map(|a| a.code_hash)
    }

    /// Update to new state and generate AccountRevert that if applied to new state will
    /// revert it to previous state. If not revert is present, update is noop.
    ///
    /// TODO consume state and return it back with AccountRevert. This would skip some bugs
    /// of not setting the state.
    ///
    /// TODO recheck if change to simple account state enum disrupts anything.
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
            let previous_account = info.clone();
            // Take present storage values as the storages that we are going to revert to.
            // As those values got destroyed.
            let previous_storage = storage
                .drain()
                .into_iter()
                .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                .collect();
            let revert = Some(AccountRevert {
                account: AccountInfoRevert::RevertTo(previous_account),
                storage: previous_storage,
                original_status,
            });

            revert
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
                .into_iter()
                .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                .collect();
            for (key, _) in destroyed_storage {
                previous_storage
                    .entry(key)
                    .or_insert(RevertToSlot::Destroyed);
            }
            let revert = Some(AccountRevert {
                account: AccountInfoRevert::RevertTo(previous_info),
                storage: previous_storage,
                original_status,
            });

            revert
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
            .map(|(key, value)| (*key, RevertToSlot::Some(value.original_value.clone())))
            .collect();

        // Missing update is for Destroyed,DestroyedAgain,DestroyedNew,DestroyedChange.
        // as those update are different between each other.
        // It updated from state before destroyed. And that is NewChanged,New,Changed,LoadedEmptyEIP161.
        // take a note that is not updating LoadedNotExisting.
        let update_part_of_destroyed =
            |this: &mut Self, updated_storage: &Storage| -> Option<AccountRevert> {
                match this.status {
                    AccountStatus::NewChanged => make_it_expload_with_aftereffect(
                        AccountStatus::NewChanged,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(&updated_storage),
                    ),
                    AccountStatus::New => make_it_expload_with_aftereffect(
                        // Previous block created account, this block destroyed it and created it again.
                        // This means that bytecode get changed.
                        AccountStatus::New,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(&updated_storage),
                    ),
                    AccountStatus::Changed => make_it_expload_with_aftereffect(
                        AccountStatus::Changed,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(&updated_storage),
                    ),
                    AccountStatus::LoadedEmptyEIP161 => make_it_expload_with_aftereffect(
                        AccountStatus::LoadedEmptyEIP161,
                        this.info.clone().unwrap_or_default(),
                        this.storage.drain().collect(),
                        destroyed_storage(&updated_storage),
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
                        return Some(AccountRevert {
                            account: revert_info,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Changed,
                        });
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

                        return Some(AccountRevert {
                            account: info_revert,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Loaded,
                        });
                    } //discard changes
                    _ => unreachable!("Invalid state"),
                }
            }
            AccountStatus::New => {
                // this state need to be loaded from db
                match self.status {
                    AccountStatus::LoadedEmptyEIP161 => {
                        self.status = AccountStatus::New;
                        self.info = updated_info;
                        extend_storage(&mut self.storage, updated_storage);

                        // old account is empty. And that is diffeerent from not existing.
                        return Some(AccountRevert {
                            account: AccountInfoRevert::RevertTo(AccountInfo::default()
                            ),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedEmptyEIP161,
                        });
                    }
                    AccountStatus::LoadedNotExisting => {
                        self.status = AccountStatus::New;
                        self.info = updated_info;
                        self.storage = updated_storage;

                        return Some(AccountRevert {
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    _ => unreachable!(
                        "Invalid transition to New account from: {self:?} to {updated_info:?} {updated_status:?}"
                    ),
                }
            }
            AccountStatus::NewChanged => match self.status {
                AccountStatus::LoadedEmptyEIP161 => {
                    let revert_info = if self.info != updated_info {
                        AccountInfoRevert::RevertTo(AccountInfo::default())
                    } else {
                        AccountInfoRevert::DoNothing
                    };
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::New;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    return Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedEmptyEIP161,
                    });
                }
                AccountStatus::LoadedNotExisting => {
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::New;
                    self.info = updated_info;
                    self.storage = updated_storage;

                    return Some(AccountRevert {
                        account: AccountInfoRevert::DeleteIt,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedNotExisting,
                    });
                }
                AccountStatus::New => {
                    let revert_info = if self.info != updated_info {
                        AccountInfoRevert::RevertTo(AccountInfo::default())
                    } else {
                        AccountInfoRevert::DoNothing
                    };
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::NewChanged;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    return Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::New,
                    });
                }
                AccountStatus::NewChanged => {
                    let revert_info = if self.info != updated_info {
                        AccountInfoRevert::RevertTo(AccountInfo::default())
                    } else {
                        AccountInfoRevert::DoNothing
                    };
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::NewChanged;
                    self.info = updated_info;
                    extend_storage(&mut self.storage, updated_storage);

                    return Some(AccountRevert {
                        account: revert_info,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::NewChanged,
                    });
                }
                _ => unreachable!("Invalid state"),
            },
            AccountStatus::Loaded => {
                // No changeset, maybe just update data
                // Do nothing for now.
                return None;
            }
            AccountStatus::LoadedNotExisting => {
                // Not changeset, maybe just update data.
                // Do nothing for now.
                return None;
            }
            AccountStatus::LoadedEmptyEIP161 => {
                // No changeset maybe just update data.
                // Do nothing for now
                return None;
            }
            AccountStatus::Destroyed => {
                self.status = AccountStatus::Destroyed;
                let this_info = self.info.take().unwrap_or_default();
                let this_storage = self.storage.drain().collect();
                let ret = match self.status {
                    AccountStatus::NewChanged => {
                        make_it_explode(AccountStatus::NewChanged, this_info, this_storage)
                    }
                    AccountStatus::New => {
                        make_it_explode(AccountStatus::New, this_info, this_storage)
                    }
                    AccountStatus::Changed => {
                        make_it_explode(AccountStatus::Changed, this_info, this_storage)
                    }
                    AccountStatus::LoadedEmptyEIP161 => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this_info, this_storage)
                    }
                    AccountStatus::Loaded => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this_info, this_storage)
                    }
                    AccountStatus::LoadedNotExisting => {
                        // Do nothing as we have LoadedNotExisting -> Destroyed (It is noop)
                        return None;
                    }
                    _ => unreachable!("Invalid state"),
                };

                // set present to destroyed.
                return ret;
            }
            AccountStatus::DestroyedNew => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &updated_storage) {
                    // set to destroyed and revert state.
                    self.status = AccountStatus::DestroyedNew;
                    self.info = updated_info;
                    self.storage = updated_storage;

                    return Some(revert_state);
                }

                let ret = match self.status {
                    AccountStatus::Destroyed => {
                        // from destroyed state new account is made
                        Some(AccountRevert {
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Destroyed,
                        })
                    }
                    AccountStatus::LoadedNotExisting => {
                        // we can make self to be New
                        //
                        // Example of this transition is loaded empty -> New -> destroyed -> New.
                        // Is same as just loaded empty -> New.
                        //
                        // This will devour the Selfdestruct as it is not needed.
                        self.status = AccountStatus::New;
                        self.info = updated_info;
                        self.storage = updated_storage;

                        return Some(AccountRevert {
                            // empty account
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    AccountStatus::DestroyedAgain => make_it_expload_with_aftereffect(
                        // destroyed again will set empty account.
                        AccountStatus::DestroyedAgain,
                        AccountInfo::default(),
                        HashMap::default(),
                        destroyed_storage(&updated_storage),
                    ),
                    AccountStatus::DestroyedNew => {
                        // From DestroyeNew -> DestroyedAgain -> DestroyedNew
                        // Note: how to handle new bytecode changed?
                        // TODO
                        return None;
                    }
                    _ => unreachable!("Invalid state"),
                };
                self.status = AccountStatus::DestroyedNew;
                self.info = updated_info;
                self.storage = updated_storage;

                return ret;
            }
            AccountStatus::DestroyedNewChanged => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &updated_storage) {
                    // set it to destroyed changed and update account as it is newest best state.
                    self.status = AccountStatus::DestroyedNewChanged;
                    self.info = updated_info;
                    self.storage = updated_storage;

                    return Some(revert_state);
                }

                let ret = match self.status {
                    AccountStatus::Destroyed => {
                        // Becomes DestroyedNew
                        AccountRevert {
                            // empty account
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::DestroyedNew => {
                        // Becomes DestroyedNewChanged
                        AccountRevert {
                            // empty account
                            account: AccountInfoRevert::RevertTo(
                                self.info.clone().unwrap_or_default(),
                            ),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::DestroyedNewChanged => {
                        let revert_info = if self.info != updated_info {
                            AccountInfoRevert::RevertTo(AccountInfo::default())
                        } else {
                            AccountInfoRevert::DoNothing
                        };
                        // Stays same as DestroyedNewChanged
                        AccountRevert {
                            // empty account
                            account: revert_info,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::LoadedNotExisting => {
                        // Becomes New.
                        // Example of this happening is NotExisting -> New -> Destroyed -> New -> Changed.
                        // This is same as NotExisting -> New.
                        self.status = AccountStatus::New;
                        self.info = updated_info;
                        self.storage = updated_storage;

                        return Some(AccountRevert {
                            // empty account
                            account: AccountInfoRevert::DeleteIt,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        });
                    }
                    _ => unreachable!("Invalid state"),
                };

                self.status = AccountStatus::DestroyedNew;
                self.info = updated_info;
                self.storage = updated_storage;

                return Some(ret);
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
                match self.status {
                    AccountStatus::Destroyed => {
                        // From destroyed to destroyed again. is noop
                        return None;
                    }
                    AccountStatus::DestroyedNew => {
                        // From destroyed new to destroyed again.
                        let ret = AccountRevert {
                            // empty account
                            account: AccountInfoRevert::RevertTo(
                                self.info.clone().unwrap_or_default(),
                            ),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNew,
                        };
                        return Some(ret);
                    }
                    AccountStatus::DestroyedNewChanged => {
                        // From DestroyedNewChanged to DestroyedAgain
                        let ret = AccountRevert {
                            // empty account
                            account: AccountInfoRevert::RevertTo(
                                self.info.clone().unwrap_or_default(),
                            ),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        };
                        return Some(ret);
                    }
                    AccountStatus::DestroyedAgain => {
                        // DestroyedAgain to DestroyedAgain is noop
                        return None;
                    }
                    AccountStatus::LoadedNotExisting => {
                        // From LoadedNotExisting to DestroyedAgain
                        // is noop as account is destroyed again
                        return None;
                    }
                    _ => unreachable!("Invalid state"),
                }
            }
        }
    }
}
