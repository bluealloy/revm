use revm_interpreter::primitives::{AccountInfo, U256};
use revm_precompile::HashMap;

use super::{AccountStatus, PlainAccount, RevertAccountState, RevertToSlot, Storage};

/// Seems better, and more cleaner. But all informations is there.
/// Should we extract storage...
#[derive(Clone, Debug)]
pub struct BundleAccount {
    pub account: Option<PlainAccount>,
    pub status: AccountStatus,
}

impl BundleAccount {
    pub fn new_loaded(info: AccountInfo, storage: Storage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Loaded,
        }
    }
    pub fn new_loaded_empty_eip161(storage: Storage) -> Self {
        Self {
            account: Some(PlainAccount::new_empty_with_storage(storage)),
            status: AccountStatus::LoadedEmptyEIP161,
        }
    }
    pub fn new_loaded_not_existing() -> Self {
        Self {
            account: None,
            status: AccountStatus::LoadedNotExisting,
        }
    }
    /// Create new account that is newly created (State is AccountStatus::New)
    pub fn new_newly_created(info: AccountInfo, storage: Storage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::New,
        }
    }

    /// Create account that is destroyed.
    pub fn new_destroyed() -> Self {
        Self {
            account: None,
            status: AccountStatus::Destroyed,
        }
    }

    /// Create changed account
    pub fn new_changed(info: AccountInfo, storage: Storage) -> Self {
        Self {
            account: Some(PlainAccount { info, storage }),
            status: AccountStatus::Changed,
        }
    }

    pub fn is_some(&self) -> bool {
        match self.status {
            AccountStatus::Changed => true,
            AccountStatus::New => true,
            AccountStatus::NewChanged => true,
            AccountStatus::DestroyedNew => true,
            AccountStatus::DestroyedNewChanged => true,
            _ => false,
        }
    }

    /// Fetch storage slot if account and storage exist
    pub fn storage_slot(&self, storage_key: U256) -> Option<U256> {
        self.account
            .as_ref()
            .and_then(|a| a.storage.get(&storage_key).map(|slot| slot.present_value))
    }

    /// Fetch account info if it exist.
    pub fn account_info(&self) -> Option<AccountInfo> {
        self.account.as_ref().map(|a| a.info.clone())
    }

    /// Touche empty account, related to EIP-161 state clear.
    pub fn touch_empty(&mut self) {
        self.status = match self.status {
            AccountStatus::DestroyedNew => AccountStatus::DestroyedAgain,
            AccountStatus::New => {
                // account can be created empty them touched.
                // Note: we can probably set it to LoadedNotExisting.
                AccountStatus::Destroyed
            }
            AccountStatus::LoadedEmptyEIP161 => AccountStatus::Destroyed,
            _ => {
                // do nothing
                unreachable!("Wrong state transition, touch empty is not possible from {self:?}");
            }
        };
        self.account = None;
    }

    /// Consume self and make account as destroyed.
    ///
    /// Set account as None and set status to Destroyer or DestroyedAgain.
    pub fn selfdestruct(&mut self) {
        self.status = match self.status {
            AccountStatus::DestroyedNew | AccountStatus::DestroyedNewChanged => {
                AccountStatus::DestroyedAgain
            }
            AccountStatus::Destroyed => {
                // mark as destroyed again, this can happen if account is created and
                // then selfdestructed in same block.
                // Note: there is no big difference between Destroyed and DestroyedAgain
                // in this case, but was added for clarity.
                AccountStatus::DestroyedAgain
            }
            _ => AccountStatus::Destroyed,
        };
        // make accoutn as None as it is destroyed.
        self.account = None
    }

    /// Newly created account.
    pub fn newly_created(&mut self, new: AccountInfo, storage: &Storage) {
        self.status = match self.status {
            // if account was destroyed previously just copy new info to it.
            AccountStatus::DestroyedAgain | AccountStatus::Destroyed => AccountStatus::DestroyedNew,
            // if account is loaded from db.
            AccountStatus::LoadedNotExisting => AccountStatus::New,
            AccountStatus::LoadedEmptyEIP161 | AccountStatus::Loaded => {
                // if account is loaded and not empty this means that account has some balance
                // this does not mean that accoun't can be created.
                // We are assuming that EVM did necessary checks before allowing account to be created.
                AccountStatus::New
            }
            _ => unreachable!(
                "Wrong state transition to create:\nfrom: {:?}\nto: {:?}",
                self, new
            ),
        };
        self.account = Some(PlainAccount {
            info: new,
            storage: storage.clone(),
        });
    }

    pub fn change(&mut self, new: AccountInfo, storage: Storage) {
        let transfer = |this_account: &mut PlainAccount| -> PlainAccount {
            let mut this_storage = core::mem::take(&mut this_account.storage);
            // TODO save original value and dont overwrite it.
            this_storage.extend(storage.into_iter());
            PlainAccount {
                info: new,
                storage: this_storage,
            }
        };
        // TODE remove helper `transfer`
        // Account should always be Some but if wrong transition happens we will panic in last match arm.
        let changed_account = transfer(&mut self.account.take().unwrap_or_default());

        self.status = match self.status {
            AccountStatus::Loaded => {
                // If account was initially loaded we are just overwriting it.
                // We are not checking if account is changed.
                // storage can be.
                AccountStatus::Changed
            }
            AccountStatus::Changed => {
                // Update to new changed state.
                AccountStatus::Changed
            }
            AccountStatus::New => {
                // promote to NewChanged.
                // Check if account is empty is done outside of this fn.
                AccountStatus::NewChanged
            }
            AccountStatus::NewChanged => {
                // Update to new changed state.
                AccountStatus::NewChanged
            }
            AccountStatus::DestroyedNew => {
                // promote to DestroyedNewChanged.
                AccountStatus::DestroyedNewChanged
            }
            AccountStatus::DestroyedNewChanged => {
                // Update to new changed state.
                AccountStatus::DestroyedNewChanged
            }
            AccountStatus::LoadedEmptyEIP161 => {
                // Change on empty account, should transfer storage if there is any.
                AccountStatus::Changed
            }
            AccountStatus::LoadedNotExisting
            | AccountStatus::Destroyed
            | AccountStatus::DestroyedAgain => {
                unreachable!("Wronge state transition change: \nfrom:{self:?}")
            }
        };
        self.account = Some(changed_account);
    }

    /// Update to new state and generate RevertAccountState that if applied to new state will
    /// revert it to previous state. If not revert is present, update is noop.
    ///
    /// TODO consume state and return it back with RevertAccountState. This would skip some bugs
    /// of not setting the state.
    ///
    /// TODO recheck if simple account state enum disrupts anything in bas way.
    pub fn update_and_create_revert(
        &mut self,
        mut main_update: Self,
    ) -> Option<RevertAccountState> {
        // Helper function that exploads account and returns revert state.
        let make_it_explode = |original_status: AccountStatus,
                               mut this: PlainAccount|
         -> Option<RevertAccountState> {
            let previous_account = this.info.clone();
            // Take present storage values as the storages that we are going to revert to.
            // As those values got destroyed.
            let previous_storage = this
                .storage
                .drain()
                .into_iter()
                .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                .collect();
            let revert = Some(RevertAccountState {
                account: Some(previous_account),
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
                                                mut this: PlainAccount,
                                                destroyed_storage: HashMap<U256, RevertToSlot>|
         -> Option<RevertAccountState> {
            let previous_account = this.info.clone();
            // Take present storage values as the storages that we are going to revert to.
            // As those values got destroyed.
            let mut previous_storage: HashMap<U256, RevertToSlot> = this
                .storage
                .drain()
                .into_iter()
                .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                .collect();
            for (key, _) in destroyed_storage {
                previous_storage
                    .entry(key)
                    .or_insert(RevertToSlot::Destroyed);
            }
            let revert = Some(RevertAccountState {
                account: Some(previous_account),
                storage: previous_storage,
                original_status,
            });

            revert
        };

        // Helper to extract storage from plain state and convert it to RevertToSlot::Destroyed.
        let destroyed_storage = |account: &PlainAccount| -> HashMap<U256, RevertToSlot> {
            account
                .storage
                .iter()
                .map(|(key, _)| (*key, RevertToSlot::Destroyed))
                .collect()
        };

        // handle it more optimal in future but for now be more flexible to set the logic.
        let previous_storage_from_update = main_update
            .account
            .as_ref()
            .map(|a| {
                a.storage
                    .iter()
                    .filter(|s| s.1.original_value != s.1.present_value)
                    .map(|(key, value)| (*key, RevertToSlot::Some(value.original_value.clone())))
                    .collect()
            })
            .unwrap_or_default();

        // Missing update is for Destroyed,DestroyedAgain,DestroyedNew,DestroyedChange.
        // as those update are different between each other.
        // It updated from state before destroyed. And that is NewChanged,New,Changed,LoadedEmptyEIP161.
        // take a note that is not updating LoadedNotExisting.
        let update_part_of_destroyed =
            |this: &mut Self, update: &PlainAccount| -> Option<RevertAccountState> {
                match this.status {
                    AccountStatus::NewChanged => make_it_expload_with_aftereffect(
                        AccountStatus::NewChanged,
                        this.account.clone().unwrap_or_default(),
                        destroyed_storage(&update),
                    ),
                    AccountStatus::New => make_it_expload_with_aftereffect(
                        // Previous block created account, this block destroyed it and created it again.
                        // This means that bytecode get changed.
                        AccountStatus::New,
                        this.account.clone().unwrap_or_default(),
                        destroyed_storage(&update),
                    ),
                    AccountStatus::Changed => make_it_expload_with_aftereffect(
                        AccountStatus::Changed,
                        this.account.clone().unwrap_or_default(),
                        destroyed_storage(&update),
                    ),
                    AccountStatus::LoadedEmptyEIP161 => make_it_expload_with_aftereffect(
                        AccountStatus::LoadedEmptyEIP161,
                        this.account.clone().unwrap_or_default(),
                        destroyed_storage(&update),
                    ),
                    _ => None,
                }
            };
        // Assume this account is going to be overwritten.
        let mut this = self.account.take().unwrap_or_default();
        // TODO CHECK WHERE MAIN_UPDATE IS USED AS WE JUST TOOK ITS ACCOUNT!!!
        let update = main_update.account.take().unwrap_or_default();
        match main_update.status {
            AccountStatus::Changed => {
                match self.status {
                    AccountStatus::Changed => {
                        // extend the storage. original values is not used inside bundle.
                        this.storage.extend(update.storage);
                        this.info = update.info;
                        return Some(RevertAccountState {
                            account: Some(this.info.clone()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::Changed,
                        });
                    }
                    AccountStatus::Loaded => {
                        // extend the storage. original values is not used inside bundle.
                        let mut storage = core::mem::take(&mut this.storage);
                        storage.extend(update.storage);
                        let previous_account = this.info.clone();
                        self.status = AccountStatus::Changed;
                        self.account = Some(PlainAccount {
                            info: update.info,
                            storage,
                        });
                        return Some(RevertAccountState {
                            account: Some(previous_account),
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
                        let mut storage = core::mem::take(&mut this.storage);
                        storage.extend(update.storage);
                        self.status = AccountStatus::New;
                        self.account = Some(PlainAccount {
                            info: update.info,
                            storage: storage,
                        });
                        // old account is empty. And that is diffeerent from not existing.
                        return Some(RevertAccountState {
                            account: Some(AccountInfo::default()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedEmptyEIP161,
                        });
                    }
                    AccountStatus::LoadedNotExisting => {
                        self.status = AccountStatus::New;
                        self.account = Some(update.clone());
                        return Some(RevertAccountState {
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    _ => unreachable!(
                        "Invalid transition to New account from: {self:?} to {main_update:?}"
                    ),
                }
            }
            AccountStatus::NewChanged => match self.status {
                AccountStatus::LoadedEmptyEIP161 => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::New;
                    self.account = Some(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertAccountState {
                        account: Some(AccountInfo::default()),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedEmptyEIP161,
                    });
                }
                AccountStatus::LoadedNotExisting => {
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::New;
                    self.account = Some(update.clone());
                    return Some(RevertAccountState {
                        account: None,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedNotExisting,
                    });
                }
                AccountStatus::New => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);

                    let previous_account = this.info.clone();
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::NewChanged;
                    self.account = Some(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertAccountState {
                        account: Some(previous_account),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::New,
                    });
                }
                AccountStatus::NewChanged => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);

                    let previous_account = this.info.clone();
                    // set as new as we didn't have that transition
                    self.status = AccountStatus::NewChanged;
                    self.account = Some(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertAccountState {
                        account: Some(previous_account),
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
                let ret = match self.status {
                    AccountStatus::NewChanged => make_it_explode(AccountStatus::NewChanged, this),
                    AccountStatus::New => make_it_explode(AccountStatus::New, this),
                    AccountStatus::Changed => make_it_explode(AccountStatus::Changed, this),
                    AccountStatus::LoadedEmptyEIP161 => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this)
                    }
                    AccountStatus::Loaded => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this)
                    }
                    AccountStatus::LoadedNotExisting => {
                        // Do nothing as we have LoadedNotExisting -> Destroyed (It is noop)
                        return None;
                    }
                    _ => unreachable!("Invalid state"),
                };

                // set present to destroyed.
                self.status = AccountStatus::Destroyed;
                // present state of account is `None`.
                self.account = None;
                return ret;
            }
            AccountStatus::DestroyedNew => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &update) {
                    // set to destroyed and revert state.
                    self.status = AccountStatus::DestroyedNew;
                    self.account = Some(update);
                    return Some(revert_state);
                }

                let ret = match self.status {
                    AccountStatus::Destroyed => {
                        // from destroyed state new account is made
                        Some(RevertAccountState {
                            account: None,
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
                        self.account = Some(update.clone());
                        return Some(RevertAccountState {
                            // empty account
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    AccountStatus::DestroyedAgain => make_it_expload_with_aftereffect(
                        // destroyed again will set empty account.
                        AccountStatus::DestroyedAgain,
                        PlainAccount::default(),
                        destroyed_storage(&update),
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
                self.account = Some(update);
                return ret;
            }
            AccountStatus::DestroyedNewChanged => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &update) {
                    // set it to destroyed changed and update account as it is newest best state.
                    self.status = AccountStatus::DestroyedNewChanged;
                    self.account = Some(update);
                    return Some(revert_state);
                }

                let ret = match self.status {
                    AccountStatus::Destroyed => {
                        // Becomes DestroyedNew
                        RevertAccountState {
                            // empty account
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::DestroyedNew => {
                        // Becomes DestroyedNewChanged
                        RevertAccountState {
                            // empty account
                            account: Some(this.info.clone()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::DestroyedNewChanged => {
                        // Stays same as DestroyedNewChanged
                        RevertAccountState {
                            // empty account
                            account: Some(this.info.clone()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        }
                    }
                    AccountStatus::LoadedNotExisting => {
                        // Becomes New.
                        // Example of this happening is NotExisting -> New -> Destroyed -> New -> Changed.
                        // This is same as NotExisting -> New.
                        self.status = AccountStatus::New;
                        self.account = Some(update.clone());
                        return Some(RevertAccountState {
                            // empty account
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNewChanged,
                        });
                    }
                    _ => unreachable!("Invalid state"),
                };

                self.status = AccountStatus::DestroyedNew;
                self.account = Some(update.clone());
                return Some(ret);
            }
            AccountStatus::DestroyedAgain => {
                // Previous block created account
                // (It was destroyed on previous block or one before).

                // check common pre destroy paths.
                if let Some(revert_state) = update_part_of_destroyed(self, &PlainAccount::default())
                {
                    // set to destroyed and revert state.
                    self.status = AccountStatus::DestroyedAgain;
                    self.account = None;
                    return Some(revert_state);
                }
                match self.status {
                    AccountStatus::Destroyed => {
                        // From destroyed to destroyed again. is noop
                        return None;
                    }
                    AccountStatus::DestroyedNew => {
                        // From destroyed new to destroyed again.
                        let ret = RevertAccountState {
                            // empty account
                            account: Some(this.info.clone()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::DestroyedNew,
                        };
                        return Some(ret);
                    }
                    AccountStatus::DestroyedNewChanged => {
                        // From DestroyedNewChanged to DestroyedAgain
                        let ret = RevertAccountState {
                            // empty account
                            account: Some(this.info.clone()),
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
