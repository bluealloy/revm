use once_cell::sync::Lazy;
use revm_interpreter::primitives::{
    db::{Database, DatabaseCommit},
    hash_map::{self, Entry},
    Account, AccountInfo, Bytecode, HashMap, State, B160, B256, KECCAK_EMPTY, U256,
};

#[derive(Clone, Debug, Default)]
pub struct PlainAccount {
    pub info: AccountInfo,
    pub storage: HashMap<U256, U256>,
}

impl From<AccountInfo> for PlainAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
        }
    }
}

static EMPTY_PLAIN_ACCOUNT: Lazy<PlainAccount> = Lazy::new(|| PlainAccount::default());

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
            has_state_clear: false,
        }
    }
    /// Legacy without state clear flag enabled
    pub fn new_legacy() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
            has_state_clear: true,
        }
    }
    /// Used for tests only. When transitioned it is not recoverable
    pub fn set_state_clear(&mut self) {
        if self.has_state_clear == true {
            return;
        }

        // mark all empty accounts as not existing
        for (address, account) in self.accounts.iter_mut() {
            // This would make LoadedEmptyEIP161 not used anymore.
            if let GlobalAccountState::LoadedEmptyEIP161 = account {
                *account = GlobalAccountState::LoadedNotExisting;
            }
        }

        self.has_state_clear = true;
    }

    pub fn trie_account(&self) -> impl IntoIterator<Item = (B160, &PlainAccount)> {
        self.accounts.iter().filter_map(|(address, account)| {
            if let GlobalAccountState::LoadedEmptyEIP161 = account {
                if self.has_state_clear {
                    return None;
                } else {
                    return Some((*address, &*EMPTY_PLAIN_ACCOUNT));
                }
            }
            if let Some(plain_acc) = account.account() {
                Some((*address, plain_acc))
            } else {
                None
            }
        })
    }

    pub fn insert_not_existing(&mut self, address: B160) {
        self.accounts
            .insert(address, GlobalAccountState::LoadedNotExisting);
    }
    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        if !info.is_empty() {
            self.accounts
                .insert(address, GlobalAccountState::Loaded(info.into()));
            return;
        }

        if self.has_state_clear {
            self.accounts
                .insert(address, GlobalAccountState::LoadedNotExisting);
        } else {
            self.accounts
                .insert(address, GlobalAccountState::LoadedEmptyEIP161);
        }
    }

    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: HashMap<U256, U256>,
    ) {
        if !info.is_empty() {
            self.accounts.insert(
                address,
                GlobalAccountState::Loaded(PlainAccount { info, storage }),
            );
            return;
        }

        if self.has_state_clear {
            self.accounts
                .insert(address, GlobalAccountState::LoadedNotExisting);
        } else {
            self.accounts
                .insert(address, GlobalAccountState::LoadedEmptyEIP161);
        }
    }

    pub fn apply_evm_state(&mut self, evm_state: &State) {
        for (address, account) in evm_state {
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
                break;
            }

            let storage = account
                .storage
                .iter()
                .map(|(k, v)| (*k, v.present_value))
                .collect::<HashMap<_, _>>();
            if account.is_newly_created() {
                // Note: it can happen that created contract get selfdestructed in same block
                // that is why is newly created is checked after selfdestructed
                //
                // TODO take care of empty account but with some storage.
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
                        this.newly_created(account.info.clone(), storage)
                    }
                    Entry::Vacant(entry) => {
                        // This means that account was not loaded through this state.
                        // and we trust that account is empty.
                        entry.insert(GlobalAccountState::New(PlainAccount {
                            info: account.info.clone(),
                            storage,
                        }));
                    }
                }
            } else {
                // account is touched, but not selfdestructed or newly created.
                // Account can be touched and not changed.
                // And when empty account is touched it needs to be removed from database.
                match self.accounts.entry(*address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.change(account.info.clone(), storage);
                    }
                    Entry::Vacant(entry) => {
                        // It is assumed initial state is Loaded
                        entry.insert(GlobalAccountState::Changed(PlainAccount {
                            info: account.info.clone(),
                            storage: storage,
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
    /// Loaded account from db.
    LoadedNotExisting,
    /// Creating empty account was only possible before SpurioudDragon hardfork
    /// And last of those account were touched (removed) from state in block 14049881.
    /// EIP-4747: Simplify EIP-161
    LoadedEmptyEIP161,
    /// Account called selfdestruct and it is removed.
    /// Initial account is found in db, this would trigger removal of account from db.
    Destroyed,
    /// Account called selfdestruct on already selfdestructed account.
    DestroyedAgain,
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
            .and_then(|a| a.storage.get(&storage_key).cloned())
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
            GlobalAccountState::LoadedEmptyEIP161 => Some(&EMPTY_PLAIN_ACCOUNT),
            GlobalAccountState::Destroyed
            | GlobalAccountState::DestroyedAgain
            | GlobalAccountState::LoadedNotExisting => None,
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
    pub fn newly_created(&mut self, new: AccountInfo, storage: HashMap<U256, U256>) {
        *self = match self {
            // if account was destroyed previously just copy new info to it.
            GlobalAccountState::DestroyedAgain | GlobalAccountState::Destroyed => {
                GlobalAccountState::DestroyedNew(PlainAccount {
                    info: new,
                    storage: HashMap::new(),
                })
            }
            // if account is loaded from db.
            GlobalAccountState::LoadedEmptyEIP161 | GlobalAccountState::LoadedNotExisting => {
                GlobalAccountState::New(PlainAccount { info: new, storage })
            }
            _ => unreachable!(
                "Wrong state transition: initial state {:?}, new state {:?}",
                self, new
            ),
        };
    }
    pub fn change(&mut self, new: AccountInfo, storage: HashMap<U256, U256>) {
        *self = match self {
            GlobalAccountState::Loaded(_) => {
                // If account was initially loaded we are just overwriting it.
                // We are not checking if account is changed.
                // as storage can be.
                GlobalAccountState::Changed(PlainAccount {
                    info: new,
                    storage: storage,
                })
            }
            GlobalAccountState::Changed(this_account) => {
                // Update to new changed state.
                let mut this_storage = core::mem::take(&mut this_account.storage);
                this_storage.extend(storage.into_iter());
                GlobalAccountState::Changed(PlainAccount {
                    info: new,
                    storage: this_storage,
                })
            }
            GlobalAccountState::New(this_account) => {
                // promote to NewChanged.
                // If account is empty it can be destroyed.
                let mut this_storage = core::mem::take(&mut this_account.storage);
                this_storage.extend(storage.into_iter());
                GlobalAccountState::NewChanged(PlainAccount {
                    info: new,
                    storage: this_storage,
                })
            }
            GlobalAccountState::NewChanged(this_account) => {
                // Update to new changed state.
                let mut this_storage = core::mem::take(&mut this_account.storage);
                this_storage.extend(storage.into_iter());
                GlobalAccountState::NewChanged(PlainAccount {
                    info: new,
                    storage: this_storage,
                })
            }
            GlobalAccountState::DestroyedNew(this_account) => {
                // promote to DestroyedNewChanged.
                // If account is empty it can be destroyed.
                let mut this_storage = core::mem::take(&mut this_account.storage);
                this_storage.extend(storage.into_iter());
                GlobalAccountState::DestroyedNewChanged(PlainAccount {
                    info: new,
                    storage: this_storage,
                })
            }
            GlobalAccountState::DestroyedNewChanged(this_account) => {
                // Update to new changed state.
                let mut this_storage = core::mem::take(&mut this_account.storage);
                this_storage.extend(storage.into_iter());
                GlobalAccountState::DestroyedNewChanged(PlainAccount {
                    info: new,
                    storage: this_storage,
                })
            }
            GlobalAccountState::LoadedNotExisting
            | GlobalAccountState::LoadedEmptyEIP161
            | GlobalAccountState::Destroyed
            | GlobalAccountState::DestroyedAgain => {
                unreachable!("Can have this transition")
            }
        }
    }
}

// TODO
pub struct StateWithChange {
    /// State
    pub state: BlockState,
    /// Changes to revert
    pub change: Vec<Vec<GlobalAccountState>>,
}

impl StateWithChange {
    pub fn apply_substate(&mut self, sub_state: BlockState) {
        for (address, account) in sub_state.accounts.into_iter() {
            match self.state.accounts.entry(address) {
                hash_map::Entry::Occupied(entry) => {
                    let this_account = entry.get();
                    match account {
                        GlobalAccountState::Changed(acc) => match this_account {
                            GlobalAccountState::Changed(this_acc) => {}
                            GlobalAccountState::Loaded(acc) => {} //discard changes
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::Destroyed => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}   // apply
                            GlobalAccountState::New(acc) => {}          // apply
                            GlobalAccountState::Changed(acc) => {}      // apply
                            GlobalAccountState::LoadedEmptyEIP161 => {} // noop
                            GlobalAccountState::LoadedNotExisting => {} // noop
                            GlobalAccountState::Loaded(acc) => {}       //noop
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::DestroyedNew(acc) => match this_account {
                            GlobalAccountState::NewChanged(acc) => {}
                            GlobalAccountState::New(acc) => {}
                            GlobalAccountState::Changed(acc) => {}
                            GlobalAccountState::Destroyed => {}
                            GlobalAccountState::LoadedEmptyEIP161 => {}
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
                            GlobalAccountState::LoadedEmptyEIP161 => {}
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
                            GlobalAccountState::LoadedEmptyEIP161 => {}
                            GlobalAccountState::LoadedNotExisting => {}
                            GlobalAccountState::Loaded(acc) => {}
                            _ => unreachable!("Invalid state"),
                        },
                        GlobalAccountState::New(acc) => {
                            // this state need to be loaded from db
                            match this_account {
                                GlobalAccountState::LoadedEmptyEIP161 => {}
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
                        GlobalAccountState::LoadedEmptyEIP161 => {}
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
