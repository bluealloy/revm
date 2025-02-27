use crate::TransactionType;
use context_interface::transaction::{AccessList, SignedAuthorization, Transaction};
use core::fmt::Debug;
use primitives::{Address, Bytes, TxKind, B256, U256};
use std::vec::Vec;

/// The transaction environment
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    pub tx_type: u8,
    /// Caller aka Author aka transaction signer
    pub caller: Address,
    /// The gas limit of the transaction
    pub gas_limit: u64,
    /// The gas price of the transaction
    pub gas_price: u128,
    /// The destination of the transaction
    pub kind: TxKind,
    /// The value sent to `transact_to`
    pub value: U256,
    /// The data of the transaction
    pub data: Bytes,

    /// The nonce of the transaction
    pub nonce: u64,

    /// The chain ID of the transaction
    ///
    /// If set to [`None`], no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    pub chain_id: Option<u64>,

    /// A list of addresses and storage keys that the transaction plans to access
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    pub access_list: AccessList,

    /// The priority fee per gas
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub gas_priority_fee: Option<u128>,

    /// The list of blob versioned hashes
    ///
    /// Per EIP there should be at least one blob present if [`max_fee_per_blob_gas`][Self::max_fee_per_blob_gas] is [`Some`].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_hashes: Vec<B256>,

    /// The max fee per blob gas
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub max_fee_per_blob_gas: u128,

    /// List of authorizations
    ///
    /// `authorization_list` contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction via [EIP-7702].
    ///
    /// [EIP-7702]: https://eips.ethereum.org/EIPS/eip-7702
    pub authorization_list: Vec<SignedAuthorization>,
}

impl Default for TxEnv {
    fn default() -> Self {
        Self {
            tx_type: 0,
            caller: Address::default(),
            gas_limit: 30_000_000,
            gas_price: 0,
            kind: TxKind::Call(Address::default()),
            value: U256::ZERO,
            data: Bytes::default(),
            nonce: 0,
            chain_id: Some(1), // Mainnet chain ID is 1
            access_list: Default::default(),
            gas_priority_fee: Some(0),
            blob_hashes: Vec::new(),
            max_fee_per_blob_gas: 0,
            authorization_list: Vec::new(),
        }
    }
}

pub enum DeriveTxTypeError {
    MissingTargetForEip4844,
    MissingTargetForEip7702,
}

impl TxEnv {
    pub fn derive_tx_type(&mut self) -> Result<(), DeriveTxTypeError> {
        let mut tx_type = TransactionType::Legacy;

        if !self.access_list.0.is_empty() {
            tx_type = TransactionType::Eip2930;
        }

        if self.gas_priority_fee.is_some() {
            tx_type = TransactionType::Eip1559;
        }

        if !self.blob_hashes.is_empty() {
            if let TxKind::Call(_) = self.kind {
                tx_type = TransactionType::Eip4844;
            } else {
                return Err(DeriveTxTypeError::MissingTargetForEip4844);
            }
        }

        if !self.authorization_list.is_empty() {
            if let TxKind::Call(_) = self.kind {
                tx_type = TransactionType::Eip7702;
            } else {
                return Err(DeriveTxTypeError::MissingTargetForEip7702);
            }
        }

        self.tx_type = tx_type as u8;
        Ok(())
    }
}

impl Transaction for TxEnv {
    type AccessList = AccessList;
    type Authorization = SignedAuthorization;

    fn tx_type(&self) -> u8 {
        self.tx_type
    }

    fn kind(&self) -> TxKind {
        self.kind
    }

    fn caller(&self) -> Address {
        self.caller
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    fn gas_price(&self) -> u128 {
        self.gas_price
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    fn access_list(&self) -> Option<&Self::AccessList> {
        Some(&self.access_list)
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.gas_price
    }

    fn max_fee_per_blob_gas(&self) -> u128 {
        self.max_fee_per_blob_gas
    }

    fn authorization_list_len(&self) -> usize {
        self.authorization_list.len()
    }

    fn authorization_list(&self) -> impl Iterator<Item = &Self::Authorization> {
        self.authorization_list.iter()
    }

    fn input(&self) -> &Bytes {
        &self.data
    }

    fn blob_versioned_hashes(&self) -> &[B256] {
        &self.blob_hashes
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.gas_priority_fee
    }
}
