use crate::{Account, EvmStorageSlot};
use primitives::{
    map::Entry, AccountId, Address, AddressAndId, AddressOrId, HashMap, StorageKey, StorageValue,
};
use std::{vec, vec::Vec};

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(AccountId, StorageKey), StorageValue>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<StorageKey, EvmStorageSlot>;

/// First page size is 3 to account for Caller, Target, and Beneficiary.
const FIRST_PAGE_SIZE: usize = 3;

/// Page size is 100.
const PAGE_SIZE: usize = 100;

/// Pages of accounts.
pub type AccountPages = Vec<Vec<(Account, Address)>>;

/// EVM State with internal account management.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmState {
    /// Index of accounts.
    index: HashMap<Address, AccountId>,
    /// Accounts.
    /// TODO make pushing of new account smarter and introduce a Vec of Vec so we dont need to clone it.
    accounts: AccountPages,
}

impl EvmState {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self {
            index: HashMap::default(),
            accounts: vec![Vec::with_capacity(FIRST_PAGE_SIZE)],
        }
    }

    /// Return pages of accounts.
    #[inline]
    pub fn accounts(&self) -> &AccountPages {
        &self.accounts
    }

    /// Return mutable reference to accounts.
    #[inline]
    pub fn accounts_mut(&mut self) -> &mut AccountPages {
        &mut self.accounts
    }

    /// Take accounts.
    #[inline]
    pub fn take(&mut self) -> (AccountPages, HashMap<Address, AccountId>) {
        let accounts = std::mem::replace(
            &mut self.accounts,
            vec![Vec::with_capacity(FIRST_PAGE_SIZE)],
        );
        let index = std::mem::take(&mut self.index);
        (accounts, index)
    }

    /// Take accounts.
    #[inline]
    pub fn take_accounts(&mut self) -> AccountPages {
        self.take().0
    }

    /// Take index.
    #[inline]
    pub fn take_index(&mut self) -> HashMap<Address, AccountId> {
        self.take().1
    }

    /// Return index of accounts.
    #[inline]
    pub fn index(&self) -> &HashMap<Address, AccountId> {
        &self.index
    }

    /// Return mutable reference to index.
    #[inline]
    pub fn index_mut(&mut self) -> &mut HashMap<Address, AccountId> {
        &mut self.index
    }

    /// Get the account id for an address or id.
    pub fn get_id(&self, address_or_id: &AddressOrId) -> Option<AccountId> {
        match address_or_id {
            AddressOrId::Id(id) => Some(*id),
            AddressOrId::Address(address) => self.index.get(address).copied(),
        }
    }

    /// Get an immutable reference to an account by address.
    pub fn get(&self, address_or_id: AddressOrId) -> Option<(&Account, AddressAndId)> {
        match address_or_id {
            AddressOrId::Id(id) => Some(self.get_by_id(id)),
            AddressOrId::Address(address) => self
                .index
                .get(&address)
                .map(|id| get_by_id(&self.accounts, *id)),
        }
    }

    /// Get a mutable reference to an account by address.
    #[inline]
    pub fn get_mut(&mut self, address_or_id: AddressOrId) -> Option<(&mut Account, AddressAndId)> {
        match address_or_id {
            AddressOrId::Id(id) => Some(self.get_by_id_mut(id)),
            AddressOrId::Address(address) => self.index.get(&address).and_then(|id| {
                self.accounts
                    .get_mut(id.0 as usize)
                    .and_then(|accounts| accounts.get_mut(id.1 as usize))
                    .map(|(acc, address)| (acc, AddressAndId::new(*address, *id)))
            }),
        }
    }

    /// Get an immutable reference to an account by id.
    #[inline]
    pub fn get_by_id(&self, id: AccountId) -> (&Account, AddressAndId) {
        get_by_id(&self.accounts, id)
    }

    /// Get a mutable reference to an account by id.
    #[inline]
    pub fn get_by_id_mut(&mut self, id: AccountId) -> (&mut Account, AddressAndId) {
        get_by_id_mut(&mut self.accounts, id)
    }

    /// Insert a new account or update an existing one.
    #[inline]
    pub fn insert(&mut self, address: Address, account: Account) -> AddressAndId {
        match self.index.get(&address) {
            Some(&id) => {
                // Update existing account
                self.accounts[id.0 as usize][id.1 as usize] = (account, address);
                AddressAndId::new(address, id)
            }
            None => {
                let id = push_account(&mut self.accounts, account, address);
                self.index.insert(address, id);
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
        self.accounts.iter().flat_map(|accounts| accounts.iter())
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
            Entry::Occupied(entry) => Ok(get_by_id_mut(&mut self.accounts, *entry.get())),
            Entry::Vacant(entry) => {
                let account = fetch(address)?;
                let id = push_account(&mut self.accounts, account, address);
                entry.insert(id);
                let address_and_id = AddressAndId::new(address, id);
                Ok((
                    &mut self.accounts.last_mut().unwrap().last_mut().unwrap().0,
                    address_and_id,
                ))
            }
        }
    }
}

/// Push an account to the accounts vector, allocating a new page if last page is full.
///
/// Returns the account id that was assigned to the account.
#[inline]
fn push_account(accounts: &mut AccountPages, account: Account, address: Address) -> AccountId {
    let page_len = accounts.len() as u32;
    if let Some(last) = accounts.last_mut() {
        let last_len = last.len();
        if (page_len == 1 && last_len < FIRST_PAGE_SIZE) || (page_len > 1 && last_len < PAGE_SIZE) {
            let id = (page_len - 1, last_len as u32);
            last.push((account, address));
            return id;
        }
    }
    let mut vec = Vec::with_capacity(PAGE_SIZE);
    vec.push((account, address));
    accounts.push(vec);
    (page_len, 0)
}

/// Get a mutable reference to an account by id.
#[inline]
fn get_by_id_mut(
    accounts: &mut [Vec<(Account, Address)>],
    id: AccountId,
) -> (&mut Account, AddressAndId) {
    let account = unsafe {
        accounts
            .get_unchecked_mut(id.0 as usize)
            .get_unchecked_mut(id.1 as usize)
    };
    (&mut account.0, AddressAndId::new(account.1, id))
}

/// Get an immutable reference to an account by id.
#[inline]
fn get_by_id(accounts: &[Vec<(Account, Address)>], id: AccountId) -> (&Account, AddressAndId) {
    let account = unsafe {
        accounts
            .get_unchecked(id.0 as usize)
            .get_unchecked(id.1 as usize)
    };
    (&account.0, AddressAndId::new(account.1, id))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountInfo, AccountStatus};

    fn create_test_account(nonce: u64) -> Account {
        Account {
            info: AccountInfo {
                balance: primitives::U256::from(1000),
                nonce,
                code_hash: primitives::KECCAK_EMPTY,
                code: None,
            },
            storage: HashMap::default(),
            status: AccountStatus::empty(),
            transaction_id: 0,
        }
    }

    #[test]
    fn test_get_mut_or_fetch_existing_account() {
        let mut state = EvmState::new();
        let address = Address::from([0x01; 20]);
        let account = create_test_account(1);

        // Insert an account
        let id = state.insert(address, account.clone());

        // Fetch existing account - should not call the fetch function
        let result: Result<_, &str> = state.get_mut_or_fetch(address, |_| {
            panic!("Fetch function should not be called for existing account");
        });

        assert!(result.is_ok());
        let (fetched_account, fetched_id) = result.unwrap();
        assert_eq!(fetched_account.info.nonce, 1);
        assert_eq!(*fetched_id.address(), address);
        assert_eq!(fetched_id.id(), id.id());
    }

    #[test]
    fn test_get_mut_or_fetch_new_account() {
        let mut state = EvmState::new();
        let address = Address::from([0x02; 20]);

        // Fetch new account - should call the fetch function
        let mut fetch_called = false;
        let result: Result<_, &str> = state.get_mut_or_fetch(address, |addr| {
            fetch_called = true;
            assert_eq!(addr, address);
            Ok(create_test_account(42))
        });

        assert!(fetch_called);
        assert!(result.is_ok());

        let (fetched_account, fetched_id) = result.unwrap();
        assert_eq!(fetched_account.info.nonce, 42);
        assert_eq!(*fetched_id.address(), address);
        assert_eq!(fetched_id.id(), (0, 0));

        // Verify account was added to state
        assert!(state.contains_key(&address));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_get_mut_or_fetch_error_propagation() {
        let mut state = EvmState::new();
        let address = Address::from([0x03; 20]);

        // Test error propagation
        let result = state.get_mut_or_fetch(address, |_| Err::<Account, &str>("fetch error"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "fetch error");

        // Verify account was not added to state
        assert!(!state.contains_key(&address));
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_get_mut_or_fetch_mutability() {
        let mut state = EvmState::new();
        let address = Address::from([0x04; 20]);

        // Fetch and modify new account
        let result: Result<_, &str> =
            state.get_mut_or_fetch(address, |_| Ok(create_test_account(10)));
        assert!(result.is_ok());

        let (account, _) = result.unwrap();
        account.info.nonce = 20;

        // Verify modification persisted
        let (account, _) = state.get(AddressOrId::Address(address)).unwrap();
        assert_eq!(account.info.nonce, 20);
    }

    #[test]
    fn test_get_mut_or_fetch_multiple_accounts() {
        let mut state = EvmState::new();
        let address1 = Address::from([0x05; 20]);
        let address2 = Address::from([0x06; 20]);
        let address3 = Address::from([0x07; 20]);
        let address4 = Address::from([0x08; 20]);

        // Add first account
        state.insert(address1, create_test_account(1));

        // Fetch existing and new accounts
        let result1: Result<_, ()> =
            state.get_mut_or_fetch(address1, |_| panic!("Should not fetch existing account"));
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().1.id(), (0, 0));

        let result2: Result<_, ()> =
            state.get_mut_or_fetch(address2, |_| Ok(create_test_account(2)));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().1.id(), (0, 1));

        let result3: Result<_, ()> =
            state.get_mut_or_fetch(address3, |_| Ok(create_test_account(3)));
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap().1.id(), (0, 2));

        let result4: Result<_, ()> =
            state.get_mut_or_fetch(address4, |_| Ok(create_test_account(4)));
        assert!(result4.is_ok());
        assert_eq!(result4.unwrap().1.id(), (1, 0));

        // Verify all accounts are in state
        assert_eq!(state.len(), 4);
        assert!(state.contains_key(&address1));
        assert!(state.contains_key(&address2));
        assert!(state.contains_key(&address3));
        assert!(state.contains_key(&address4));
    }

    #[test]
    fn test_pagination_behavior() {
        let mut state = EvmState::new();
        let mut addresses = Vec::new();

        // Add more than 100 accounts to test pagination
        for i in 0..150 {
            let mut bytes = [0u8; 20];
            bytes[19] = i as u8;
            let address = Address::from(bytes);
            addresses.push(address);
            let _: Result<_, &str> =
                state.get_mut_or_fetch(address, |_| Ok(create_test_account(i as u64)));
        }

        // Verify all accounts are accessible
        assert_eq!(state.len(), 150);
        for (i, address) in addresses.iter().enumerate() {
            let (account, _) = state.get(AddressOrId::Address(*address)).unwrap();
            assert_eq!(account.info.nonce, i as u64);
        }

        // Verify pagination structure
        assert!(state.accounts.len() >= 2); // Should have at least 2 pages
        assert_eq!(state.accounts[0].len(), 100); // First page should be full
    }

    #[test]
    fn test_get_by_id_and_address() {
        let mut state = EvmState::new();
        let address = Address::from([0x08; 20]);

        // Insert account and get its ID
        let id_info = state.insert(address, create_test_account(99));
        let id = id_info.id();

        // Test get by ID
        let (account_by_id, addr_id) = state.get_by_id(id);
        assert_eq!(account_by_id.info.nonce, 99);
        assert_eq!(*addr_id.address(), address);
        assert_eq!(addr_id.id(), id);

        // Test get by address
        let (account_by_addr, addr_id2) = state.get(AddressOrId::Address(address)).unwrap();
        assert_eq!(account_by_addr.info.nonce, 99);
        assert_eq!(addr_id2, addr_id);

        // Test get_mut by ID
        let (account_mut_by_id, _) = state.get_by_id_mut(id);
        account_mut_by_id.info.nonce = 100;

        // Verify modification
        let (account, _) = state.get_by_id(id);
        assert_eq!(account.info.nonce, 100);
    }

    #[test]
    fn test_insert_update_existing() {
        let mut state = EvmState::new();
        let address = Address::from([0x09; 20]);

        // Initial insert
        let id1 = state.insert(address, create_test_account(1));
        assert_eq!(state.len(), 1);

        // Update existing account
        let id2 = state.insert(address, create_test_account(2));
        assert_eq!(state.len(), 1); // Length should not change
        assert_eq!(id1.id(), id2.id()); // ID should remain the same

        // Verify update
        let (account, _) = state.get(AddressOrId::Address(address)).unwrap();
        assert_eq!(account.info.nonce, 2);
    }

    #[test]
    fn test_clear_and_is_empty() {
        let mut state = EvmState::new();
        assert!(state.is_empty());

        // Add some accounts
        for i in 0..5 {
            let mut bytes = [0u8; 20];
            bytes[19] = i as u8;
            let address = Address::from(bytes);
            state.insert(address, create_test_account(i));
        }

        assert!(!state.is_empty());
        assert_eq!(state.len(), 5);

        // Clear state
        state.clear();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert_eq!(state.accounts.len(), 0);
    }

    #[test]
    fn test_iter() {
        let mut state = EvmState::new();
        let mut expected_addresses = Vec::new();

        // Add accounts
        for i in 0..10 {
            let mut bytes = [0u8; 20];
            bytes[19] = i as u8;
            let address = Address::from(bytes);
            expected_addresses.push(address);
            state.insert(address, create_test_account(i));
        }

        // Iterate and collect
        let collected: Vec<_> = state.iter().collect();
        assert_eq!(collected.len(), 10);

        // Verify all addresses are present
        for (account, address) in collected {
            assert!(expected_addresses.contains(&address));
            let index = expected_addresses
                .iter()
                .position(|&a| a == *address)
                .unwrap();
            assert_eq!(account.info.nonce, index as u64);
        }
    }
}
