use auto_impl::auto_impl;
use primitives::{Address, B256};

/// Access list type is introduced in EIP-2930, and every
/// transaction after it contains access list.
///
/// **Note**: Iterator over access list returns account address and storage slot keys that
/// are warm loaded before transaction execution.
///
/// Number of account and storage slots is used to calculate initial tx gas cost.
#[auto_impl(&, Box, Arc, Rc)]
pub trait AccessListTrait {
    /// Iterate over access list.
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)>;

    /// Returns number of account and storage slots.
    fn access_list_nums(&self) -> (usize, usize) {
        let storage_num = self.access_list().map(|i| i.1.count()).sum();
        let account_num = self.access_list().count();

        (account_num, storage_num)
    }
}
