use crate::CommonTxFields;
use primitives::{Address, TxKind, B256};

// TODO move to specs impl iterator trait
pub trait AccessListInterface {
    fn iter(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)>;

    /// Not performant way to count number of account and storages.
    fn num_account_storages(&self) -> (usize, usize) {
        let storage_num = self.iter().map(|i| i.1.count()).sum();
        let account_num = self.iter().count();

        (account_num, storage_num)
    }
}

pub trait Eip2930Tx: CommonTxFields {
    type AccessList: AccessListInterface;

    fn chain_id(&self) -> u64;

    fn gas_price(&self) -> u128;

    fn kind(&self) -> TxKind;

    fn access_list(&self) -> &Self::AccessList;
}

// TODO move to default context
use specification::eip2930::AccessList;

impl AccessListInterface for AccessList {
    fn iter(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.0.iter().map(|item| {
            let slots = item.storage_keys.iter().copied();
            (item.address, slots)
        })
    }
}
