use primitives::{Address, B256};

/// Access list type is introduced in EIP-2930, and every
/// transaction after it contains access list.
///
/// Note
///
/// Iterator over access list returns account address and storage slot keys that
/// are warm loaded before transaction execution.
///
/// Number of account and storage slots is used to calculate initial tx gas cost.
pub trait AccessListTrait {
    /// Iterate over access list.
    fn iter(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)>;

    /// Returns number of account and storage slots.
    fn num_account_storages(&self) -> (usize, usize) {
        let storage_num = self.iter().map(|i| i.1.count()).sum();
        let account_num = self.iter().count();

        (account_num, storage_num)
    }
}

// TODO move to default context
use specification::eip2930::AccessList;

impl AccessListTrait for AccessList {
    fn iter(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.0.iter().map(|item| {
            let slots = item.storage_keys.iter().copied();
            (item.address, slots)
        })
    }
}
