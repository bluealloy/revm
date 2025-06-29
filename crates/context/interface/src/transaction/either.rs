use super::Transaction;
use either::Either;
use primitives::{Address, Bytes, TxKind, B256, U256};

impl<L, R> Transaction for Either<L, R>
where
    L: Transaction + 'static,
    R: for<'a> Transaction<
            AccessListItem<'a> = L::AccessListItem<'a>,
            Authorization<'a> = L::Authorization<'a>,
        > + 'static,
{
    type AccessListItem<'a>
        = L::AccessListItem<'a>
    where
        Self: 'a;

    type Authorization<'a>
        = L::Authorization<'a>
    where
        Self: 'a;

    fn tx_type(&self) -> u8 {
        match self {
            Either::Left(l) => l.tx_type(),
            Either::Right(r) => r.tx_type(),
        }
    }

    fn caller(&self) -> Address {
        match self {
            Either::Left(l) => l.caller(),
            Either::Right(r) => r.caller(),
        }
    }

    fn caller_u256(&self) -> U256 {
        match self {
            Either::Left(l) => l.caller_u256(),
            Either::Right(r) => r.caller_u256(),
        }
    }

    fn gas_limit(&self) -> u64 {
        match self {
            Either::Left(l) => l.gas_limit(),
            Either::Right(r) => r.gas_limit(),
        }
    }

    fn value(&self) -> U256 {
        match self {
            Either::Left(l) => l.value(),
            Either::Right(r) => r.value(),
        }
    }

    fn input(&self) -> &Bytes {
        match self {
            Either::Left(l) => l.input(),
            Either::Right(r) => r.input(),
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            Either::Left(l) => l.nonce(),
            Either::Right(r) => r.nonce(),
        }
    }

    fn kind(&self) -> TxKind {
        match self {
            Either::Left(l) => l.kind(),
            Either::Right(r) => r.kind(),
        }
    }

    fn chain_id(&self) -> Option<u64> {
        match self {
            Either::Left(l) => l.chain_id(),
            Either::Right(r) => r.chain_id(),
        }
    }

    fn gas_price(&self) -> u128 {
        match self {
            Either::Left(l) => l.gas_price(),
            Either::Right(r) => r.gas_price(),
        }
    }

    fn access_list(&self) -> Option<impl Iterator<Item = Self::AccessListItem<'_>>> {
        match self {
            Either::Left(l) => l.access_list().map(Either::Left),
            Either::Right(r) => r.access_list().map(Either::Right),
        }
    }

    fn blob_versioned_hashes(&self) -> &[B256] {
        match self {
            Either::Left(l) => l.blob_versioned_hashes(),
            Either::Right(r) => r.blob_versioned_hashes(),
        }
    }

    fn max_fee_per_blob_gas(&self) -> u128 {
        match self {
            Either::Left(l) => l.max_fee_per_blob_gas(),
            Either::Right(r) => r.max_fee_per_blob_gas(),
        }
    }

    fn authorization_list_len(&self) -> usize {
        match self {
            Either::Left(l) => l.authorization_list_len(),
            Either::Right(r) => r.authorization_list_len(),
        }
    }

    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>> {
        match self {
            Either::Left(l) => Either::Left(l.authorization_list()),
            Either::Right(r) => Either::Right(r.authorization_list()),
        }
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        match self {
            Either::Left(l) => l.max_priority_fee_per_gas(),
            Either::Right(r) => r.max_priority_fee_per_gas(),
        }
    }
}
