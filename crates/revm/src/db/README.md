
# Newest state of the code:

We have four states (HashMaps of accounts) and have two channels.

States are:
* EVM State. Account and original/present storage
* Cached state. Account and present values.
* Block State. Account and original/present storage
* Bundle State. Account and present storage.

Block and bundle state is used to generate reverts. While bundle and block are used to generate reverts for changesets.

Best way to think about it is that different states are different points of time.
* EVM State is transaction granular it contains contract call changes.
* Cache state is always up to date and evm state is aplied to it.
* Block state is created for new block and merged to bundle after block end.
    EVM state is aplied to it.
* Bundle state contains state before block started and it is updated when blocks state
    gets merged.

EVM State
(It has both original/present storage and new account)
(Should we have both original/present account? It is didferent as account is standalone
while storage depends on account state.)
|          \
|           \
|            [Block State] (It has original/present storage and new account).
Original storage is needed to create changeset without asking plain storage.
|            |
[cachedb]    |
|            v
|            [Bundled state] (It has only changeset and plain state, Original storage is not needed)
One of reason why this is the case is because on revert of canonical chain
we can't get previous storage value. And it is not needed.
|           /
v          /
database mdbx

# Dump of my thoughts, removing in future.

// THIS IS NOT GONA WORK.
// As revert from database does not have of previous previos values that we put here.
// original_value is needed only when merging from block to the bundle state.
// So it is not needed for plain state of the bundle. SHOULD WE REMOVE ORIGINAL VALUE?
// IT IS USED TO GENERATE REVERTS, can we go without it?

// It is obtained from tx to block merge.
// It is needed for block to bundle merge and generating changesets.




    /// Update account and generate revert. Revert can be done over multiple
    /// transtions
    /*
        We dont want to save previous state inside db as info is not needed.
        So we need to simulate it somehow.

        Idea is to use only subset of Statuses (Selfdestruct is not needed as full storage is present):
        AccountStatus::Changed // if plain state has account.
        AccountStatus::LoadedNotExisting // if revert to account is None
        AccountStatus::LoadedEmptyEIP161 // if revert to account is empty.
        AccountStatus::New if plain state does not have it, but revert is some.
        Tricky: if New is present we should make any Changed to NewChanged.
        This means we should iterate over already created account and make then NewChanged.

    */



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
