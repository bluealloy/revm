use revm_interpreter::primitives::{
    db::{Database, DatabaseCommit},
    hash_map::{self, Entry},
    Account, AccountInfo, Bytecode, HashMap, State, StorageSlot, B160, B256, PRECOMPILE3, U256,
};

#[derive(Clone, Debug, Default)]
pub struct PlainAccount {
    pub info: AccountInfo,
    pub storage: Storage,
}

impl PlainAccount {
    pub fn new_empty_with_storage(storage: Storage) -> Self {
        Self {
            info: AccountInfo::default(),
            storage,
        }
    }
}

pub type Storage = HashMap<U256, StorageSlot>;

impl From<AccountInfo> for PlainAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BlockState {
    pub accounts: HashMap<B160, GlobalAccountState>,
    pub contracts: HashMap<B256, Bytecode>,
    pub has_state_clear: bool,
}

impl DatabaseCommit for BlockState {
    fn commit(&mut self, changes: HashMap<B160, Account>) {
        self.apply_evm_state(&changes)
    }
}

impl BlockState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
            has_state_clear: true,
        }
    }
    /// Legacy without state clear flag enabled
    pub fn new_legacy() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
            has_state_clear: false,
        }
    }
    /// Used for tests only. When transitioned it is not recoverable
    pub fn set_state_clear(&mut self) {
        if self.has_state_clear == true {
            return;
        }

        self.has_state_clear = true;
    }

    pub fn trie_account(&self) -> impl IntoIterator<Item = (B160, &PlainAccount)> {
        self.accounts.iter().filter_map(|(address, account)| {
            account.account().map(|plain_acc| (*address, plain_acc))
        })
    }

    pub fn insert_not_existing(&mut self, address: B160) {
        self.accounts
            .insert(address, GlobalAccountState::LoadedNotExisting);
    }

    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        let account = if !info.is_empty() {
            GlobalAccountState::Loaded(info.into())
        } else {
            GlobalAccountState::LoadedEmptyEIP161(PlainAccount::default())
        };
        self.accounts.insert(address, account);
    }

    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: Storage,
    ) {
        let account = if !info.is_empty() {
            GlobalAccountState::Loaded(PlainAccount { info, storage })
        } else {
            GlobalAccountState::LoadedEmptyEIP161(PlainAccount::new_empty_with_storage(storage))
        };
        self.accounts.insert(address, account);
    }

    pub fn apply_evm_state(&mut self, evm_state: &State) {
        //println!("PRINT STATE:");
        for (address, account) in evm_state {
            //println!("\n------:{:?} -> {:?}", address, account);
            if !account.is_touched() {
                continue;
            } else if account.is_selfdestructed() {
                // If it is marked as selfdestructed we to changed state to destroyed.
                match self.accounts.entry(*address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.selfdestruct();
                    }
                    Entry::Vacant(entry) => {
                        // if account is not present in db, we can just mark it as destroyed.
                        // This means that account was not loaded through this state.
                        entry.insert(GlobalAccountState::Destroyed);
                    }
                }
                continue;
            }
            let is_empty = account.is_empty();
            if account.is_created() {
                // Note: it can happen that created contract get selfdestructed in same block
                // that is why is newly created is checked after selfdestructed
                //
                // Note: Create2 (Petersburg) was after state clear EIP (Spurious Dragon)
                // so we dont need to clear
                //
                // Note: It is possibility to create KECCAK_EMPTY contract with some storage
                // by just setting storage inside CRATE contstructor. Overlap of those contracts
                // is not possible because CREATE2 is introduced later.
                //
                match self.accounts.entry(*address) {
                    // if account is already present id db.
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.newly_created(account.info.clone(), &account.storage)
                    }
                    Entry::Vacant(entry) => {
                        // This means that account was not loaded through this state.
                        // and we trust that account is empty.
                        entry.insert(GlobalAccountState::New(PlainAccount {
                            info: account.info.clone(),
                            storage: account.storage.clone(),
                        }));
                    }
                }
            } else {
                // Account is touched, but not selfdestructed or newly created.
                // Account can be touched and not changed.

                // And when empty account is touched it needs to be removed from database.
                // EIP-161 state clear
                if self.has_state_clear && is_empty {
                    // TODO Check if sending ZERO value created account pre state clear???

                    if *address == PRECOMPILE3 {
                        // Test related, this is considered bug that broke one of testsnets
                        // but it didn't reach mainnet as on mainnet any precompile had some balance.
                        continue;
                    }
                    // touch empty account.
                    match self.accounts.entry(*address) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().touch_empty();
                        }
                        Entry::Vacant(entry) => {}
                    }
                    // else do nothing as account is not existing
                    continue;
                }

                // mark account as changed.
                match self.accounts.entry(*address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.change(account.info.clone(), account.storage.clone());
                    }
                    Entry::Vacant(entry) => {
                        // It is assumed initial state is Loaded
                        entry.insert(GlobalAccountState::Changed(PlainAccount {
                            info: account.info.clone(),
                            storage: account.storage.clone(),
                        }));
                    }
                }
            }
        }
    }
}

impl Database for BlockState {
    type Error = ();

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(account) = self.accounts.get(&address) {
            return Ok(account.account_info());
        }

        Ok(None)
    }

    fn code_by_hash(
        &mut self,
        _code_hash: revm_interpreter::primitives::B256,
    ) -> Result<Bytecode, Self::Error> {
        unreachable!("Code is always returned in basic account info")
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        if let Some(account) = self.accounts.get(&address) {
            return Ok(account.storage_slot(index).unwrap_or_default());
        }

        Ok(U256::ZERO)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        Ok(B256::zero())
    }
}

/// This is action on state.
#[derive(Clone, Debug)]
pub enum GlobalAccountState {
    /// Loaded from db
    Loaded(PlainAccount),
    /// Account was present and it got changed from db
    Changed(PlainAccount),
    /// Account is not found inside db and it is newly created
    New(PlainAccount),
    /// New account that got changed
    NewChanged(PlainAccount),
    /// Account created that was previously destroyed
    DestroyedNew(PlainAccount),
    /// Account changed that was previously destroyed then created.
    DestroyedNewChanged(PlainAccount),
    /// Creating empty account was only possible before SpurioudDragon hardfork
    /// And last of those account were touched (removed) from state in block 14049881.
    /// EIP-4747: Simplify EIP-161
    /// Note: There is possibility that account is empty but its storage is not.
    /// We are storing full account is it is easier to handle.
    LoadedEmptyEIP161(PlainAccount),
    /// Account called selfdestruct and it is removed.
    /// Initial account is found in db, this would trigger removal of account from db.
    Destroyed,
    /// Account called selfdestruct on already selfdestructed account.
    DestroyedAgain,
    /// Loaded account from db.
    LoadedNotExisting,
}

impl GlobalAccountState {
    pub fn is_some(&self) -> bool {
        match self {
            GlobalAccountState::Changed(_) => true,
            GlobalAccountState::New(_) => true,
            GlobalAccountState::NewChanged(_) => true,
            GlobalAccountState::DestroyedNew(_) => true,
            GlobalAccountState::DestroyedNewChanged(_) => true,
            _ => false,
        }
    }

    pub fn storage_slot(&self, storage_key: U256) -> Option<U256> {
        self.account()
            .and_then(|a| a.storage.get(&storage_key).map(|slot| slot.present_value))
    }

    pub fn account_info(&self) -> Option<AccountInfo> {
        self.account().map(|a| a.info.clone())
    }

    pub fn account(&self) -> Option<&PlainAccount> {
        match self {
            GlobalAccountState::Loaded(account) => Some(account),
            GlobalAccountState::Changed(account) => Some(account),
            GlobalAccountState::New(account) => Some(account),
            GlobalAccountState::NewChanged(account) => Some(account),
            GlobalAccountState::DestroyedNew(account) => Some(account),
            GlobalAccountState::DestroyedNewChanged(account) => Some(account),
            GlobalAccountState::LoadedEmptyEIP161(account) => Some(account),
            GlobalAccountState::Destroyed
            | GlobalAccountState::DestroyedAgain
            | GlobalAccountState::LoadedNotExisting => None,
        }
    }

    pub fn touch_empty(&mut self) {
        *self = match self {
            GlobalAccountState::DestroyedNew(_) => GlobalAccountState::DestroyedAgain,
            GlobalAccountState::New(_) => {
                // account can be created empty them touched.
                // Note: we can probably set it to LoadedNotExisting.
                GlobalAccountState::Destroyed
            }
            GlobalAccountState::LoadedEmptyEIP161(_) => GlobalAccountState::Destroyed,
            _ => {
                // do nothing
                unreachable!("Wrong state transition, touch empty is not possible from {self:?}");
            }
        }
    }
    /// Consume self and make account as destroyed.
    pub fn selfdestruct(&mut self) {
        *self = match self {
            GlobalAccountState::DestroyedNew(_) | GlobalAccountState::DestroyedNewChanged(_) => {
                GlobalAccountState::DestroyedAgain
            }
            GlobalAccountState::Destroyed => {
                // mark as destroyed again, this can happen if account is created and
                // then selfdestructed in same block.
                // Note: there is no big difference between Destroyed and DestroyedAgain
                // in this case, but was added for clarity.
                GlobalAccountState::DestroyedAgain
            }
            _ => GlobalAccountState::Destroyed,
        };
    }
    pub fn newly_created(&mut self, new: AccountInfo, storage: &Storage) {
        *self = match self {
            // if account was destroyed previously just copy new info to it.
            GlobalAccountState::DestroyedAgain | GlobalAccountState::Destroyed => {
                GlobalAccountState::DestroyedNew(PlainAccount {
                    info: new,
                    storage: HashMap::new(),
                })
            }
            // if account is loaded from db.
            GlobalAccountState::LoadedNotExisting => GlobalAccountState::New(PlainAccount {
                info: new,
                storage: storage.clone(),
            }),
            GlobalAccountState::LoadedEmptyEIP161(_) | GlobalAccountState::Loaded(_) => {
                // if account is loaded and not empty this means that account has some balance
                // this does not mean that accoun't can be created.
                // We are assuming that EVM did necessary checks before allowing account to be created.
                GlobalAccountState::New(PlainAccount {
                    info: new,
                    storage: storage.clone(),
                })
            }
            _ => unreachable!(
                "Wrong state transition to create:\nfrom: {:?}\nto: {:?}",
                self, new
            ),
        };
    }
    pub fn change(&mut self, new: AccountInfo, storage: Storage) {
        //println!("\nCHANGE:\n    FROM: {self:?}\n    TO: {new:?}");
        let transfer = |this_account: &mut PlainAccount| -> PlainAccount {
            let mut this_storage = core::mem::take(&mut this_account.storage);
            // TODO save original value and dont overwrite it.
            this_storage.extend(storage.into_iter());
            PlainAccount {
                info: new,
                storage: this_storage,
            }
        };
        *self = match self {
            GlobalAccountState::Loaded(this_account) => {
                // If account was initially loaded we are just overwriting it.
                // We are not checking if account is changed.
                // storage can be.
                GlobalAccountState::Changed(transfer(this_account))
            }
            GlobalAccountState::Changed(this_account) => {
                // Update to new changed state.
                GlobalAccountState::Changed(transfer(this_account))
            }
            GlobalAccountState::New(this_account) => {
                // promote to NewChanged.
                // Check if account is empty is done outside of this fn.
                GlobalAccountState::NewChanged(transfer(this_account))
            }
            GlobalAccountState::NewChanged(this_account) => {
                // Update to new changed state.
                GlobalAccountState::NewChanged(transfer(this_account))
            }
            GlobalAccountState::DestroyedNew(this_account) => {
                // promote to DestroyedNewChanged.
                GlobalAccountState::DestroyedNewChanged(transfer(this_account))
            }
            GlobalAccountState::DestroyedNewChanged(this_account) => {
                // Update to new changed state.
                GlobalAccountState::DestroyedNewChanged(transfer(this_account))
            }

            GlobalAccountState::LoadedEmptyEIP161(this_account) => {
                // Change on empty account, should transfer storage if there is any.
                GlobalAccountState::Changed(transfer(this_account))
            }
            GlobalAccountState::LoadedNotExisting
            | GlobalAccountState::Destroyed
            | GlobalAccountState::DestroyedAgain => {
                unreachable!("Wronge state transition change: \nfrom:{self:?}")
            }
        }
    }

    /// Update to new state and generate RevertState that if applied to new state will
    /// revert it to previous state. If not revert is present, update is noop.
    pub fn update_and_create_revert(&mut self, main_update: Self) -> Option<RevertState> {
        // Helper function that exploads account and returns revert state.
        let make_it_explode =
            |original_status: AccountStatus, this: &mut PlainAccount| -> Option<RevertState> {
                let previous_account = this.info.clone();
                // Take present storage values as the storages that we are going to revert to.
                // As those values got destroyed.
                let previous_storage = this
                    .storage
                    .drain()
                    .into_iter()
                    .map(|(key, value)| (key, RevertToSlot::Some(value.present_value)))
                    .collect();
                let revert = Some(RevertState {
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
                                                this: &mut PlainAccount,
                                                destroyed_storage: HashMap<U256, RevertToSlot>|
         -> Option<RevertState> {
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
            let revert = Some(RevertState {
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
                .map(|(key, value)| (*key, RevertToSlot::Destroyed))
                .collect()
        };

        // handle it more optimal in future but for now be more flexible to set the logic.
        let previous_storage_from_update = main_update
            .account()
            .map(|a| {
                a.storage
                    .iter()
                    .filter(|s| s.1.original_value != s.1.present_value)
                    .map(|(key, value)| (*key, RevertToSlot::Some(value.original_value.clone())))
                    .collect()
            })
            .unwrap_or_default();

        match main_update {
            GlobalAccountState::Changed(update) => match self {
                GlobalAccountState::Changed(this) => {
                    // extend the storage. original values is not used inside bundle.
                    this.storage.extend(update.storage);
                    this.info = update.info;
                    return Some(RevertState {
                        account: Some(this.info.clone()),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::Changed,
                    });
                }
                GlobalAccountState::Loaded(this) => {
                    // extend the storage. original values is not used inside bundle.
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);
                    let previous_account = this.info.clone();
                    *self = GlobalAccountState::Changed(PlainAccount {
                        info: update.info,
                        storage,
                    });
                    return Some(RevertState {
                        account: Some(previous_account),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::Loaded,
                    });
                } //discard changes
                _ => unreachable!("Invalid state"),
            },
            GlobalAccountState::New(update) => {
                // this state need to be loaded from db
                match self {
                    GlobalAccountState::LoadedEmptyEIP161(this) => {
                        let mut storage = core::mem::take(&mut this.storage);
                        storage.extend(update.storage);
                        *self = GlobalAccountState::New(PlainAccount {
                            info: update.info,
                            storage: storage,
                        });
                        return Some(RevertState {
                            account: Some(AccountInfo::default()),
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedEmptyEIP161,
                        });
                    }
                    GlobalAccountState::LoadedNotExisting => {
                        *self = GlobalAccountState::New(update.clone());
                        return Some(RevertState {
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    _ => unreachable!("Invalid state"),
                }
            }
            GlobalAccountState::NewChanged(update) => match self {
                GlobalAccountState::LoadedEmptyEIP161(this) => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);
                    // set as new as we didn't have that transition
                    *self = GlobalAccountState::New(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertState {
                        account: Some(AccountInfo::default()),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedEmptyEIP161,
                    });
                }
                GlobalAccountState::LoadedNotExisting => {
                    // set as new as we didn't have that transition
                    *self = GlobalAccountState::New(update.clone());
                    return Some(RevertState {
                        account: None,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::LoadedNotExisting,
                    });
                }
                GlobalAccountState::New(this) => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);

                    let previous_account = this.info.clone();
                    // set as new as we didn't have that transition
                    *self = GlobalAccountState::NewChanged(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertState {
                        account: Some(previous_account),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::New,
                    });
                }
                GlobalAccountState::NewChanged(this) => {
                    let mut storage = core::mem::take(&mut this.storage);
                    storage.extend(update.storage);

                    let previous_account = this.info.clone();
                    // set as new as we didn't have that transition
                    *self = GlobalAccountState::NewChanged(PlainAccount {
                        info: update.info,
                        storage: storage,
                    });
                    return Some(RevertState {
                        account: Some(previous_account),
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::NewChanged,
                    });
                }
                _ => unreachable!("Invalid state"),
            },
            GlobalAccountState::Loaded(_update) => {
                // No changeset, maybe just update data
                // Do nothing for now.
                return None;
            }
            GlobalAccountState::LoadedNotExisting => {
                // Not changeset, maybe just update data.
                // Do nothing for now.
                return None;
            }
            GlobalAccountState::LoadedEmptyEIP161(_update) => {
                // No changeset maybe just update data.
                // Do nothing for now
                return None;
            }
            GlobalAccountState::Destroyed => {
                let ret = match self {
                    GlobalAccountState::NewChanged(this) => {
                        make_it_explode(AccountStatus::NewChanged, this)
                    }
                    GlobalAccountState::New(this) => make_it_explode(AccountStatus::New, this),
                    GlobalAccountState::Changed(this) => {
                        make_it_explode(AccountStatus::Changed, this)
                    }
                    GlobalAccountState::LoadedEmptyEIP161(this) => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this)
                    }
                    GlobalAccountState::Loaded(this) => {
                        make_it_explode(AccountStatus::LoadedEmptyEIP161, this)
                    }
                    GlobalAccountState::LoadedNotExisting => {
                        // Do nothing as we have LoadedNotExisting -> Destroyed (It is noop)
                        None
                    }
                    _ => unreachable!("Invalid state"),
                };

                // set present to destroyed.
                *self = GlobalAccountState::Destroyed;
                return ret;
            }
            GlobalAccountState::DestroyedNew(update) => {
                // Previous block created account
                // (It was destroyed on previous block or one before).
                let ret = match self {
                    GlobalAccountState::NewChanged(this) => make_it_expload_with_aftereffect(
                        AccountStatus::NewChanged,
                        this,
                        destroyed_storage(&update),
                    ),
                    GlobalAccountState::New(this) => make_it_expload_with_aftereffect(
                        // Previous block created account, this block destroyed it and created it again.
                        // This means that bytecode get changed.
                        AccountStatus::New,
                        this,
                        destroyed_storage(&update),
                    ),
                    GlobalAccountState::Changed(this) => make_it_expload_with_aftereffect(
                        AccountStatus::Changed,
                        this,
                        destroyed_storage(&update),
                    ),
                    GlobalAccountState::Destroyed => Some(RevertState {
                        account: None,
                        storage: previous_storage_from_update,
                        original_status: AccountStatus::Destroyed,
                    }),
                    GlobalAccountState::LoadedEmptyEIP161(this) => {
                        make_it_expload_with_aftereffect(
                            AccountStatus::LoadedEmptyEIP161,
                            this,
                            destroyed_storage(&update),
                        )
                    }
                    GlobalAccountState::LoadedNotExisting => {
                        // we can make self to be New
                        // Example of loaded empty -> New -> destroyed -> New.
                        // Is same as just empty -> New
                        *self = GlobalAccountState::New(update.clone());
                        return Some(RevertState {
                            // empty account
                            account: None,
                            storage: previous_storage_from_update,
                            original_status: AccountStatus::LoadedNotExisting,
                        });
                    }
                    GlobalAccountState::Loaded(this) => make_it_expload_with_aftereffect(
                        AccountStatus::Loaded,
                        this,
                        destroyed_storage(&update),
                    ),
                    GlobalAccountState::DestroyedAgain => make_it_expload_with_aftereffect(
                        // destroyed again will set empty account.
                        AccountStatus::DestroyedAgain,
                        &mut PlainAccount::default(),
                        destroyed_storage(&update),
                    ),
                    GlobalAccountState::DestroyedNew(_this) => {
                        // From DestroyeNew -> DestroyedAgain -> DestroyedNew
                        // Note: how to handle new bytecode changed?
                        // TODO
                        return None;
                    }
                    _ => unreachable!("Invalid state"),
                };
                *self = GlobalAccountState::DestroyedNew(update);
                return ret;
            }
            GlobalAccountState::DestroyedNewChanged(update) => match self {
                GlobalAccountState::NewChanged(this) => {}
                GlobalAccountState::New(this) => {}
                GlobalAccountState::Changed(this) => {}
                GlobalAccountState::Destroyed => {}
                GlobalAccountState::LoadedEmptyEIP161(this) => {}
                GlobalAccountState::LoadedNotExisting => {}
                GlobalAccountState::Loaded(this) => {}
                _ => unreachable!("Invalid state"),
            },
            GlobalAccountState::DestroyedAgain => match self {
                GlobalAccountState::NewChanged(this) => {}
                GlobalAccountState::New(this) => {}
                GlobalAccountState::Changed(this) => {}
                GlobalAccountState::Destroyed => {}
                GlobalAccountState::DestroyedNew(this) => {}
                GlobalAccountState::DestroyedNewChanged(this) => {}
                GlobalAccountState::DestroyedAgain => {}
                GlobalAccountState::LoadedEmptyEIP161(this) => {}
                GlobalAccountState::LoadedNotExisting => {}
                GlobalAccountState::Loaded(this) => {}
                _ => unreachable!("Invalid state"),
            },
        }

        None
    }
}

// TODO
pub struct StateWithChange {
    /// State
    pub state: BlockState,
    /// Changes to revert
    pub change: Vec<Vec<GlobalAccountState>>,
}

/*
This is three way comparison

database storage, relevant only for selfdestruction.
Original state (Before block): Account::new.
Present state (Present world state): Account::NewChanged.
New state (New world state inside same block): Account::NewChanged
PreviousValue: All info that is needed to revert new state.

We have first interaction when creating changeset.
Then we need to update changeset, updating is crazy, should we just think about it
as original -> new and ignore intermediate state?

How should we think about this.
* Revert to changed state is maybe most appropriate as it tell us what is original state.
---* Revert from state can be bad as from state gets changed.


* For every Revert we need to think how changeset is going to look like.

Example if account gets destroyed but was changed, we need to make it as destroyed
and we need to apply previous storage to it as storage can contains changed from new storage.

Additionaly we should have additional storage from present state

We want to revert to NEW this means rewriting info (easy) but for storage.


If original state is new but it gets destroyed, what should we do.
 */

/*
New one:

Confusing think for me is to what to do when selfdestruct happen and little bit for
how i should think about reverts.
 */

/*
Example

State:
1: 02
2: 10
3: 50
4: 1000 (some random value)
5: 0 nothing.

Block1:
* Change1:
    1: 02->03
    2: 10->20

World Change1:
    1: 03
    2: 20

Block2:
* Change2:
    1: 03->04
    2: 20->30
RevertTo is Change1:
    1: 03, 2: 20.
* Change3:
    3: 50->51
RevertTo is Change1:
    1: 03, 2: 20, 3: 50. Append changes
* Destroyed:
    RevertTo is same. Maybe we can remove zeroes from RevertTo
    When applying selfdestruct to state, read all storage, and then additionaly
    apply Change1 RevertTo.
* DestroyedNew:
    1: 0->5
    3: 0->52
    4: 0->100
    5: 0->999
    This is tricky, here we have slot 4 that potentially has some value in db.
Generate state for old world to new world.

RevertTo is simple when comparing old and new state. As we dont think about full database storage.
Changeset is tricky.
For changeset we want to have
    1: 03
    2: 20
    3: 50
    5: 1000

We need old world state, and that is only thing we need.
We use destroyed storage and apply only state on it, aftr that we need to append
DestroyedNew storage zeroes.




So it can be Some or destroyed.


database has: [02,10,50,1000,0]

WorldState:
DestroyedNew:
    1: 5
    3: 52

Original state Block1:
    Change1:

RevertTo Block2:
    This is Change1 state we want to get:
        1: 03
        2: 20
    We need to:
        Change 1: 05->03
        Change 2: 0->20
        Change 3: 52->0
 */

/// Assumption is that Revert can return full state from any future state to any past state.
///
/// It is created when new account state is applied to old account state.
/// And it is used to revert new account state to the old account state.
///
/// RevertState is structured in this way as we need to save it inside database.
/// And we need to be able to read it from database.
#[derive(Clone, Default)]
pub struct RevertState {
    account: Option<AccountInfo>,
    storage: HashMap<U256, RevertToSlot>,
    original_status: AccountStatus,
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

#[derive(Clone, Default)]
pub enum AccountStatus {
    #[default]
    LoadedNotExisting,
    Loaded,
    LoadedEmptyEIP161,
    Changed,
    New,
    NewChanged,
    Destroyed,
    DestroyedNew,
    DestroyedNewChanged,
    DestroyedAgain,
}

/// Previous values needs to have all information needed to revert any plain account
/// and storage changes. This means that we need to compare previous state with new state
/// And if storage was first set to the state we need to put zero to cancel it on revert.
///
/// Additionaly we should have information on previous state enum of account, so we can set it.
///
pub enum RevertTo {
    /// Revert to account info, and revert all set storages.
    /// On any new state old storage is needed. Dont insert storage after selfdestruct.
    Loaded(PlainAccount),
    /// NOTE Loaded empty can still contains storage. Edgecase when crate returns empty account
    /// but it sent storage on init
    LoadedEmptyEIP161(Storage),
    /// For revert, Delete account
    LoadedNotExisting,

    /// Account is marked as newly created and multiple NewChanges are aggregated into one.
    /// Changeset will containd None, and storage will contains only zeroes.
    ///
    /// Previous values of account state is:
    /// For account is Empty account
    /// For storage is set of zeroes/empty to cancel any set storage.
    ///
    /// If it is loaded empty we need to mark is as such.
    New {
        // Should be HashSet
        storage: Storage,
        was_loaded_empty_eip161: bool,
    },
    /// Account is originaly changed.
    /// Only if new state is. Changed
    ///
    /// Previous values of account state is:
    /// For account is previous changed account.
    /// For storage is set of zeroes/empty to cancel any set storage.
    ///
    /// `was_loaded_previosly` is set to true if account had Loaded state.
    Changed {
        account: PlainAccount, // old account and previous storage
        was_loaded_previosly: bool,
    },
    /// Account is destroyed. All storages need to be removed from state
    /// and inserted into changeset.
    ///
    /// if original bundle state is any of previous values
    /// And new state is `Destroyed`.
    ///
    /// If original bundle state is changed we need to save change storage
    Destroyed {
        address: B160,
        old_account: PlainAccount, // old storage that got updated.
    },
    /// If account is newly created but it was already destroyed earlier in the bundle.
    DestroyedNew {
        address: B160,
        old_storage: Storage, // Set zeros for every plain storage entry
    },
    // DestroyedNewChanged {
    //     address: B160,
    //     old_storage: Storage, // Set zeros for every plain storage entry
    // }
}

impl StateWithChange {
    pub fn apply_block_substate_and_create_reverts(
        &mut self,
        block_state: BlockState,
    ) -> Vec<RevertState> {
        let reverts = Vec::new();
        for (address, block_account) in block_state.accounts.into_iter() {
            match self.state.accounts.entry(address) {
                hash_map::Entry::Occupied(entry) => {
                    let this_account = entry.get();
                }
                hash_map::Entry::Vacant(entry) => {
                    // TODO what to set here, just update i guess
                }
            }
        }
        reverts
    }
}

/*

Transtion Needs to contains both old global state and new global state.

If it is from LoadedEmpty to Destroyed is a lot different if it is from New -> Destroyed.


pub struct Change {
    old_state: GlobalAccountState,
}

pub struct StateWithChange {
    global_state: GlobalAccountState,
    changeset: Change,
}

database state:
* Changed(Acccount)


Action:
* SelfDestructed

New state:
* SelfDestructed (state cleared)


If it is previous block Changed(Account)->SelfDestructed is saved

If it is same block it means that one of changes already happened so we need to switch it
Loaded->Changed needs to become Loaded->SelfDestructed

Now we have two parts here, one is inside block as in merging change selfdestruct:
For this We need to devour Changes and set it to


And second is if `Change` is part of previous changeset.


What do we need to have what paths we need to cover.

First one is transaction execution from EVM. We got this one!

Second one is block execution and aggregation of transction changes.
We need to generate changesets for it

Third is multi block execution and their changesets. This part is needed to
flush bundle of block changed to db and for tree.

Is third way not needed? Or asked differently is second way enought as standalone
 to be used inside third way.



For all levels there is two parts, global state and changeset.

Global state is applied to plain state, it need to contain only new values and if it is first selfdestruct.

ChangeSet needs to have all info to revert global state to scope of the block.


So comming back for initial problem how to set Changed -> SelfDestructed change inside one block.
Should we add notion of transitions,

My hunch is telling me that there is some abstraction that we are missing and that we need to
saparate our thinking on current state and changeset.

Should we have AccountTransition as a way to model transition between global states.
This would allow us to have more consise way to apply and revert changes.

it is a big difference when we model changeset that are on top of plain state or
if it is on top of previous changeset. As we have more information inside changeset with
comparison with plain state, we have both (If it is new, and if it is destroyed).

Both new and destroyed means that we dont look at the storage.

*/

/*

Changed -> SelfDestructedNew

 */

/*
how to handle it


 */

/*
ChangeSet


All pair of transfer


Loaded -> New
Loaded -> New -> Changed
Loaded -> New -> Changed -> SelfDestructed
Loaded -> New -> Changed -> SelfDestructed -> loop


ChangeSet ->
Loaded
SelfDestructed



    Destroyed --> DestroyedNew
    Changed --> Destroyed
    Changed --> Changed
    New --> Destroyed
    New --> Changed
    DestroyedNew --> DestroyedNewChanged
    DestroyedNewChanged --> Destroyed
    DestroyedNew --> Destroyed
    Loaded --> Destroyed : destroyed
    Loaded --> Changed : changed
    Loaded --> New : newly created



 */

/*
* Mark it for selfdestruct.
* Touch but not change account.
    For empty accounts (State clear EIP):
        * before spurious dragon create account
        * after spurious dragon remove account if present inside db ignore otherwise.
* Touch and change account. Nonce, balance or code
* Created newly created account (considered touched).
 */

/*
Model step by step transition between account states.

Main problem is how to go from

Block 1:
LoadedNotExisting -> New

Changeset is obvious it is LoadedNotExisting enum.

Block 2:

New -> Changed
Changed -> Changed
Changed -> Destroyed

Not to desect this
New -> Changed
There is not changeset here.
So changeset need to be changed to revert back any storage and
balance that we have changed

Changed -> Changed
So changeset is Changed and we just need to update the balance
and nonce and updated storage.

Changed -> Destroyed
Destroyed is very interesting here.

What do we want, selfdestructs removes any storage from database

But for revert previous state is New but Changed -> Changed is making storage dirty with other changes.

So we do need to have old state, transitions and new state. so that transitions can be reverted if needed.

Main thing here is that we have global state, and we need to think what data do we need to revert it to previos state.


So new global state is now Destroyed and we need to be able revert it to the New but present global state is Changed.

What do we need to revert from Destroyed --> to New

There is option to remove destroyed storage and just add new storage. And
There is option of setting all storages to ZERO.

Storage is main problem how to handle it.


BREAKTHROUGH: Have first state, transition and present state.
This would help us with reverting of the state as we just need to replace the present state
with first state. First state can potentialy be removed if revert is not needed (as in pipeline execution).

Now we can focus on transition.
Changeset is generated when present state is replaces with new state

For Focus states that we have:
* old state (State transaction start executing), It is same as present state at the start.
* present state (State after N transaction execution).
* new state (State that we want to apply to present state and update the changeset)
* transition between old state and present state

We have two transtions that we need to think about:
First transition is easy
Any other transitions need to merge one after another
We need to create transitions between present state and new state and merge it
already created transition between old and present state.


Transition need old values
Transitions {
    New -> Set Not existing
    Change -> Old change
    Destroyed -> Old account.
    NewDestroyed -> OldAccount.
    Change
}

BREAKTHROUGHT: Transition depends on old state. if old state is Destroyed or old state is New matters a lot.
If new state is NewDestroyed. In case of New transition to destroyed, transition would be new account data
, while if it is transtion between Destroyed to DestroyedNew, transition would be Empty account and storage.


Question: Can we generate changeset from old and new state.
Answer: No, unless we can match every new account with old state.

Match every new storage with old storage values is maybe way to go.

Journal has both Old Storage and New Storage. This can be a way to go.
And we already have old account and new account.


Lets simplify it and think only about account and after that think about storage as it is more difficult:


For account old state helps us to not have duplicated values on block level granularity.

For example if LoadedNotExisting and new state is Destroyed or DestroyedAgain it is noop.
Account are simple as we have old state and new state and we save old state

Storage is complex as state depends on the selfdestruct.
So transition is hard to generate as we dont have linear path.


BREAKTHROUGHT: Hm when applying state we should first apply plain state, and read old state
from database for accounts that IS DESTROYED. Only AFTER that we can apply transitions as transitions depend on storage and
diff of storage that is inside database.

This would allow us to apply plain state first and then go over transitions and apply them.

We would have original storage that is ready for selfdestruct.

PlainState ->


BREAKTHROUGHT: So algorithm of selfdestructed account need to read all storages. and use those account
when first selfdestruct appears. Other transitions already have all needed values.

for calculating changeset we need old and new account state. nothing more.

New account state would be superset of old account state
Some cases
* If old is Changed and new is Destroyed (or any destroyed):
PreviousEntry consist of full plain state storage, with ADDITION of all values of Changed state.
* if old is DestroyedNew and new is DestroyedAgain:
changeset is

CAN WE GENERATE PREVIOUS ENTRY ONLY FROM OLD AND NEW STATE.

[EVM State] Tx level, Lives for one tx
 |
 |
 v
[Block state] updated on one by one transition from tx. Lives for one block duration.
 |
 |
 v
[Bundled state] updated by block state (account can have multi state transitions)
[PreviousValues] When commiting block state generate PreviousEntry (create changesets).
 |
 |
 v
Database mdbx. Plain state

EVM State
|          \
|           \
|            [Block State]
|            |
[cachedb]    |
|            v
|            [Bundled state]
|           /
v          /
database mdbx


Insights:
* We have multiple states in execution.
    * Tx (EVM state) Used as accesslist
    * Block state
    * Bundle state (Multi blocks)
    * Database
* Block state updates happen by one transition (one TX). Transition means one connection on
mermaid graph.
* Bundle state update account by one or more transitions.
* When updating bundle we can generate ChangeSet between block state and old bundle state.
* Account can be dirrectly applied to the plain state, we need to save selfdestructed storage
as we need to append those to the changeset of first selfdestruct
* For reverts, it is best to just save old account state. Reverting becomes a lot simpler.
This can be ommited for pipeline execution as revert is not needed.
* Diff between old and new state can only happen if we have all old values or if new values
contain pair of old->new. I think second approche is better as we can ommit saving loaded values
but just changed one.


Notice that we have four levels and if we fetch values from EVM we are touching 4 hashmaps.
PreviousValues are tied together and depends on each other.

What we presently have

[EVM State] Tx level
 | \
 |  \ updates PostState with output of evm execution over multiple blocks
 v
[CacheDB] state Over multi blocks.
 |
 |
 v
 database (mdbx)

 */
