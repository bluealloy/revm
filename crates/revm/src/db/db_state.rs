use revm_interpreter::primitives::{hash_map, Account, AccountInfo, HashMap, B160, U256};

pub struct ClotAccount {
    info: AccountInfo,
    storage: HashMap<U256, U256>,
}

/// This is action on state.
pub enum GlobalAccountState {
    /// Loaded from db
    Loaded(ClotAccount),
    /// Account was present and it got changed from db
    Changed(ClotAccount),
    /// Account is not found inside db and it is newly created
    New(ClotAccount),
    /// New account that got changed
    NewChanged(ClotAccount),
    /// Account created that was previously destroyed
    DestroyedNew(ClotAccount),
    /// Account changed that was previously destroyed then created.
    DestroyedNewChanged(ClotAccount),
    /// Loaded account from db.
    LoadedNotExisting,
    /// Loaded empty account
    LoadedEmpty,
    /// Account called selfdestruct and it is removed.
    /// Initial account is found in db, this would trigger removal of account from db.
    Destroyed,
    /// Account called selfdestruct on already selfdestructed account.
    DestroyedAgain,
}

pub enum ChangedState {
    NewChanged,
    Changed,
    DestroyedNewChanged,
}

pub enum Transitions {
    LoadedEmpty,
    LoadedNotExisting,
    Loaded(ClotAccount),
    New {
        account: ClotAccount, // old state
        is_destroyed: bool,
    },
    Changed {
        account: ClotAccount, // old state
        change_state: ChangedState,
    },
    Destroyed,
    DestroyedAgain,
}

pub enum Change {
    AccountChange { old: AccountInfo },
    StorageChange { old: bool },
}

pub struct SubState {
    /// Global state
    state: HashMap<B160, GlobalAccountState>,
}

impl SubState {}

pub struct StateWithChange {
    /// State
    pub state: SubState,
    /// Changes to revert
    pub change: Vec<Vec<Change>>,
}

impl StateWithChange {
    pub fn apply_substate(&mut self, sub_state: SubState) {
        for (address, account) in sub_state.state.into_iter() {
            match self.state.state.entry(address) {
                hash_map::Entry::Occupied(entry) => {
                    let this_account = entry.get();
                    match account {
                        GlobalAccountState::Changed(acc) => match this_account {
                            GlobalAccountState::Changed(this_acc) => {}
                            GlobalAccountState::Loaded(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::Destroyed => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}
                            GlobalAccountState::New(acc) => {}
                            GlobalAccountState::Changed(acc) => {}
                            GlobalAccountState::LoadedEmpty => {}
                            GlobalAccountState::LoadedNotExisting => {}
                            GlobalAccountState::Loaded(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::DestroyedNew(acc) => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}
                            GlobalAccountState::New(acc) => {}
                            GlobalAccountState::Changed(acc) => {}
                            GlobalAccountState::Destroyed => {}
                            GlobalAccountState::LoadedEmpty => {}
                            GlobalAccountState::LoadedNotExisting => {}
                            GlobalAccountState::Loaded(acc) => {}
                            GlobalAccountState::DestroyedAgain => {}
                            GlobalAccountState::DestroyedNew(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::DestroyedNewChanged(acc) => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}
                            GlobalAccountState::New(acc) => {}
                            GlobalAccountState::Changed(acc) => {}
                            GlobalAccountState::Destroyed => {}
                            GlobalAccountState::LoadedEmpty => {}
                            GlobalAccountState::LoadedNotExisting => {}
                            GlobalAccountState::Loaded(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::DestroyedAgain => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}
                            GlobalAccountState::New(acc) => {}
                            GlobalAccountState::Changed(acc) => {}
                            GlobalAccountState::Destroyed => {}
                            GlobalAccountState::DestroyedNew(acc) => {}
                            GlobalAccountState::DestroyedNewChanged(acc) => {}
                            GlobalAccountState::DestroyedAgain => {}
                            GlobalAccountState::LoadedEmpty => {}
                            GlobalAccountState::LoadedNotExisting => {}
                            GlobalAccountState::Loaded(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::New(acc) => {
                            // this state need to be loaded from db
                            match this_account {
                                GlobalAccountState::LoadedEmpty => {}
                                GlobalAccountState::LoadedNotExisting => {}
                                _ => unreachable!("Invalid state"),
                            }
                        }
                        GlobalAccountState::NewChanged(acc) => match this_account {
                            GlobalAccountState::New(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::Loaded(acc) => {}
                        GlobalAccountState::LoadedNotExisting => {}
                        GlobalAccountState::LoadedEmpty => {}
                    }
                }
                hash_map::Entry::Vacant(entry) => {}
            }
        }
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
if it is on top of previous changeset. As we have moro information inside changeset with
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
