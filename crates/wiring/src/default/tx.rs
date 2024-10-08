use crate::{result::InvalidTransaction, Transaction};
use core::fmt::Debug;
use primitives::{Address, Bytes, TxKind, B256, U256};
use specification::eip2930::AccessList;
use specification::eip7702::AuthorizationList;
use std::vec::Vec;
use transaction::{
    eip7702::Authorization, CommonTxFields, Eip1559CommonTxFields, Eip1559Tx, Eip2930Tx, Eip4844Tx,
    Eip7702Tx, LegacyTx, TransactionType,
};

/// The transaction environment.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    pub tx_type: TransactionType,
    /// Caller aka Author aka transaction signer.
    pub caller: Address,
    /// The gas limit of the transaction.
    pub gas_limit: u64,
    /// The gas price of the transaction.
    pub gas_price: U256,
    /// The destination of the transaction.
    pub transact_to: TxKind,
    /// The value sent to `transact_to`.
    pub value: U256,
    /// The data of the transaction.
    pub data: Bytes,

    /// The nonce of the transaction.
    pub nonce: u64,

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    pub chain_id: Option<u64>,

    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    pub access_list: AccessList,

    /// The priority fee per gas.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub gas_priority_fee: Option<U256>,

    /// The list of blob versioned hashes. Per EIP there should be at least
    /// one blob present if [`Self::max_fee_per_blob_gas`] is `Some`.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_hashes: Vec<B256>,

    /// The max fee per blob gas.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub max_fee_per_blob_gas: Option<U256>,

    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    pub authorization_list: AuthorizationList,
}

impl Default for TxEnv {
    fn default() -> Self {
        Self {
            tx_type: TransactionType::Legacy,
            caller: Address::default(),
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            transact_to: TxKind::Call(Address::default()),
            value: U256::ZERO,
            data: Bytes::default(),
            nonce: 0,
            chain_id: Some(1), // Mainnet chain ID is 1
            access_list: AccessList::default(),
            gas_priority_fee: Some(U256::ZERO),
            blob_hashes: Vec::new(),
            max_fee_per_blob_gas: Some(U256::ZERO),
            authorization_list: AuthorizationList::default(),
        }
    }
}

impl CommonTxFields for TxEnv {
    fn caller(&self) -> Address {
        self.caller
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn input(&self) -> &Bytes {
        &self.data
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Eip1559CommonTxFields for TxEnv {
    type AccessList = AccessList;

    fn chain_id(&self) -> u64 {
        self.chain_id.unwrap_or_default()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.gas_price.to()
    }

    fn max_priority_fee_per_gas(&self) -> u128 {
        self.gas_priority_fee.unwrap_or_default().to()
    }

    fn access_list(&self) -> &Self::AccessList {
        &self.access_list
    }
}

impl LegacyTx for TxEnv {
    fn kind(&self) -> TxKind {
        self.transact_to
    }

    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    fn gas_price(&self) -> u128 {
        self.gas_price.try_into().unwrap_or(u128::MAX)
    }
}

impl Eip2930Tx for TxEnv {
    type AccessList = AccessList;

    fn access_list(&self) -> &Self::AccessList {
        &self.access_list
    }

    fn chain_id(&self) -> u64 {
        self.chain_id.unwrap_or_default()
    }

    fn gas_price(&self) -> u128 {
        self.gas_price.to()
    }

    fn kind(&self) -> TxKind {
        self.transact_to
    }
}

impl Eip1559Tx for TxEnv {
    fn kind(&self) -> TxKind {
        self.transact_to
    }
}

impl Eip4844Tx for TxEnv {
    fn destination(&self) -> Address {
        match self.transact_to {
            TxKind::Call(addr) => addr,
            TxKind::Create => panic!("Create transaction are not allowed in Eip4844"),
        }
    }

    fn blob_versioned_hashes(&self) -> &[B256] {
        &self.blob_hashes
    }

    fn max_fee_per_blob_gas(&self) -> u128 {
        self.max_fee_per_blob_gas.unwrap_or_default().to()
    }
}

impl Eip7702Tx for TxEnv {
    fn destination(&self) -> Address {
        match self.transact_to {
            TxKind::Call(addr) => addr,
            TxKind::Create => panic!("Create transaction are not allowed in Eip7702"),
        }
    }

    fn authorization_list_len(&self) -> usize {
        self.authorization_list.len()
    }

    fn authorization_list_iter(&self) -> impl Iterator<Item = impl Authorization> {
        self.authorization_list.recovered_iter()
    }
}

impl Transaction for TxEnv {
    type TransactionError = InvalidTransaction;
    type TransactionType = TransactionType;

    type AccessList = <Self::Eip2930 as Eip2930Tx>::AccessList;

    type Legacy = Self;

    type Eip1559 = Self;

    type Eip2930 = Self;

    type Eip4844 = Self;

    type Eip7702 = Self;

    fn tx_type(&self) -> Self::TransactionType {
        self.tx_type
    }

    fn legacy(&self) -> &Self::Legacy {
        self
    }

    fn eip2930(&self) -> &Self::Eip2930 {
        self
    }

    fn eip1559(&self) -> &Self::Eip1559 {
        self
    }

    fn eip4844(&self) -> &Self::Eip4844 {
        self
    }

    fn eip7702(&self) -> &Self::Eip7702 {
        self
    }
}
