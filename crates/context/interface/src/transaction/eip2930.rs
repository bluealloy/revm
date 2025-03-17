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
pub trait AccessListItemTr {
    /// Returns account address.
    fn address(&self) -> &Address;

    /// Returns storage slot keys.
    fn storage_slots(&self) -> impl Iterator<Item = &B256>;
}
