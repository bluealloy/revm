use crate::{Account, EvmStorageSlot};
use primitives::{
    map::Entry, AccountId, Address, AddressAndId, AddressOrId, HashMap, StorageKey, StorageValue,
};

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(AccountId, StorageKey), StorageValue>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<StorageKey, EvmStorageSlot>;

/// EVM State with internal account management.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmState {
    /// Index of accounts.
    pub index: HashMap<Address, AccountId>,
    /// Accounts.
    /// TODO make pushing of new account smarter and introduce a Vec of Vec so we dont need to clone it.
    pub accounts: Vec<(Account, Address)>,
}


impl EvmState {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self {
            index: HashMap::default(),
            accounts: Vec::default(),
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
    #[inline]
    pub fn get_mut(&mut self, address_or_id: AddressOrId) -> Option<(&mut Account, AddressAndId)> {
        match address_or_id {
            AddressOrId::Id(id) => self
                .accounts
                .get_mut(id)
                .map(|(acc, address)| (acc, AddressAndId::new(*address, id))),
            AddressOrId::Address(address) => self.index.get(&address).and_then(|id| {
                self.accounts
                    .get_mut(*id)
                    .map(|(acc, address)| (acc, AddressAndId::new(*address, *id)))
            }),
        }
    }

    /// Get an immutable reference to an account by id.
    #[inline]
    pub fn get_by_id(&self, id: AccountId) -> Option<(&Account, AddressAndId)> {
        self.accounts
            .get(id)
            .map(|(acc, address)| (acc, AddressAndId::new(*address, id)))
    }

    /// Get a mutable reference to an account by id.
    #[inline]
    pub fn get_by_id_mut(&mut self, id: AccountId) -> Option<(&mut Account, AddressAndId)> {
        self.accounts
            .get_mut(id)
            .map(|(acc, address)| (acc, AddressAndId::new(*address, id)))
    }

    /// Insert a new account or update an existing one.
    #[inline]
    pub fn insert(&mut self, address: Address, account: Account) -> AddressAndId {
        match self.index.get(&address) {
            Some(&id) => {
                // Update existing account
                self.accounts[id] = (account, address);
                AddressAndId::new(address, id)
            }
            None => {
                let id = self.accounts.len();
                self.index.insert(address, id);
                self.accounts.push((account, address));
                AddressAndId::new(address, id)
            }
        }
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

    /// Get a mutable reference to an account by address or fetch it if it doesn't exist.
    #[inline]
    pub fn get_mut_or_fetch<F, ERROR>(
        &mut self,
        address: Address,
        fetch: F,
    ) -> Result<(&mut Account, AddressAndId), ERROR>
    where
        F: FnOnce(Address) -> Result<Account, ERROR>,
    {
        match self.index.entry(address) {
            Entry::Occupied(entry) => {
                let address_and_id = AddressAndId::new(address, *entry.get());
                let (account, _) = self.accounts.get_mut(*entry.get()).unwrap();
                Ok((account, address_and_id))
            }
            Entry::Vacant(entry) => {
                let account = fetch(address)?;
                let id = self.accounts.len();
                entry.insert(id);
                self.accounts.reserve(32);
                self.accounts.push((account, address));
                let address_and_id = AddressAndId::new(address, id);
                Ok((&mut self.accounts.last_mut().unwrap().0, address_and_id))
            }
        }
    }
}

impl Default for EvmState {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<Address, Account>> for EvmState {
    fn from(map: HashMap<Address, Account>) -> Self {
        let mut state = EvmState::new();
        for (address, account) in map {
            state.insert(address, account);
        }
        state
    }
}
