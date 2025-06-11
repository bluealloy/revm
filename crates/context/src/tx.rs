//! This module contains [`TxEnv`] struct and implements [`Transaction`] trait for it.
use crate::TransactionType;
use context_interface::{
    either::Either,
    transaction::{
        AccessList, AccessListItem, Authorization, RecoveredAuthority, RecoveredAuthorization,
        SignedAuthorization, Transaction,
    },
};
use core::fmt::Debug;
use primitives::{Address, Bytes, TxKind, B256, U256};
use std::{vec, vec::Vec};

/// The Transaction Environment is a struct that contains all fields that can be found in all Ethereum transaction,
/// including EIP-4844, EIP-7702, EIP-7873, etc.  It implements the [`Transaction`] trait, which is used inside the EVM to execute a transaction.
///
/// [`TxEnvBuilder`] builder is recommended way to create a new [`TxEnv`] as it will automatically
/// set the transaction type based on the fields set.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    /// Transaction type
    pub tx_type: u8,
    /// Caller aka Author aka transaction signer
    pub caller: Address,
    /// The gas limit of the transaction.
    pub gas_limit: u64,
    /// The gas price of the transaction.
    ///
    /// For EIP-1559 transaction this represent max_gas_fee.
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
    pub authorization_list: Vec<Either<SignedAuthorization, RecoveredAuthorization>>,
    // TODO(EOF)
    // /// List of initcodes that is part of Initcode transaction.
    // ///
    // /// [EIP-7873](https://eips.ethereum.org/EIPS/eip-7873)
    // pub initcodes: Vec<Bytes>,
}

impl Default for TxEnv {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

/// Error type for deriving transaction type used as error in [`TxEnv::derive_tx_type`] function.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeriveTxTypeError {
    /// Missing target for EIP-4844
    MissingTargetForEip4844,
    /// Missing target for EIP-7702
    MissingTargetForEip7702,
    /// Missing target for EIP-7873
    MissingTargetForEip7873,
}

impl TxEnv {
    /// Derives tx type from transaction fields and sets it to `tx_type`.
    /// Returns error in case some fields were not set correctly.
    pub fn derive_tx_type(&mut self) -> Result<(), DeriveTxTypeError> {
        if !self.access_list.0.is_empty() {
            self.tx_type = TransactionType::Eip2930 as u8;
        }

        if self.gas_priority_fee.is_some() {
            self.tx_type = TransactionType::Eip1559 as u8;
        }

        if !self.blob_hashes.is_empty() || self.max_fee_per_blob_gas > 0 {
            if let TxKind::Call(_) = self.kind {
                self.tx_type = TransactionType::Eip4844 as u8;
                return Ok(());
            } else {
                return Err(DeriveTxTypeError::MissingTargetForEip4844);
            }
        }

        if !self.authorization_list.is_empty() {
            if let TxKind::Call(_) = self.kind {
                self.tx_type = TransactionType::Eip7702 as u8;
                return Ok(());
            } else {
                return Err(DeriveTxTypeError::MissingTargetForEip7702);
            }
        }

        // TODO(EOF)
        // if !self.initcodes.is_empty() {
        //     if let TxKind::Call(_) = self.kind {
        //         self.tx_type = TransactionType::Eip7873 as u8;
        //         return Ok(());
        //     } else {
        //         return Err(DeriveTxTypeError::MissingTargetForEip7873);
        //     }
        // }

        Ok(())
    }

    /// Insert a list of signed authorizations into the authorization list.
    pub fn set_signed_authorization(&mut self, auth: Vec<SignedAuthorization>) {
        self.authorization_list = auth.into_iter().map(Either::Left).collect();
    }

    /// Insert a list of recovered authorizations into the authorization list.
    pub fn set_recovered_authorization(&mut self, auth: Vec<RecoveredAuthorization>) {
        self.authorization_list = auth.into_iter().map(Either::Right).collect();
    }
}

impl Transaction for TxEnv {
    type AccessListItem<'a> = &'a AccessListItem;
    type Authorization<'a> = &'a Either<SignedAuthorization, RecoveredAuthorization>;

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

    fn access_list(&self) -> Option<impl Iterator<Item = Self::AccessListItem<'_>>> {
        Some(self.access_list.0.iter())
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

    fn authorization_list(&self) -> impl Iterator<Item = Self::Authorization<'_>> {
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

    // TODO(EOF)
    // fn initcodes(&self) -> &[Bytes] {
    //     &self.initcodes
    // }
}

/// Builder for constructing [`TxEnv`] instances
#[derive(Default, Debug)]
pub struct TxEnvBuilder {
    tx_type: Option<u8>,
    caller: Address,
    gas_limit: u64,
    gas_price: u128,
    kind: TxKind,
    value: U256,
    data: Bytes,
    nonce: u64,
    chain_id: Option<u64>,
    access_list: AccessList,
    gas_priority_fee: Option<u128>,
    blob_hashes: Vec<B256>,
    max_fee_per_blob_gas: u128,
    authorization_list: Vec<Either<SignedAuthorization, RecoveredAuthorization>>,
}

impl TxEnvBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            tx_type: None,
            caller: Address::default(),
            gas_limit: 30_000_000,
            gas_price: 0,
            kind: TxKind::Call(Address::default()),
            value: U256::ZERO,
            data: Bytes::default(),
            nonce: 0,
            chain_id: Some(1), // Mainnet chain ID is 1
            access_list: Default::default(),
            gas_priority_fee: None,
            blob_hashes: Vec::new(),
            max_fee_per_blob_gas: 0,
            authorization_list: Vec::new(),
        }
    }

    /// Set the transaction type
    pub fn tx_type(mut self, tx_type: Option<u8>) -> Self {
        self.tx_type = tx_type;
        self
    }

    /// Set the caller address
    pub fn caller(mut self, caller: Address) -> Self {
        self.caller = caller;
        self
    }

    /// Set the gas limit
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = gas_limit;
        self
    }

    /// Set the max fee per gas.
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: u128) -> Self {
        self.gas_price = max_fee_per_gas;
        self
    }

    /// Set the gas price
    pub fn gas_price(mut self, gas_price: u128) -> Self {
        self.gas_price = gas_price;
        self
    }

    /// Set the transaction kind
    pub fn kind(mut self, kind: TxKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the transaction value
    pub fn value(mut self, value: U256) -> Self {
        self.value = value;
        self
    }

    /// Set the transaction data
    pub fn data(mut self, data: Bytes) -> Self {
        self.data = data;
        self
    }

    /// Set the transaction nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set the chain ID
    pub fn chain_id(mut self, chain_id: Option<u64>) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Set the access list
    pub fn access_list(mut self, access_list: AccessList) -> Self {
        self.access_list = access_list;
        self
    }

    /// Set the gas priority fee
    pub fn gas_priority_fee(mut self, gas_priority_fee: Option<u128>) -> Self {
        self.gas_priority_fee = gas_priority_fee;
        self
    }

    /// Set the blob hashes
    pub fn blob_hashes(mut self, blob_hashes: Vec<B256>) -> Self {
        self.blob_hashes = blob_hashes;
        self
    }

    /// Set the max fee per blob gas
    pub fn max_fee_per_blob_gas(mut self, max_fee_per_blob_gas: u128) -> Self {
        self.max_fee_per_blob_gas = max_fee_per_blob_gas;
        self
    }

    /// Set the authorization list
    pub fn authorization_list(
        mut self,
        authorization_list: Vec<Either<SignedAuthorization, RecoveredAuthorization>>,
    ) -> Self {
        self.authorization_list = authorization_list;
        self
    }

    /// Build the final [`TxEnv`] with default values for missing fields.
    pub fn build_fill(mut self) -> TxEnv {
        let tx_type_not_set = self.tx_type.is_some();
        if let Some(tx_type) = self.tx_type {
            match TransactionType::from(tx_type) {
                TransactionType::Legacy => {
                    // do nothing
                }
                TransactionType::Eip2930 => {
                    // do nothing, all fields are set. Access list can be empty.
                }
                TransactionType::Eip1559 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        self.gas_priority_fee = Some(0);
                    }
                }
                TransactionType::Eip4844 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        self.gas_priority_fee = Some(0);
                    }

                    // blob hashes can be empty
                    if self.blob_hashes.is_empty() {
                        self.blob_hashes = vec![B256::default()];
                    }

                    // target is required
                    if !self.kind.is_call() {
                        self.kind = TxKind::Call(Address::default());
                    }
                }
                TransactionType::Eip7702 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        self.gas_priority_fee = Some(0);
                    }

                    // authorization list can be empty
                    if self.authorization_list.is_empty() {
                        // add dummy authorization
                        self.authorization_list =
                            vec![Either::Right(RecoveredAuthorization::new_unchecked(
                                Authorization {
                                    chain_id: U256::from(self.chain_id.unwrap_or(1)),
                                    address: self.caller,
                                    nonce: self.nonce,
                                },
                                RecoveredAuthority::Invalid,
                            ))];
                    }

                    // target is required
                    if !self.kind.is_call() {
                        self.kind = TxKind::Call(Address::default());
                    }
                }
                TransactionType::Custom => {
                    // do nothing
                }
            }
        }

        let mut tx = TxEnv {
            tx_type: self.tx_type.unwrap_or(0),
            caller: self.caller,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            kind: self.kind,
            value: self.value,
            data: self.data,
            nonce: self.nonce,
            chain_id: self.chain_id,
            access_list: self.access_list,
            gas_priority_fee: self.gas_priority_fee,
            blob_hashes: self.blob_hashes,
            max_fee_per_blob_gas: self.max_fee_per_blob_gas,
            authorization_list: self.authorization_list,
        };

        // if tx_type is not set, derive it from fields and fix errors.
        if tx_type_not_set {
            match tx.derive_tx_type() {
                Ok(_) => {}
                Err(DeriveTxTypeError::MissingTargetForEip4844) => {
                    tx.kind = TxKind::Call(Address::default());
                }
                Err(DeriveTxTypeError::MissingTargetForEip7702) => {
                    tx.kind = TxKind::Call(Address::default());
                }
                Err(DeriveTxTypeError::MissingTargetForEip7873) => {
                    tx.kind = TxKind::Call(Address::default());
                }
            }
        }

        tx
    }

    /// Build the final [`TxEnv`], returns error if some fields are wrongly set.
    /// If it is fine to fill missing fields with default values, use [`TxEnvBuilder::build_fill`] instead.
    pub fn build(self) -> Result<TxEnv, TxEnvBuildError> {
        // if tx_type is set, check if all needed fields are set correctly.
        if let Some(tx_type) = self.tx_type {
            match TransactionType::from(tx_type) {
                TransactionType::Legacy => {
                    // do nothing
                }
                TransactionType::Eip2930 => {
                    // do nothing, all fields are set. Access list can be empty.
                }
                TransactionType::Eip1559 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        return Err(TxEnvBuildError::MissingGasPriorityFeeForEip1559);
                    }
                }
                TransactionType::Eip4844 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        return Err(TxEnvBuildError::MissingGasPriorityFeeForEip1559);
                    }

                    // blob hashes can be empty
                    if self.blob_hashes.is_empty() {
                        return Err(TxEnvBuildError::MissingBlobHashesForEip4844);
                    }

                    // target is required
                    if !self.kind.is_call() {
                        return Err(TxEnvBuildError::MissingTargetForEip4844);
                    }
                }
                TransactionType::Eip7702 => {
                    // gas priority fee is required
                    if self.gas_priority_fee.is_none() {
                        return Err(TxEnvBuildError::MissingGasPriorityFeeForEip1559);
                    }

                    // authorization list can be empty
                    if self.authorization_list.is_empty() {
                        return Err(TxEnvBuildError::MissingAuthorizationListForEip7702);
                    }

                    // target is required
                    if !self.kind.is_call() {
                        return Err(DeriveTxTypeError::MissingTargetForEip4844.into());
                    }
                }
                _ => {
                    panic!()
                }
            }
        }

        let mut tx = TxEnv {
            tx_type: self.tx_type.unwrap_or(0),
            caller: self.caller,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            kind: self.kind,
            value: self.value,
            data: self.data,
            nonce: self.nonce,
            chain_id: self.chain_id,
            access_list: self.access_list,
            gas_priority_fee: self.gas_priority_fee,
            blob_hashes: self.blob_hashes,
            max_fee_per_blob_gas: self.max_fee_per_blob_gas,
            authorization_list: self.authorization_list,
        };

        // Derive tx type from fields, if some fields are wrongly set it will return an error.
        tx.derive_tx_type()?;

        Ok(tx)
    }
}

/// Error type for building [`TxEnv`]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TxEnvBuildError {
    /// Derive tx type error
    DeriveErr(DeriveTxTypeError),
    /// Missing priority fee for EIP-1559
    MissingGasPriorityFeeForEip1559,
    /// Missing blob hashes for EIP-4844
    MissingBlobHashesForEip4844,
    /// Missing authorization list for EIP-7702
    MissingAuthorizationListForEip7702,
    /// Missing target for EIP-4844
    MissingTargetForEip4844,
}

impl From<DeriveTxTypeError> for TxEnvBuildError {
    fn from(error: DeriveTxTypeError) -> Self {
        TxEnvBuildError::DeriveErr(error)
    }
}

impl TxEnv {
    /// Create a new builder for constructing a [`TxEnv`]
    pub fn builder() -> TxEnvBuilder {
        TxEnvBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effective_gas_setup(
        tx_type: TransactionType,
        gas_price: u128,
        gas_priority_fee: Option<u128>,
    ) -> u128 {
        let tx = TxEnv {
            tx_type: tx_type as u8,
            gas_price,
            gas_priority_fee,
            ..Default::default()
        };
        let base_fee = 100;
        tx.effective_gas_price(base_fee)
    }

    #[test]
    fn test_effective_gas_price() {
        assert_eq!(90, effective_gas_setup(TransactionType::Legacy, 90, None));
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Legacy, 90, Some(0))
        );
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Legacy, 90, Some(10))
        );
        assert_eq!(
            120,
            effective_gas_setup(TransactionType::Legacy, 120, Some(10))
        );
        assert_eq!(90, effective_gas_setup(TransactionType::Eip2930, 90, None));
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip2930, 90, Some(0))
        );
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip2930, 90, Some(10))
        );
        assert_eq!(
            120,
            effective_gas_setup(TransactionType::Eip2930, 120, Some(10))
        );
        assert_eq!(90, effective_gas_setup(TransactionType::Eip1559, 90, None));
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip1559, 90, Some(0))
        );
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip1559, 90, Some(10))
        );
        assert_eq!(
            110,
            effective_gas_setup(TransactionType::Eip1559, 120, Some(10))
        );
        assert_eq!(90, effective_gas_setup(TransactionType::Eip4844, 90, None));
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip4844, 90, Some(0))
        );
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip4844, 90, Some(10))
        );
        assert_eq!(
            110,
            effective_gas_setup(TransactionType::Eip4844, 120, Some(10))
        );
        assert_eq!(90, effective_gas_setup(TransactionType::Eip7702, 90, None));
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip7702, 90, Some(0))
        );
        assert_eq!(
            90,
            effective_gas_setup(TransactionType::Eip7702, 90, Some(10))
        );
        assert_eq!(
            110,
            effective_gas_setup(TransactionType::Eip7702, 120, Some(10))
        );
    }
}
