use super::TransitionAccount;
use revm_interpreter::primitives::{hash_map::Entry, HashMap, StorageSlot, B160, U256};

/// TODO Rename this to become StorageWithOriginalValues or something like that.
/// This is used inside EVM and for block state. It is needed for block state to
/// be able to create changeset agains bundle state.
///
/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before t
pub type Storage = HashMap<U256, StorageSlot>;

#[derive(Clone, Debug)]
pub struct TransitionState {
    /// Block state account with account state
    pub accounts: HashMap<B160, TransitionAccount>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork).
    pub has_state_clear: bool,
}

impl Default for TransitionState {
    fn default() -> Self {
        // be default make state clear EIP enabled
        TransitionState {
            accounts: HashMap::new(),
            has_state_clear: true,
        }
    }
}

impl TransitionState {
    /// For newest fork this should be always `true`.
    ///
    /// For blocks before SpuriousDragon set this to `false`.
    pub fn new(has_state_clear: bool) -> Self {
        Self {
            accounts: HashMap::new(),
            has_state_clear,
        }
    }

    /// Used for tests only. When transitioned it is not recoverable
    pub fn set_state_clear(&mut self) {
        if self.has_state_clear == true {
            return;
        }

        self.has_state_clear = true;
    }

    pub fn add_transitions(&mut self, transitions: Vec<(B160, TransitionAccount)>) {
        for (address, account) in transitions {
            match self.accounts.entry(address) {
                Entry::Occupied(entry) => {
                    let entry = entry.into_mut();
                    entry.update(account);
                }
                Entry::Vacant(entry) => {
                    entry.insert(account);
                }
            }
        }
    }

    // pub fn insert_not_existing(&mut self, address: B160) {
    //     self.accounts
    //         .insert(address, BundleAccount::new_loaded_not_existing());
    // }

    // pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
    //     let account = if !info.is_empty() {
    //         BundleAccount::new_loaded(info, HashMap::default())
    //     } else {
    //         BundleAccount::new_loaded_empty_eip161(HashMap::default())
    //     };
    //     self.accounts.insert(address, account);
    // }

    // pub fn insert_account_with_storage(
    //     &mut self,
    //     address: B160,
    //     info: AccountInfo,
    //     storage: Storage,
    // ) {
    //     let account = if !info.is_empty() {
    //         BundleAccount::new_loaded(info, storage)
    //     } else {
    //         BundleAccount::new_loaded_empty_eip161(storage)
    //     };
    //     self.accounts.insert(address, account);
    // }
}
