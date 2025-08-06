use crate::{Account, EvmStorageSlot};
use primitives::{
    AccountId, Address, AddressAndId, AddressOrId, HashMap, StorageKey, StorageValue,
};

/// EVM State is a mapping from addresses to accounts.
pub type EvmState = HashMap<Address, Account>;

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(AccountId, StorageKey), StorageValue>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<StorageKey, EvmStorageSlot>;

/// EVM State with internal account management.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmStateNew {
    /// Index of accounts.
    pub index: HashMap<Address, AccountId>,
    /// Accounts.
    /// TODO make pushing of new account smarter and introduce a Vec of Vec so we dont need to clone it.
    pub accounts: Vec<(Account, Address)>,
}

impl EvmStateNew {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self {
            index: HashMap::default(),
            accounts: Vec::new(),
        }
    }

    /// Get the account id for an address or id.
    pub fn get_id(&self, address_or_id: &AddressOrId) -> Option<AccountId> {
        match address_or_id {
            AddressOrId::Id(id) => Some(*id),
            AddressOrId::Address(address) => self.index.get(address).copied(),
        }
    }

    /// Get an immutable reference to an account by address.
    pub fn get(&self, address_or_id: &AddressOrId) -> Option<(&Account, AddressAndId)> {
        match address_or_id {
            AddressOrId::Id(id) => self
                .accounts
                .get(*id)
                .map(|(acc, address)| (acc, AddressAndId::new(*address, *id))),
            AddressOrId::Address(address) => self.index.get(address).and_then(|id| {
                self.accounts
                    .get(*id)
                    .map(|(acc, address)| (acc, AddressAndId::new(*address, *id)))
            }),
        }
    }

    /// Get a mutable reference to an account by address.
    pub fn get_mut(&mut self, address_or_id: &AddressOrId) -> Option<(&mut Account, AddressAndId)> {
        match address_or_id {
            AddressOrId::Id(id) => self
                .accounts
                .get_mut(*id)
                .map(|(acc, address)| (acc, AddressAndId::new(*address, *id))),
            AddressOrId::Address(address) => self.index.get(address).and_then(|id| {
                self.accounts
                    .get_mut(*id)
                    .map(|(acc, address)| (acc, AddressAndId::new(*address, *id)))
            }),
        }
    }

    /// Insert a new account or update an existing one.
    pub fn insert(&mut self, address: Address, account: Account) -> AddressAndId {
        todo!()
        // match self.index.get(&address) {
        //     Some(&id) => {
        //         // Update existing account
        //         let old_account = std::mem::replace(&mut self.accounts[id], (account, address));
        //         Some(old_account)
        //     }
        //     None => {
        //         // Insert new account
        //         let id = self.accounts.len();
        //         self.accounts.push((account, address));
        //         self.index.insert(address, id);
        //         None
        //     }
        // }
    }

    /// Remove an account by address.
    pub fn remove(&mut self, address: &Address) -> Option<Account> {
        todo!()
        // self.index.remove(address).and_then(|id| {
        //     // Note: This doesn't actually remove from the Vec to avoid invalidating indices.
        //     // The account at this index becomes invalid and shouldn't be accessed directly.
        //     // A proper implementation might mark it as deleted or use a different data structure.
        //     self.accounts.get(id).cloned()
        // })
    }

    /// Check if an account exists.
    pub fn contains_key(&self, address: &Address) -> bool {
        self.index.contains_key(address)
    }

    /// Get the number of accounts.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if the state is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Clear all accounts.
    pub fn clear(&mut self) {
        self.index.clear();
        self.accounts.clear();
    }

    /// Iterate over all accounts.
    pub fn iter(&self) -> impl Iterator<Item = &(Account, Address)> + '_ {
        self.accounts.iter()
    }
    /// Iterate mutably over all accounts.
    /// Returns a vector of (Address, &mut Account) pairs.
    ///
    /// Note: This collects addresses into a Vec to avoid borrowing issues.
    pub fn iter_mut(&mut self) -> Vec<(Address, &mut Account)> {
        todo!()
        // let mut result = Vec::new();
        // let addresses: Vec<(Address, AccountId)> =
        //     self.index.iter().map(|(k, &v)| (*k, v)).collect();

        // // We need to use unsafe here to get multiple mutable references
        // // This is safe because we know each AccountId maps to a unique index
        // for (addr, id) in addresses {
        //     if let Some(account) = self.accounts.get_mut(id) {
        //         result.push((addr, account as *mut Account));
        //     }
        // }

        // // Convert raw pointers back to references
        // result
        //     .into_iter()
        //     .map(|(addr, ptr)| unsafe { (addr, &mut *ptr) })
        //     .collect()
    }

    // /// Get a mutable reference to an account, inserting a default if it doesn't exist.
    // pub fn get_or_insert_default(&mut self, address: Address) -> &mut Account {
    //     if !self.contains_key(&address) {
    //         self.insert(address, Account::default());
    //     }
    //     self.get_mut(&address).unwrap()
    // }

    // /// Get a mutable reference to an account, inserting with a function if it doesn't exist.
    // pub fn get_or_insert_with<F>(&mut self, address_or_id: &AddressOrId, f: F) -> &mut Account
    // where
    //     F: FnOnce() -> Account,
    // {
    //     if !self.contains_key(&address) {
    //         self.insert(address, f());
    //     }
    //     self.get_mut(&address).unwrap()
    // }

    // /// Take ownership of the state, returning the underlying HashMap.
    // pub fn take(&mut self) -> HashMap<Address, Account> {
    //     let mut map = HashMap::new();
    //     let index = std::mem::take(&mut self.index);
    //     let accounts = std::mem::take(&mut self.accounts);

    //     for (address, id) in index {
    //         if let Some(account) = accounts.get(id) {
    //             map.insert(address, account.clone());
    //         }
    //     }
    //     map
    // }
}

impl Default for EvmStateNew {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<Address, Account>> for EvmStateNew {
    fn from(map: HashMap<Address, Account>) -> Self {
        let mut state = EvmStateNew::new();
        for (address, account) in map {
            state.insert(address, account);
        }
        state
    }
}

// impl From<EvmStateNew> for HashMap<Address, Account> {
//     fn from(state: EvmStateNew) -> Self {
//         let mut map = HashMap::new();
//         for (address, id) in state.index {
//             if let Some(account) = state.accounts.get(id) {
//                 map.insert(address, account.clone());
//             }
//         }
//         map
//     }
// }
