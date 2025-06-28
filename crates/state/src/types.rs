use super::{Account, EvmStorageSlot};
use primitives::{
    address::{AddressTr, NamedAddress},
    Address, HashMap, IndexEntry, IndexMap, StorageKey, StorageValue,
};

/// EVM State is a mapping from addresses to accounts.
pub type EvmState = IndexMap<Address, Account>;

/// Structure used for EIP-1153 transient storage
pub type TransientStorage = HashMap<(Address, StorageKey), StorageValue>;

/// An account's Storage is a mapping from 256-bit integer keys to [EvmStorageSlot]s.
pub type EvmStorage = HashMap<StorageKey, EvmStorageSlot>;

/// EVM State is a mapping from addresses to accounts.
#[derive(Debug, Clone, Default)]
pub struct EvmState2 {
    inner: IndexMap<Address, Account>,
    // The caller address of the current transaction.
    caller: Option<usize>,
    // The target address of the current transaction.
    target: Option<usize>,
    // The beneficiary address of the current transaction.
    beneficiary: Option<usize>,
}

impl PartialEq for EvmState2 {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for EvmState2 {}

impl From<EvmState2> for EvmState {
    fn from(state: EvmState2) -> Self {
        state.inner
    }
}

impl EvmState2 {
    /// Returns a mutable reference to the inner state.
    pub fn as_inner_mut(&mut self) -> &mut IndexMap<Address, Account> {
        &mut self.inner
    }

    /// Returns an immutable reference to the inner state.
    pub fn as_inner(&self) -> &IndexMap<Address, Account> {
        &self.inner
    }

    /// Returns a mutable reference to the address
    pub fn get_mut<A: AddressTr>(&mut self, address: &A) -> Option<&mut Account> {
        self.on_account(
            address,
            |inner, index| {
                inner
                    .get_index_mut(index)
                    .expect("Account expected to be loaded")
                    .1
            },
            |inner, address| {
                inner
                    .get_full_mut(&address)
                    .map(|(index, _, val)| (index, val))
            },
        )
    }

    /// Returns an immutable reference or inserts a new account if it does not exist.
    pub fn get_mut_or_insert<A: AddressTr, E: 'static, F: FnOnce() -> Result<Account, E>>(
        &mut self,
        address: &A,
        on_insert: F,
    ) -> Result<(&mut Account, bool), E> {
        self.on_account(
            address,
            |inner, index| {
                Ok((
                    inner
                        .get_index_mut(index)
                        .expect("Account should be present after insertion")
                        .1,
                    true,
                ))
            },
            |inner, address| {
                let entry = inner.entry(address);
                let index = entry.index();
                let exist = matches!(entry, IndexEntry::Occupied(_));
                let res = match on_insert() {
                    Ok(account) => Ok((entry.or_insert(account), exist)),
                    Err(err) => Err(err),
                };
                Some((index, res))
            },
        )
        .expect("Account should be present after insertion")
    }

    /// Returns an immutable reference to the address or `None` if it does not exist.
    pub fn get<A: AddressTr>(&mut self, address: &A) -> Option<&Account> {
        self.on_account(
            address,
            |inner, index| {
                inner
                    .get_index(index)
                    .expect("Account expected to be loaded")
                    .1
            },
            |inner, address| inner.get_full(&address).map(|(index, _, val)| (index, val)),
        )
    }

    fn on_account<
        'a: 'b,
        'b,
        A: AddressTr,
        B: 'a,
        F1: FnOnce(&'b mut IndexMap<Address, Account>, usize) -> B,
        F2: FnOnce(&'b mut IndexMap<Address, Account>, Address) -> Option<(usize, B)>,
    >(
        &'a mut self,
        address: &A,
        on_index: F1,
        on_address: F2,
    ) -> Option<B> {
        let (index, address) = match address.into_named_address() {
            NamedAddress::Caller(address) => (Some(&mut self.caller), address),
            NamedAddress::Target(address) => (Some(&mut self.target), address),
            NamedAddress::Beneficiary(address) => (Some(&mut self.beneficiary), address),
            NamedAddress::Unnamed(address) => (None, address),
        };

        if let Some(index_opt) = index {
            if let Some(index) = *index_opt {
                return Some(on_index(&mut self.inner, index));
            } else {
                if let Some((index, res)) = on_address(&mut self.inner, address) {
                    *index_opt = Some(index);
                    return Some(res);
                }
                return None;
            }
        }
        on_address(&mut self.inner, address).map(|(_, res)| res)
    }
}
