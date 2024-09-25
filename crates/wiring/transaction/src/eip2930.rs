use crate::CommonTxFields;
use primitives::{Address, TxKind, B256};

// TODO move to specs
pub trait AccessListInterface {
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)>;
}

pub trait Eip2930Tx: CommonTxFields {
    fn chain_id(&self) -> u64;

    fn gas_price(&self) -> u128;

    fn kind(&self) -> TxKind;

    fn access_list(&self) -> &impl AccessListInterface;
}

use specification::eip2930::AccessList;

impl AccessListInterface for AccessList {
    fn access_list(&self) -> impl Iterator<Item = (Address, impl Iterator<Item = B256>)> {
        self.0.iter().map(|item| {
            let slots = item.storage_keys.iter().map(|s| *s);
            (item.address, slots)
        })
    }
}
