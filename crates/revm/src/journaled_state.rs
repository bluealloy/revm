use crate::interpreter::{inner_models::SelfDestructResult, InstructionResult};
use crate::primitives::{
    db::Database, hash_map::Entry, Account, Bytecode, HashMap, Log, State, StorageSlot, B160,
    KECCAK_EMPTY, U256,
};
use alloc::{vec, vec::Vec};
use core::mem::{self};
use revm_interpreter::primitives::Spec;
use revm_interpreter::primitives::SpecId::SPURIOUS_DRAGON;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournaledState {
    /// Current state.
    pub state: State,
    /// logs
    pub logs: Vec<Log>,
    /// how deep are we in call stack.
    pub depth: usize,
    /// journal with changes that happened between calls.
    pub journal: Vec<Vec<JournalEntry>>,
    /// Ethereum before EIP-161 differently defined empty and not-existing account
    /// so we need to take care of that difference. Set this to false if you are handling
    /// legacy transactions
    pub is_before_spurious_dragon: bool,
    /// It is assumed that precompiles start from 0x1 address and spand next N addresses.
    /// we are using that assumption here
    pub num_of_precompiles: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalEntry {
    /// Used to mark account that is hot inside EVM in regards to EIP-2929 AccessList.
    /// Action: We will add Account to state.
    /// Revert: we will remove account from state.
    AccountLoaded { address: B160 },
    /// Mark account to be destroyed and journal balance to be reverted
    /// Action: Mark account and transfer the balance
    /// Revert: Unmark the account and transfer balance back
    AccountDestroyed {
        address: B160,
        target: B160,
        was_destroyed: bool, // if account had already been destroyed before this journal entry
        had_balance: U256,
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched { address: B160 },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer { from: B160, to: B160, balance: U256 },
    /// Increment nonce
    /// Action: Increment nonce by one
    /// Revert: Decrement nonce by one
    NonceChange {
        address: B160, //geth has nonce value,
    },
    /// It is used to track both storage change and hot load of storage slot. For hot load in regard
    /// to EIP-2929 AccessList had_value will be None
    /// Action: Storage change or hot load
    /// Revert: Revert to previous value or remove slot from storage
    StorageChange {
        address: B160,
        key: U256,
        had_value: Option<U256>, //if none, storage slot was cold loaded from db and needs to be removed
    },
    /// Code changed
    /// Action: Account code changed
    /// Revert: Revert to previous bytecode.
    CodeChange { address: B160, had_code: Bytecode },
}

/// SubRoutine checkpoint that will help us to go back from this
pub struct JournalCheckpoint {
    log_i: usize,
    journal_i: usize,
}

impl JournaledState {
    /// Create new JournaledState.
    ///
    /// num_of_precompiles is used to determine how many precompiles are there.
    /// Assumption is that number of N first addresses are precompiles (exclusing 0x00..00)
    ///
    /// Note: This function will journal state after Spurious Dragon fork.
    /// And will not take into account if account is not existing or empty.
    pub fn new(num_of_precompiles: usize) -> JournaledState {
        Self {
            state: HashMap::new(),
            logs: Vec::new(),
            journal: vec![vec![]],
            depth: 0,
            is_before_spurious_dragon: false,
            num_of_precompiles,
        }
    }

    /// Same as [`Self::new`] but will journal state before Spurious Dragon fork.
    ///
    /// Note: Before Spurious Dragon fork empty and not existing accounts were treated differently.
    pub fn new_legacy(num_of_precompiles: usize) -> JournaledState {
        let mut journal = Self::new(num_of_precompiles);
        journal.is_before_spurious_dragon = true;
        journal
    }

    /// Return reference to state.
    pub fn state(&mut self) -> &mut State {
        &mut self.state
    }

    /// Mark account as touched as only touched accounts will be added to state.
    /// This is expecially important for state clear where touched empty accounts needs to
    /// be removed from state.
    pub fn touch(&mut self, address: &B160) {
        if let Some(account) = self.state.get_mut(address) {
            Self::touch_account(self.journal.last_mut().unwrap(), address, account);
        }
    }

    fn touch_account(journal: &mut Vec<JournalEntry>, address: &B160, account: &mut Account) {
        if !account.is_touched() {
            journal.push(JournalEntry::AccountTouched { address: *address });
            account.mark_touch();
        }
    }

    /// do cleanup and return modified state
    pub fn finalize(&mut self) -> (State, Vec<Log>) {
        let state = mem::take(&mut self.state);

        let logs = mem::take(&mut self.logs);
        self.journal = vec![vec![]];
        self.depth = 0;
        (state, logs)
    }

    /// Use it with load_account function.
    pub fn account(&self, address: B160) -> &Account {
        self.state.get(&address).unwrap() // Always assume that acc is already loaded
    }

    pub fn depth(&self) -> u64 {
        self.depth as u64
    }

    /// use it only if you know that acc is hot
    /// Assume account is hot
    pub fn set_code(&mut self, address: B160, code: Bytecode) {
        let account = self.state.get_mut(&address).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), &address, account);

        self.journal
            .last_mut()
            .unwrap()
            .push(JournalEntry::CodeChange {
                address,
                had_code: code.clone(),
            });

        account.info.code_hash = code.hash();
        account.info.code = Some(code);
    }

    pub fn inc_nonce(&mut self, address: B160) -> Option<u64> {
        let account = self.state.get_mut(&address).unwrap();
        // Check if nonce is going to overflow.
        if account.info.nonce == u64::MAX {
            return None;
        }
        Self::touch_account(self.journal.last_mut().unwrap(), &address, account);
        self.journal
            .last_mut()
            .unwrap()
            .push(JournalEntry::NonceChange { address });

        account.info.nonce += 1;

        Some(account.info.nonce)
    }

    pub fn transfer<DB: Database>(
        &mut self,
        from: &B160,
        to: &B160,
        balance: U256,
        db: &mut DB,
    ) -> Result<(), InstructionResult> {
        // load accounts
        self.load_account(*from, db)
            .map_err(|_| InstructionResult::FatalExternalError)?;

        self.load_account(*to, db)
            .map_err(|_| InstructionResult::FatalExternalError)?;

        // sub balance from
        let from_account = &mut self.state.get_mut(from).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), from, from_account);
        let from_balance = &mut from_account.info.balance;
        *from_balance = from_balance
            .checked_sub(balance)
            .ok_or(InstructionResult::OutOfFund)?;

        // add balance to
        let to_account = &mut self.state.get_mut(to).unwrap();
        Self::touch_account(self.journal.last_mut().unwrap(), to, to_account);
        let to_balance = &mut to_account.info.balance;
        *to_balance = to_balance
            .checked_add(balance)
            .ok_or(InstructionResult::OverflowPayment)?;
        // Overflow of U256 balance is not possible to happen on mainnet. We dont bother to return funds from from_acc.

        self.journal
            .last_mut()
            .unwrap()
            .push(JournalEntry::BalanceTransfer {
                from: *from,
                to: *to,
                balance,
            });

        Ok(())
    }

    /// Create account or return false if collision is detected.
    ///
    /// There are few steps done:
    /// 1. Make created account hot loaded (AccessList) and this should
    ///     be done before subrouting checkpoint is created.
    /// 2. Check if there is colission of newly created account with existing one.
    /// 3. Mark created account as created.
    /// 4. Add fund to created account
    /// 5. Increment nonce of created account if SpuriousDragon is active
    /// 6. Decrease balance of caller account.
    ///  
    /// Safety: It is assumed that caller balance is already checked and that
    /// caller is already loaded inside evm. This is already done inside `create_inner`
    pub fn create_account_checkpoint<SPEC: Spec>(
        &mut self,
        caller: B160,
        address: B160,
        balance: U256,
    ) -> Result<JournalCheckpoint, InstructionResult> {
        // Enter subroutine
        let checkpoint = self.checkpoint();

        // Newly created account is present, as we just loaded it.
        let account = self.state.get_mut(&address).unwrap();
        let last_journal = self.journal.last_mut().unwrap();

        // check if it is possible to create this account.
        if Self::check_account_collision(address, account, self.num_of_precompiles) {
            self.checkpoint_revert(checkpoint);
            return Err(InstructionResult::CreateCollision);
        }

        // set account status to created.
        account.mark_created();
        account.info.code = None;

        // Set all storages to default value. They need to be present to act as accessed slots in access list.
        // it shouldn't be possible for them to have different values then zero as code is not existing for this account,
        // but because tests can change that assumption we are doing it.
        let empty = StorageSlot::default();
        account
            .storage
            .iter_mut()
            .for_each(|(_, slot)| *slot = empty.clone());

        // touch account. This is important as for pre SpuriousDragon account could be
        // saved even empty.
        Self::touch_account(last_journal, &address, account);

        // Add balance to created account, as we already have target here.
        let Some(new_balance) = account.info.balance.checked_add(balance) else {
            self.checkpoint_revert(checkpoint);
            return Err(InstructionResult::OverflowPayment);
        };
        account.info.balance = new_balance;

        // EIP-161: State trie clearing (invariant-preserving alternative)
        if SPEC::enabled(SPURIOUS_DRAGON) {
            account.info.nonce = 1;
            last_journal.push(JournalEntry::NonceChange { address });
        }

        // Sub balance from caller
        let caller_account = self.state.get_mut(&caller).unwrap();
        // Balance is already checked in `create_inner`, so it is safe to just substract.
        caller_account.info.balance -= balance;

        // add journal entry of transfered balance
        last_journal.push(JournalEntry::BalanceTransfer {
            from: caller,
            to: address,
            balance,
        });

        Ok(checkpoint)
    }

    #[inline(always)]
    pub fn check_account_collision(
        address: B160,
        account: &mut Account,
        num_of_precompiles: usize,
    ) -> bool {
        // Check collision. Bytecode needs to be empty.
        if account.info.code_hash != KECCAK_EMPTY {
            return true;
        }
        // Check collision. Nonce is not zero
        if account.info.nonce != 0 {
            return true;
        }

        // Check collision. New account address is precompile.
        if is_precompile(address, num_of_precompiles) {
            return true;
        }

        false
    }

    fn journal_revert(
        state: &mut State,
        journal_entries: Vec<JournalEntry>,
        is_spurious_dragon_enabled: bool,
    ) {
        const PRECOMPILE3: B160 =
            B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
        for entry in journal_entries.into_iter().rev() {
            match entry {
                JournalEntry::AccountLoaded { address } => {
                    if is_spurious_dragon_enabled && address == PRECOMPILE3 {
                        continue;
                    }
                    state.remove(&address);
                }
                JournalEntry::AccountTouched { address } => {
                    if is_spurious_dragon_enabled && address == PRECOMPILE3 {
                        continue;
                    }
                    // remove touched status
                    state.get_mut(&address).unwrap().unmark_touch();
                }
                JournalEntry::AccountDestroyed {
                    address,
                    target,
                    was_destroyed,
                    had_balance,
                } => {
                    let account = state.get_mut(&address).unwrap();
                    // set previous ste of selfdestructed flag. as there could be multiple
                    // selfdestructs in one transaction.
                    if was_destroyed {
                        // flag is still selfdestructed
                        account.mark_selfdestruct();
                    } else {
                        // flag that is not selfdestructed
                        account.unmark_selfdestruct();
                    }
                    account.info.balance += had_balance;

                    if address != target {
                        let target = state.get_mut(&target).unwrap();
                        target.info.balance -= had_balance;
                    }
                }
                JournalEntry::BalanceTransfer { from, to, balance } => {
                    // we dont need to check overflow and underflow when adding sub subtracting the balance.
                    let from = state.get_mut(&from).unwrap();
                    from.info.balance += balance;
                    let to = state.get_mut(&to).unwrap();
                    to.info.balance -= balance;
                }
                JournalEntry::NonceChange { address } => {
                    state.get_mut(&address).unwrap().info.nonce -= 1;
                }
                JournalEntry::StorageChange {
                    address,
                    key,
                    had_value,
                } => {
                    let storage = &mut state.get_mut(&address).unwrap().storage;
                    if let Some(had_value) = had_value {
                        storage.get_mut(&key).unwrap().present_value = had_value;
                    } else {
                        storage.remove(&key);
                    }
                }
                JournalEntry::CodeChange { address, had_code } => {
                    let acc = state.get_mut(&address).unwrap();
                    acc.info.code_hash = had_code.hash();
                    acc.info.code = Some(had_code);
                }
            }
        }
    }

    pub fn checkpoint(&mut self) -> JournalCheckpoint {
        let checkpoint = JournalCheckpoint {
            log_i: self.logs.len(),
            journal_i: self.journal.len(),
        };
        self.depth += 1;
        self.journal.push(Default::default());
        checkpoint
    }

    pub fn checkpoint_commit(&mut self) {
        self.depth -= 1;
    }

    pub fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        let is_spurious_dragon_enabled = !self.is_before_spurious_dragon;
        let state = &mut self.state;
        self.depth -= 1;
        // iterate over last N journals sets and revert our global state
        let leng = self.journal.len();
        self.journal
            .iter_mut()
            .rev()
            .take(leng - checkpoint.journal_i)
            .for_each(|cs| Self::journal_revert(state, mem::take(cs), is_spurious_dragon_enabled));

        self.logs.truncate(checkpoint.log_i);
        self.journal.truncate(checkpoint.journal_i);
    }

    /// transfer balance from address to target. Check if target exist/is_cold
    pub fn selfdestruct<DB: Database>(
        &mut self,
        address: B160,
        target: B160,
        db: &mut DB,
    ) -> Result<SelfDestructResult, DB::Error> {
        let (is_cold, target_exists) = self.load_account_exist(target, db)?;
        // transfer all the balance
        let acc = self.state.get_mut(&address).unwrap();
        let balance = mem::take(&mut acc.info.balance);
        let previously_destroyed = acc.is_selfdestructed();
        acc.mark_selfdestruct();

        // NOTE: In case that target and destroyed addresses are same, balance will be lost.
        // ref: https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/vm/instructions.go#L832-L833
        // https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/state/statedb.go#L449
        if address != target {
            let target_account = self.state.get_mut(&target).unwrap();
            // touch target account
            Self::touch_account(self.journal.last_mut().unwrap(), &target, target_account);
            target_account.info.balance += balance;
        }

        self.journal
            .last_mut()
            .unwrap()
            .push(JournalEntry::AccountDestroyed {
                address,
                target,
                was_destroyed: previously_destroyed,
                had_balance: balance,
            });

        Ok(SelfDestructResult {
            had_value: balance != U256::ZERO,
            is_cold,
            target_exists,
            previously_destroyed,
        })
    }

    pub fn initial_account_and_code_load<DB: Database>(
        &mut self,
        address: B160,
        db: &mut DB,
    ) -> Result<&mut Account, DB::Error> {
        let account = self.initial_account_load(address, &[], db)?;
        if account.info.code.is_none() {
            if account.info.code_hash == KECCAK_EMPTY {
                account.info.code = Some(Bytecode::new());
            } else {
                // load code if requested
                account.info.code = Some(db.code_by_hash(account.info.code_hash)?);
            }
        }

        Ok(account)
    }

    /// Initial load of account. This load will not be tracked inside journal
    pub fn initial_account_load<DB: Database>(
        &mut self,
        address: B160,
        slots: &[U256],
        db: &mut DB,
    ) -> Result<&mut Account, DB::Error> {
        match self.state.entry(address) {
            Entry::Occupied(entry) => {
                let account = entry.into_mut();

                Ok(account)
            }
            Entry::Vacant(vac) => {
                let mut account = db
                    .basic(address)?
                    .map(|i| i.into())
                    .unwrap_or(Account::new_not_existing());

                for slot in slots {
                    let storage = db.storage(address, *slot)?;
                    account.storage.insert(*slot, StorageSlot::new(storage));
                }

                Ok(vac.insert(account))
            }
        }
    }

    /// load account into memory. return if it is cold or hot accessed
    pub fn load_account<DB: Database>(
        &mut self,
        address: B160,
        db: &mut DB,
    ) -> Result<(&mut Account, bool), DB::Error> {
        Ok(match self.state.entry(address) {
            Entry::Occupied(entry) => (entry.into_mut(), false),
            Entry::Vacant(vac) => {
                let account = if let Some(account) = db.basic(address)? {
                    account.into()
                } else {
                    Account::new_not_existing()
                };

                // journal loading of account. AccessList touch.
                self.journal
                    .last_mut()
                    .unwrap()
                    .push(JournalEntry::AccountLoaded { address });

                // precompiles are hot loaded so we need to take that into account
                let is_cold = !is_precompile(address, self.num_of_precompiles);

                (vac.insert(account), is_cold)
            }
        })
    }

    // first is is_cold second bool is exists.
    pub fn load_account_exist<DB: Database>(
        &mut self,
        address: B160,
        db: &mut DB,
    ) -> Result<(bool, bool), DB::Error> {
        let is_before_spurious_dragon = self.is_before_spurious_dragon;
        let (acc, is_cold) = self.load_account(address, db)?;

        let exist = if is_before_spurious_dragon {
            let is_existing = !acc.is_loaded_as_not_existing();
            let is_touched = acc.is_touched();
            is_existing || is_touched
        } else {
            !acc.is_empty()
        };
        Ok((is_cold, exist))
    }

    pub fn load_code<DB: Database>(
        &mut self,
        address: B160,
        db: &mut DB,
    ) -> Result<(&mut Account, bool), DB::Error> {
        let (acc, is_cold) = self.load_account(address, db)?;
        if acc.info.code.is_none() {
            if acc.info.code_hash == KECCAK_EMPTY {
                let empty = Bytecode::new();
                acc.info.code = Some(empty);
            } else {
                let code = db.code_by_hash(acc.info.code_hash)?;
                acc.info.code = Some(code);
            }
        }
        Ok((acc, is_cold))
    }

    // account is already present and loaded.
    pub fn sload<DB: Database>(
        &mut self,
        address: B160,
        key: U256,
        db: &mut DB,
    ) -> Result<(U256, bool), DB::Error> {
        let account = self.state.get_mut(&address).unwrap(); // asume acc is hot
        let is_newly_created = account.is_newly_created();
        let load = match account.storage.entry(key) {
            Entry::Occupied(occ) => (occ.get().present_value, false),
            Entry::Vacant(vac) => {
                // if storage was cleared, we dont need to ping db.
                let value = if is_newly_created {
                    U256::ZERO
                } else {
                    db.storage(address, key)?
                };
                // add it to journal as cold loaded.
                self.journal
                    .last_mut()
                    .unwrap()
                    .push(JournalEntry::StorageChange {
                        address,
                        key,
                        had_value: None,
                    });

                vac.insert(StorageSlot::new(value));

                (value, true)
            }
        };
        Ok(load)
    }

    /// account should already be present in our state.
    /// returns (original,present,new) slot
    pub fn sstore<DB: Database>(
        &mut self,
        address: B160,
        key: U256,
        new: U256,
        db: &mut DB,
    ) -> Result<(U256, U256, U256, bool), DB::Error> {
        // assume that acc exists and load the slot.
        let (present, is_cold) = self.sload(address, key, db)?;
        let acc = self.state.get_mut(&address).unwrap();

        // if there is no original value in dirty return present value, that is our original.
        let slot = acc.storage.get_mut(&key).unwrap();

        // new value is same as present, we dont need to do anything
        if present == new {
            return Ok((slot.original_value, present, new, is_cold));
        }

        self.journal
            .last_mut()
            .unwrap()
            .push(JournalEntry::StorageChange {
                address,
                key,
                had_value: Some(present),
            });
        // insert value into present state.
        slot.present_value = new;
        Ok((slot.original_value, present, new, is_cold))
    }

    /// push log into subroutine
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }
}

/// Check if address is precompile by having assumption
/// that precompiles are in range of 1 to N.
#[inline(always)]
pub fn is_precompile(address: B160, num_of_precompiles: usize) -> bool {
    if !address[..18].iter().all(|i| *i == 0) {
        return false;
    }
    let num = u16::from_be_bytes([address[18], address[19]]);
    num.wrapping_sub(1) < num_of_precompiles as u16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_precompile() {
        assert!(
            !is_precompile(
                B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                3
            ),
            "Zero is not precompile"
        );

        assert!(
            !is_precompile(
                B160([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9]),
                3
            ),
            "0x100..0 is not precompile"
        );

        assert!(
            !is_precompile(
                B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]),
                3
            ),
            "0x000..4 is not precompile"
        );

        assert!(
            is_precompile(
                B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
                3
            ),
            "0x00..01 is precompile"
        );

        assert!(
            is_precompile(
                B160([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]),
                3
            ),
            "0x000..3 is precompile"
        );
    }
}
