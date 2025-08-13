//! Module containing the [`JournalInner`] that is part of [`crate::Journal`].
use crate::{entry::SelfdestructionRevertStatus, warm_addresses::WarmAddresses};

use super::JournalEntryTr;
use bytecode::Bytecode;
use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::{AccountLoad, JournalCheckpoint, TransferError},
};
use core::mem;
use database_interface::Database;
use primitives::{
    hardfork::SpecId::{self, *},
    hash_map::Entry,
    AccountId, Address, AddressAndId, AddressOrId, Log, StorageKey, StorageValue, B256,
    KECCAK_EMPTY, U256,
};
use state::{Account, EvmState, EvmStorageSlot, TransientStorage};
use std::vec::Vec;
/// Inner journal state that contains journal and state changes.
///
/// Spec Id is a essential information for the Journal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournalInner<ENTRY> {
    /// The current state new
    pub state: EvmState,
    /// Transient storage that is discarded after every transaction.
    ///
    /// See [EIP-1153](https://eips.ethereum.org/EIPS/eip-1153).
    pub transient_storage: TransientStorage,
    /// Emitted logs
    pub logs: Vec<Log>,
    /// The current call stack depth
    pub depth: usize,
    /// The journal of state changes, one for each transaction
    pub journal: Vec<ENTRY>,
    /// Global transaction id that represent number of transactions executed (Including reverted ones).
    /// It can be different from number of `journal_history` as some transaction could be
    /// reverted or had a error on execution.
    ///
    /// This ID is used in `Self::state` to determine if account/storage is touched/warm/cold.
    pub transaction_id: usize,
    /// The spec ID for the EVM. Spec is required for some journal entries and needs to be set for
    /// JournalInner to be functional.
    ///
    /// If spec is set it assumed that precompile addresses are set as well for this particular spec.
    ///
    /// This spec is used for two things:
    ///
    /// - [EIP-161]: Prior to this EIP, Ethereum had separate definitions for empty and non-existing accounts.
    /// - [EIP-6780]: `SELFDESTRUCT` only in same transaction
    ///
    /// [EIP-161]: https://eips.ethereum.org/EIPS/eip-161
    /// [EIP-6780]: https://eips.ethereum.org/EIPS/eip-6780
    pub spec: SpecId,
    /// Warm addresses containing current precompiles and coinbase, caller and tx target addresses.
    pub warm_addresses: WarmAddresses,
}

impl<ENTRY: JournalEntryTr> Default for JournalInner<ENTRY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<ENTRY: JournalEntryTr> JournalInner<ENTRY> {
    /// Creates new [`JournalInner`].
    ///
    /// `warm_preloaded_addresses` is used to determine if address is considered warm loaded.
    /// In ordinary case this is precompile or beneficiary.
    pub fn new() -> JournalInner<ENTRY> {
        Self {
            state: EvmState::new(),
            transient_storage: TransientStorage::default(),
            logs: Vec::new(),
            journal: Vec::default(),
            transaction_id: 0,
            depth: 0,
            spec: SpecId::default(),
            warm_addresses: WarmAddresses::new(),
        }
    }

    /// Returns the logs
    #[inline]
    pub fn take_logs(&mut self) -> Vec<Log> {
        mem::take(&mut self.logs)
    }

    /// Prepare for next transaction, by committing the current journal to history, incrementing the transaction id
    /// and returning the logs.
    ///
    /// This function is used to prepare for next transaction. It will save the current journal
    /// and clear the journal for the next transaction.
    ///
    /// `commit_tx` is used even for discarding transactions so transaction_id will be incremented.
    pub fn commit_tx(&mut self) {
        // Clears all field from JournalInner. Doing it this way to avoid
        // missing any field.
        let Self {
            state,
            transient_storage,
            logs,
            depth,
            journal,
            transaction_id,
            spec,
            warm_addresses,
        } = self;
        // Spec precompiles and state are not changed. It is always set again execution.
        let _ = spec;
        let _ = state;
        transient_storage.clear();
        *depth = 0;

        // Do nothing with journal history so we can skip cloning present journal.
        journal.clear();

        // Clear coinbase address warming for next tx
        warm_addresses.clear_addresses();

        // increment transaction id.
        *transaction_id += 1;
        logs.clear();
    }

    /// Discard the current transaction, by reverting the journal entries and incrementing the transaction id.
    pub fn discard_tx(&mut self) {
        // if there is no journal entries, there has not been any changes.
        let Self {
            state,
            transient_storage,
            logs,
            depth,
            journal,
            transaction_id,
            spec,
            warm_addresses,
        } = self;
        let is_spurious_dragon_enabled = spec.is_enabled_in(SPURIOUS_DRAGON);
        // iterate over all journals entries and revert our global state
        journal.drain(..).rev().for_each(|entry| {
            entry.revert(state, None, is_spurious_dragon_enabled);
        });
        transient_storage.clear();
        *depth = 0;
        logs.clear();
        *transaction_id += 1;

        // Clear coinbase address warming for next tx
        warm_addresses.clear_addresses();
    }

    /// Take the [`EvmState`] and clears the journal by resetting it to initial state.
    ///
    /// Note: Precompile addresses and spec are preserved and initial state of
    /// warm_preloaded_addresses will contain precompiles addresses.
    #[inline]
    pub fn finalize(&mut self) -> EvmState {
        // Clears all field from JournalInner. Doing it this way to avoid
        // missing any field.
        let Self {
            state,
            transient_storage,
            logs,
            depth,
            journal,
            transaction_id,
            spec,
            warm_addresses,
        } = self;
        // Spec is not changed. And it is always set again in execution.
        let _ = spec;
        // Clear coinbase address warming for next tx
        warm_addresses.clear_addresses();

        let state = mem::take(state);

        logs.clear();
        transient_storage.clear();

        // clear journal and journal history.
        journal.clear();
        *depth = 0;
        // reset transaction id.
        *transaction_id = 0;

        state
    }

    /// Return reference to state.
    #[inline]
    pub fn state(&mut self) -> &mut EvmState {
        &mut self.state
    }

    /// Sets SpecId.
    #[inline]
    pub fn set_spec_id(&mut self, spec: SpecId) {
        self.spec = spec;
    }

    /// Mark account as touched as only touched accounts will be added to state.
    /// This is especially important for state clear where touched empty accounts needs to
    /// be removed from state.
    #[inline]
    pub fn touch(&mut self, address_or_id: AddressOrId) {
        if let Some((account, id)) = self.state.get_mut(address_or_id) {
            Self::touch_account(&mut self.journal, id.id(), account);
        }
    }

    /// Mark account as touched.
    #[inline]
    fn touch_account(journal: &mut Vec<ENTRY>, id: AccountId, account: &mut Account) {
        if !account.is_touched() {
            journal.push(ENTRY::account_touched(id));
            account.mark_touch();
        }
    }

    /// Returns the _loaded_ [Account] for the given address.
    ///
    /// This assumes that the account has already been loaded.
    ///
    /// # Panics
    ///
    /// Panics if the account has not been loaded and is missing from the state set.
    #[inline]
    pub fn account(&self, address_or_id: AddressOrId) -> (&Account, AddressAndId) {
        self.state
            .get(address_or_id)
            .expect("Account expected to be loaded") // Always assume that acc is already loaded.
    }

    /// Set code and its hash to the account.
    ///
    /// Note: Assume account is warm and that hash is calculated from code.
    #[inline]
    pub fn set_code_with_hash(&mut self, address_or_id: AddressOrId, code: Bytecode, hash: B256) {
        let (account, address) = self.state.get_mut(address_or_id).unwrap();
        Self::touch_account(&mut self.journal, address.id(), account);

        self.journal.push(ENTRY::code_changed(address.id()));

        account.info.code_hash = hash;
        account.info.code = Some(code);
    }

    /// Use it only if you know that acc is warm.
    ///
    /// Assume account is warm.
    ///
    /// In case of EIP-7702 code with zero address, the bytecode will be erased.
    #[inline]
    pub fn set_code(&mut self, address_or_id: AddressOrId, code: Bytecode) {
        if let Bytecode::Eip7702(eip7702_bytecode) = &code {
            if eip7702_bytecode.address().is_zero() {
                self.set_code_with_hash(address_or_id, Bytecode::default(), KECCAK_EMPTY);
                return;
            }
        }

        let hash = code.hash_slow();
        self.set_code_with_hash(address_or_id, code, hash)
    }

    /// Add journal entry for caller accounting.
    #[inline]
    pub fn caller_accounting_journal_entry(
        &mut self,
        address_or_id: AddressOrId,
        old_balance: U256,
        bump_nonce: bool,
    ) {
        let id = self.state.get_id(&address_or_id).unwrap();
        // account balance changed.
        self.journal.push(ENTRY::balance_changed(id, old_balance));
        // account is touched.
        self.journal.push(ENTRY::account_touched(id));

        if bump_nonce {
            // nonce changed.
            self.journal.push(ENTRY::nonce_changed(id));
        }
    }

    /// Increments the balance of the account.
    ///
    /// Mark account as touched.
    #[inline]
    pub fn balance_incr<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
        balance: U256,
    ) -> Result<AddressAndId, DB::Error> {
        let (account, address) = self.load_account(db, address_or_id)?.data;
        let old_balance = account.info.balance;
        account.info.balance = account.info.balance.saturating_add(balance);

        // march account as touched.
        if !account.is_touched() {
            account.mark_touch();
            self.journal.push(ENTRY::account_touched(address.id()));
        }

        // add journal entry for balance increment.
        self.journal
            .push(ENTRY::balance_changed(address.id(), old_balance));
        Ok(address)
    }

    /// Increments the balance of the account.
    ///
    /// Mark account as touched.
    #[inline]
    pub fn balance_incr_by_id(&mut self, account_id: AccountId, balance: U256) {
        let account = self.state.get_by_id_mut(account_id).0;
        let old_balance = account.info.balance;
        account.info.balance = account.info.balance.saturating_add(balance);

        // march account as touched.
        if !account.is_touched() {
            account.mark_touch();
            self.journal.push(ENTRY::account_touched(account_id));
        }

        // add journal entry for balance increment.
        self.journal
            .push(ENTRY::balance_changed(account_id, old_balance))
    }

    /// Increments the nonce of the account.
    #[inline]
    pub fn nonce_bump_journal_entry(&mut self, address_or_id: AddressOrId) {
        // TODO check if it is okay to unwrap here
        let id = self.state.get_id(&address_or_id).unwrap();
        self.journal.push(ENTRY::nonce_changed(id));
    }

    /// Transfers balance from two accounts. Returns error if sender balance is not enough.
    #[inline]
    pub fn transfer(
        &mut self,
        from: AccountId,
        to: AccountId,
        balance: U256,
    ) -> Option<TransferError> {
        if balance.is_zero() {
            let to_account = self.state.get_by_id_mut(to).0;
            Self::touch_account(&mut self.journal, to, to_account);
            return None;
        }

        // sub balance from
        let (from_account, _) = self.state.get_by_id_mut(from);
        Self::touch_account(&mut self.journal, from, from_account);
        let from_balance = &mut from_account.info.balance;

        let Some(from_balance_decr) = from_balance.checked_sub(balance) else {
            return Some(TransferError::OutOfFunds);
        };
        *from_balance = from_balance_decr;

        // add balance to
        let (to_account, _) = self.state.get_by_id_mut(to);
        Self::touch_account(&mut self.journal, to, to_account);
        let to_balance = &mut to_account.info.balance;
        let Some(to_balance_incr) = to_balance.checked_add(balance) else {
            return Some(TransferError::OverflowPayment);
        };
        *to_balance = to_balance_incr;
        // Overflow of U256 balance is not possible to happen on mainnet. We don't bother to return funds from from_acc.

        self.journal
            .push(ENTRY::balance_transfer(from, to, balance));

        None
    }

    /// Creates account or returns false if collision is detected.
    ///
    /// There are few steps done:
    /// 1. Make created account warm loaded (AccessList) and this should
    ///    be done before subroutine checkpoint is created.
    /// 2. Check if there is collision of newly created account with existing one.
    /// 3. Mark created account as created.
    /// 4. Add fund to created account
    /// 5. Increment nonce of created account if SpuriousDragon is active
    /// 6. Decrease balance of caller account.
    ///
    /// # Panics
    ///
    /// Panics if the caller is not loaded inside the EVM state.
    /// This should have been done inside `create_inner`.
    #[inline]
    pub fn create_account_checkpoint(
        &mut self,
        caller_or_id: AddressOrId,
        target_or_id: AddressOrId,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        // Enter subroutine
        let checkpoint = self.checkpoint();

        // Fetch balance of caller.
        let (caller_acc, caller) = self.state.get(caller_or_id).unwrap();
        let caller_balance = caller_acc.info.balance;
        // Check if caller has enough balance to send to the created contract.
        if caller_balance < balance {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::OutOfFunds);
        }

        // Newly created account is present, as we just loaded it.
        let (target_acc, target) = self.state.get_mut(target_or_id).unwrap();
        let last_journal = &mut self.journal;

        // New account can be created if:
        // Bytecode is not empty.
        // Nonce is not zero
        // Account is not precompile.
        if target_acc.info.code_hash != KECCAK_EMPTY || target_acc.info.nonce != 0 {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::CreateCollision);
        }

        // set account status to create.
        let is_created_globally = target_acc.mark_created_locally();

        // this entry will revert set nonce.
        last_journal.push(ENTRY::account_created(target.id(), is_created_globally));
        target_acc.info.code = None;
        // EIP-161: State trie clearing (invariant-preserving alternative)
        if spec_id.is_enabled_in(SPURIOUS_DRAGON) {
            // nonce is going to be reset to zero in AccountCreated journal entry.
            target_acc.info.nonce = 1;
        }

        // touch account. This is important as for pre SpuriousDragon account could be
        // saved even empty.
        Self::touch_account(last_journal, target.id(), target_acc);

        // Add balance to created account, as we already have target here.
        let Some(new_balance) = target_acc.info.balance.checked_add(balance) else {
            self.checkpoint_revert(checkpoint);
            return Err(TransferError::OverflowPayment);
        };
        target_acc.info.balance = new_balance;

        // safe to decrement for the caller as balance check is already done.
        self.state
            .get_mut(AddressOrId::Id(caller.id()))
            .unwrap()
            .0
            .info
            .balance -= balance;

        // add journal entry of transferred balance
        last_journal.push(ENTRY::balance_transfer(caller.id(), target.id(), balance));

        Ok(checkpoint)
    }

    /// Makes a checkpoint that in case of Revert can bring back state to this point.
    #[inline]
    pub fn checkpoint(&mut self) -> JournalCheckpoint {
        let checkpoint = JournalCheckpoint {
            log_i: self.logs.len(),
            journal_i: self.journal.len(),
        };
        self.depth += 1;
        checkpoint
    }

    /// Commits the checkpoint.
    #[inline]
    pub fn checkpoint_commit(&mut self) {
        self.depth -= 1;
    }

    /// Reverts all changes to state until given checkpoint.
    #[inline]
    pub fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        let is_spurious_dragon_enabled = self.spec.is_enabled_in(SPURIOUS_DRAGON);
        let state = &mut self.state;
        let transient_storage = &mut self.transient_storage;
        self.depth -= 1;
        self.logs.truncate(checkpoint.log_i);

        // iterate over last N journals sets and revert our global state
        self.journal
            .drain(checkpoint.journal_i..)
            .rev()
            .for_each(|entry| {
                entry.revert(state, Some(transient_storage), is_spurious_dragon_enabled);
            });
    }

    /// Performs selfdestruct action.
    /// Transfers balance from address to target. Check if target exist/is_cold
    ///
    /// Note: Balance will be lost if address and target are the same BUT when
    /// current spec enables Cancun, this happens only when the account associated to address
    /// is created in the same tx
    ///
    /// # References:
    ///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/vm/instructions.go#L832-L833>
    ///  * <https://github.com/ethereum/go-ethereum/blob/141cd425310b503c5678e674a8c3872cf46b7086/core/state/statedb.go#L449>
    ///  * <https://eips.ethereum.org/EIPS/eip-6780>
    #[inline]
    pub fn selfdestruct<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, DB::Error> {
        let (acc, address) = self.state.get_mut(address_or_id).unwrap();
        let balance = acc.info.balance;

        let spec = self.spec;
        let target_account_load = self.load_account(db, AddressOrId::Address(target))?;
        let is_cold = target_account_load.is_cold;
        let is_empty = target_account_load.0.state_clear_aware_is_empty(spec);
        let (target_acc, target) = target_account_load.data;

        if target != address_or_id {
            // Both accounts are loaded before this point, `address` as we execute its contract.
            // and `target` at the beginning of the function.
            target_acc.info.balance += balance;
            if !target_acc.is_touched() {
                target_acc.mark_touch();
                self.journal.push(ENTRY::account_touched(target.id()));
            }
        }

        let (acc, _) = self.state.get_mut(address_or_id).unwrap();

        let destroyed_status = if !acc.is_selfdestructed() {
            SelfdestructionRevertStatus::GloballySelfdestroyed
        } else if !acc.is_selfdestructed_locally() {
            SelfdestructionRevertStatus::LocallySelfdestroyed
        } else {
            SelfdestructionRevertStatus::RepeatedSelfdestruction
        };

        let is_cancun_enabled = spec.is_enabled_in(CANCUN);

        // EIP-6780 (Cancun hard-fork): selfdestruct only if contract is created in the same tx
        let journal_entry = if acc.is_created_locally() || !is_cancun_enabled {
            acc.mark_selfdestructed_locally();
            acc.info.balance = U256::ZERO;
            Some(ENTRY::account_destroyed(
                address.id(),
                target.id(),
                destroyed_status,
                balance,
            ))
        } else if address.address() != target.address() {
            acc.info.balance = U256::ZERO;
            Some(ENTRY::balance_transfer(address.id(), target.id(), balance))
        } else {
            // State is not changed:
            // * if we are after Cancun upgrade and
            // * Selfdestruct account that is created in the same transaction and
            // * Specify the target is same as selfdestructed account. The balance stays unchanged.
            None
        };

        if let Some(entry) = journal_entry {
            self.journal.push(entry);
        };

        Ok(StateLoad {
            data: SelfDestructResult {
                had_value: !balance.is_zero(),
                target_exists: !is_empty,
                previously_destroyed: destroyed_status
                    == SelfdestructionRevertStatus::RepeatedSelfdestruction,
            },
            is_cold,
        })
    }

    /// Loads account into memory. return if it is cold or warm accessed
    #[inline]
    pub fn load_account<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, DB::Error> {
        self.load_account_optional(db, address_or_id, false, [])
    }

    /// Loads account into memory. If account is EIP-7702 type it will additionally
    /// load delegated account.
    ///
    /// It will mark both this and delegated account as warm loaded.
    ///
    /// Returns information about the account (If it is empty or cold loaded) and if present the information
    /// about the delegated account (If it is cold loaded).
    #[inline]
    pub fn load_account_delegated<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<AccountLoad>, DB::Error> {
        let spec = self.spec;
        let is_eip7702_enabled = spec.is_enabled_in(SpecId::PRAGUE);
        let account = self.load_account_optional(db, address_or_id, is_eip7702_enabled, [])?;
        let is_empty = account.0.state_clear_aware_is_empty(spec);

        let mut account_load = StateLoad::new(
            AccountLoad {
                address_and_id: account.1,
                delegated_account_address: None,
                is_empty,
            },
            account.is_cold,
        );

        // load delegate code if account is EIP-7702
        if let Some(Bytecode::Eip7702(code)) = &account.0.info.code {
            let address = code.address();
            let delegate_account = self.load_account(db, AddressOrId::Address(address))?;
            account_load.data.delegated_account_address =
                Some(StateLoad::new(delegate_account.1, delegate_account.is_cold));
        }

        Ok(account_load)
    }

    /// Loads account and its code. If account is already loaded it will load its code.
    ///
    /// It will mark account as warm loaded. If not existing Database will be queried for data.
    ///
    /// In case of EIP-7702 delegated account will not be loaded,
    /// [`Self::load_account_delegated`] should be used instead.
    #[inline]
    pub fn load_code<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, DB::Error> {
        self.load_account_optional(db, address_or_id, true, [])
    }

    /// Loads account. If account is already loaded it will be marked as warm.
    #[inline(never)]
    pub fn load_account_optional<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
        load_code: bool,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, DB::Error> {
        let (account, address_and_id) = match address_or_id {
            AddressOrId::Address(address) => {
                self.state
                    .get_mut_or_fetch(address, |address| -> Result<Account, DB::Error> {
                        db.basic(address).map(|account| {
                            if let Some(account) = account {
                                account.into()
                            } else {
                                Account::new_not_existing(self.transaction_id)
                            }
                        })
                    })?
            }
            AddressOrId::Id(id) => self.state.get_by_id_mut(id),
        };

        let mut is_cold = account.mark_warm_with_transaction_id(self.transaction_id);

        if is_cold {
            // if it is cold loaded and we have selfdestructed locally it means that
            // account was selfdestructed in previous transaction and we need to clear its information and storage.
            if account.is_selfdestructed_locally() {
                account.selfdestruct();
                account.unmark_selfdestructed_locally();
            }
            // unmark locally created
            account.unmark_created_locally();

            // Precompiles among some other account(coinbase included) are warm loaded so we need to take that into account
            is_cold = self.warm_addresses.is_cold(address_and_id.address());

            // journal loading of cold account.
            if is_cold {
                self.journal
                    .push(ENTRY::account_warmed(address_and_id.id()));
            }
        }

        if load_code {
            let info = &mut account.info;
            if info.code.is_none() {
                let code = if info.code_hash == KECCAK_EMPTY {
                    Bytecode::default()
                } else {
                    db.code_by_hash(info.code_hash)?
                };
                info.code = Some(code);
            }
        }

        for storage_key in storage_keys.into_iter() {
            sload_with_account(
                account,
                db,
                &mut self.journal,
                self.transaction_id,
                address_and_id,
                storage_key,
            )?;
        }

        Ok(StateLoad {
            data: (account, address_and_id),
            is_cold,
        })
    }

    /// Loads storage slot.
    ///
    /// # Panics
    ///
    /// Panics if the account is not present in the state.
    #[inline]
    pub fn sload<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, DB::Error> {
        // assume acc is warm
        let (account, id) = self.state.get_mut(address_or_id).unwrap();
        // only if account is created in this tx we can assume that storage is empty.
        sload_with_account(account, db, &mut self.journal, self.transaction_id, id, key)
    }

    /// Stores storage slot.
    ///
    /// And returns (original,present,new) slot value.
    ///
    /// **Note**: Account should already be present in our state.
    #[inline(never)]
    pub fn sstore<DB: Database>(
        &mut self,
        db: &mut DB,
        address_or_id: AddressOrId,
        key: StorageKey,
        new: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, DB::Error> {
        // assume that acc exists and load the slot.

        let (account, id) = self.state.get_mut(address_or_id).unwrap();
        // only if account is created in this tx we can assume that storage is empty.
        let present =
            sload_with_account(account, db, &mut self.journal, self.transaction_id, id, key)?;

        // if there is no original value in dirty return present value, that is our original.
        let slot = account.storage.get_mut(&key).unwrap();

        // new value is same as present, we don't need to do anything
        if present.data == new {
            return Ok(StateLoad::new(
                SStoreResult {
                    original_value: slot.original_value(),
                    present_value: present.data,
                    new_value: new,
                },
                present.is_cold,
            ));
        }

        self.journal
            .push(ENTRY::storage_changed(id.id(), key, present.data));
        // insert value into present state.
        slot.present_value = new;
        Ok(StateLoad::new(
            SStoreResult {
                original_value: slot.original_value(),
                present_value: present.data,
                new_value: new,
            },
            present.is_cold,
        ))
    }

    /// Read transient storage tied to the account.
    ///
    /// EIP-1153: Transient storage opcodes
    #[inline]
    pub fn tload(&mut self, address_or_id: AddressOrId, key: StorageKey) -> StorageValue {
        let id = self.state.get_id(&address_or_id).unwrap();
        self.transient_storage
            .get(&(id, key))
            .copied()
            .unwrap_or_default()
    }

    /// Store transient storage tied to the account.
    ///
    /// If values is different add entry to the journal
    /// so that old state can be reverted if that action is needed.
    ///
    /// EIP-1153: Transient storage opcodes
    #[inline]
    pub fn tstore(&mut self, address_or_id: AddressOrId, key: StorageKey, new: StorageValue) {
        let id = self.state.get_id(&address_or_id).unwrap();
        let had_value = if new.is_zero() {
            // if new values is zero, remove entry from transient storage.
            // if previous values was some insert it inside journal.
            // If it is none nothing should be inserted.
            self.transient_storage.remove(&(id, key))
        } else {
            // insert values
            let previous_value = self
                .transient_storage
                .insert((id, key), new)
                .unwrap_or_default();

            // check if previous value is same
            if previous_value != new {
                // if it is different, insert previous values inside journal.
                Some(previous_value)
            } else {
                None
            }
        };

        if let Some(had_value) = had_value {
            // insert in journal only if value was changed.
            self.journal
                .push(ENTRY::transient_storage_changed(id, key, had_value));
        }
    }

    /// Pushes log into subroutine.
    #[inline]
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }
}

/// Loads storage slot with account.
#[inline]
pub fn sload_with_account<DB: Database, ENTRY: JournalEntryTr>(
    account: &mut Account,
    db: &mut DB,
    journal: &mut Vec<ENTRY>,
    transaction_id: usize,
    address_or_id: AddressAndId,
    key: StorageKey,
) -> Result<StateLoad<StorageValue>, DB::Error> {
    let is_newly_created = account.is_created();
    let (value, is_cold) = match account.storage.entry(key) {
        Entry::Occupied(occ) => {
            let slot = occ.into_mut();
            let is_cold = slot.mark_warm_with_transaction_id(transaction_id);
            (slot.present_value, is_cold)
        }
        Entry::Vacant(vac) => {
            // if storage was cleared, we don't need to ping db.
            let value = if is_newly_created {
                StorageValue::ZERO
            } else {
                db.storage(*address_or_id.address(), key)?
            };

            vac.insert(EvmStorageSlot::new(value, transaction_id));

            (value, true)
        }
    };

    if is_cold {
        // add it to journal as cold loaded.
        journal.push(ENTRY::storage_warmed(address_or_id.id(), key));
    }

    Ok(StateLoad::new(value, is_cold))
}
