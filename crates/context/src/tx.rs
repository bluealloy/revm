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
use database_interface::{BENCH_CALLER, BENCH_TARGET};
use primitives::{eip7825, Address, Bytes, TxKind, B256, U256};
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
    /// Creates a new TxEnv with benchmark-specific values.
    pub fn new_bench() -> Self {
        Self {
            caller: BENCH_CALLER,
            kind: TxKind::Call(BENCH_TARGET),
            gas_limit: 1_000_000_000,
            ..Default::default()
        }
    }

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
            gas_limit: eip7825::TX_GAS_LIMIT_CAP,
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

    /// Get the transaction type
    pub fn get_tx_type(&self) -> Option<u8> {
        self.tx_type
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

    /// Set the transaction kind to call
    pub fn call(mut self, target: Address) -> Self {
        self.kind = TxKind::Call(target);
        self
    }

    /// Set the transaction kind to create
    pub fn create(mut self) -> Self {
        self.kind = TxKind::Create;
        self
    }

    /// Set the transaction kind to create
    pub fn to(self, target: Address) -> Self {
        self.call(target)
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

    /// Insert a list of signed authorizations into the authorization list.
    pub fn authorization_list_signed(mut self, auth: Vec<SignedAuthorization>) -> Self {
        self.authorization_list = auth.into_iter().map(Either::Left).collect();
        self
    }

    /// Insert a list of recovered authorizations into the authorization list.
    pub fn authorization_list_recovered(mut self, auth: Vec<RecoveredAuthorization>) -> Self {
        self.authorization_list = auth.into_iter().map(Either::Right).collect();
        self
    }

    /// Build the final [`TxEnv`] with default values for missing fields.
    pub fn build_fill(mut self) -> TxEnv {
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
        if self.tx_type.is_none() {
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
                TransactionType::Custom => {
                    // do nothing, custom transaction type is handled by the caller.
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
        if self.tx_type.is_none() {
            tx.derive_tx_type()?;
        }

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

    /// Create a new builder for constructing a [`TxEnv`] with benchmark-specific values.
    pub fn builder_for_bench() -> TxEnvBuilder {
        TxEnv::new_bench().modify()
    }

    /// Modify the [`TxEnv`] by using builder pattern.
    pub fn modify(self) -> TxEnvBuilder {
        let TxEnv {
            tx_type,
            caller,
            gas_limit,
            gas_price,
            kind,
            value,
            data,
            nonce,
            chain_id,
            access_list,
            gas_priority_fee,
            blob_hashes,
            max_fee_per_blob_gas,
            authorization_list,
        } = self;

        TxEnvBuilder::new()
            .tx_type(Some(tx_type))
            .caller(caller)
            .gas_limit(gas_limit)
            .gas_price(gas_price)
            .kind(kind)
            .value(value)
            .data(data)
            .nonce(nonce)
            .chain_id(chain_id)
            .access_list(access_list)
            .gas_priority_fee(gas_priority_fee)
            .blob_hashes(blob_hashes)
            .max_fee_per_blob_gas(max_fee_per_blob_gas)
            .authorization_list(authorization_list)
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
    fn test_tx_env_builder_build_valid_legacy() {
        // Legacy transaction
        let tx = TxEnvBuilder::new()
            .tx_type(Some(0))
            .caller(Address::from([1u8; 20]))
            .gas_limit(21000)
            .gas_price(20)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .value(U256::from(100))
            .data(Bytes::from(vec![0x01, 0x02]))
            .nonce(5)
            .chain_id(Some(1))
            .build()
            .unwrap();

        assert_eq!(tx.kind, TxKind::Call(Address::from([2u8; 20])));
        assert_eq!(tx.caller, Address::from([1u8; 20]));
        assert_eq!(tx.gas_limit, 21000);
        assert_eq!(tx.gas_price, 20);
        assert_eq!(tx.value, U256::from(100));
        assert_eq!(tx.data, Bytes::from(vec![0x01, 0x02]));
        assert_eq!(tx.nonce, 5);
        assert_eq!(tx.chain_id, Some(1));
        assert_eq!(tx.tx_type, TransactionType::Legacy);
    }

    #[test]
    fn test_tx_env_builder_build_valid_eip2930() {
        // EIP-2930 transaction with access list
        let access_list = AccessList(vec![AccessListItem {
            address: Address::from([3u8; 20]),
            storage_keys: vec![B256::from([4u8; 32])],
        }]);
        let tx = TxEnvBuilder::new()
            .tx_type(Some(1))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(25)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .access_list(access_list.clone())
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip2930);
        assert_eq!(tx.access_list, access_list);
    }

    #[test]
    fn test_tx_env_builder_build_valid_eip1559() {
        // EIP-1559 transaction
        let tx = TxEnvBuilder::new()
            .tx_type(Some(2))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(30)
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip1559);
        assert_eq!(tx.gas_priority_fee, Some(10));
    }

    #[test]
    fn test_tx_env_builder_build_valid_eip4844() {
        // EIP-4844 blob transaction
        let blob_hashes = vec![B256::from([5u8; 32]), B256::from([6u8; 32])];
        let tx = TxEnvBuilder::new()
            .tx_type(Some(3))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(30)
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .blob_hashes(blob_hashes.clone())
            .max_fee_per_blob_gas(100)
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip4844);
        assert_eq!(tx.blob_hashes, blob_hashes);
        assert_eq!(tx.max_fee_per_blob_gas, 100);
    }

    #[test]
    fn test_tx_env_builder_build_valid_eip7702() {
        // EIP-7702 EOA code transaction
        let auth = RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                nonce: 0,
                address: Address::default(),
            },
            RecoveredAuthority::Valid(Address::default()),
        );
        let auth_list = vec![Either::Right(auth)];

        let tx = TxEnvBuilder::new()
            .tx_type(Some(4))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(30)
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .authorization_list(auth_list.clone())
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip7702);
        assert_eq!(tx.authorization_list.len(), 1);
    }

    #[test]
    fn test_tx_env_builder_build_create_transaction() {
        // Contract creation transaction
        let bytecode = Bytes::from(vec![0x60, 0x80, 0x60, 0x40]);
        let tx = TxEnvBuilder::new()
            .kind(TxKind::Create)
            .data(bytecode.clone())
            .gas_limit(100000)
            .gas_price(20)
            .build()
            .unwrap();

        assert_eq!(tx.kind, TxKind::Create);
        assert_eq!(tx.data, bytecode);
    }

    #[test]
    fn test_tx_env_builder_build_errors_eip1559_missing_priority_fee() {
        // EIP-1559 without gas_priority_fee should fail
        let result = TxEnvBuilder::new()
            .tx_type(Some(2))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(30)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build();

        assert!(matches!(
            result,
            Err(TxEnvBuildError::MissingGasPriorityFeeForEip1559)
        ));
    }

    #[test]
    fn test_tx_env_builder_build_errors_eip4844_missing_blob_hashes() {
        // EIP-4844 without blob hashes should fail
        let result = TxEnvBuilder::new()
            .tx_type(Some(3))
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build();

        assert!(matches!(
            result,
            Err(TxEnvBuildError::MissingBlobHashesForEip4844)
        ));
    }

    #[test]
    fn test_tx_env_builder_build_errors_eip4844_not_call() {
        // EIP-4844 with Create should fail
        let result = TxEnvBuilder::new()
            .tx_type(Some(3))
            .gas_priority_fee(Some(10))
            .blob_hashes(vec![B256::from([5u8; 32])])
            .kind(TxKind::Create)
            .build();

        assert!(matches!(
            result,
            Err(TxEnvBuildError::MissingTargetForEip4844)
        ));
    }

    #[test]
    fn test_tx_env_builder_build_errors_eip7702_missing_auth_list() {
        // EIP-7702 without authorization list should fail
        let result = TxEnvBuilder::new()
            .tx_type(Some(4))
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build();

        assert!(matches!(
            result,
            Err(TxEnvBuildError::MissingAuthorizationListForEip7702)
        ));
    }

    #[test]
    fn test_tx_env_builder_build_errors_eip7702_not_call() {
        // EIP-7702 with Create should fail
        let auth = RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                nonce: 0,
                address: Address::default(),
            },
            RecoveredAuthority::Valid(Address::default()),
        );
        let result = TxEnvBuilder::new()
            .tx_type(Some(4))
            .gas_priority_fee(Some(10))
            .authorization_list(vec![Either::Right(auth)])
            .kind(TxKind::Create)
            .build();

        assert!(matches!(result, Err(TxEnvBuildError::DeriveErr(_))));
    }

    #[test]
    fn test_tx_env_builder_build_fill_legacy() {
        // Legacy transaction with build_fill
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_limit(21000)
            .gas_price(20)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Legacy);
        assert_eq!(tx.gas_priority_fee, None);
    }

    #[test]
    fn test_tx_env_builder_build_fill_eip1559_missing_priority_fee() {
        // EIP-1559 without gas_priority_fee should be filled with 0
        let tx = TxEnvBuilder::new()
            .tx_type(Some(2))
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(30)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Eip1559);
        assert_eq!(tx.gas_priority_fee, Some(0));
    }

    #[test]
    fn test_tx_env_builder_build_fill_eip4844_missing_blob_hashes() {
        // EIP-4844 without blob hashes should add default blob hash
        let tx = TxEnvBuilder::new()
            .tx_type(Some(3))
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Eip4844);
        assert_eq!(tx.blob_hashes.len(), 1);
        assert_eq!(tx.blob_hashes[0], B256::default());
    }

    #[test]
    fn test_tx_env_builder_build_fill_eip4844_create_to_call() {
        // EIP-4844 with Create should be converted to Call
        let tx = TxEnvBuilder::new()
            .tx_type(Some(3))
            .gas_priority_fee(Some(10))
            .blob_hashes(vec![B256::from([5u8; 32])])
            .kind(TxKind::Create)
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Eip4844);
        assert_eq!(tx.kind, TxKind::Call(Address::default()));
    }

    #[test]
    fn test_tx_env_builder_build_fill_eip7702_missing_auth_list() {
        // EIP-7702 without authorization list should add dummy auth
        let tx = TxEnvBuilder::new()
            .tx_type(Some(4))
            .gas_priority_fee(Some(10))
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Eip7702);
        assert_eq!(tx.authorization_list.len(), 1);
    }

    #[test]
    fn test_tx_env_builder_build_fill_eip7702_create_to_call() {
        // EIP-7702 with Create should be converted to Call
        let auth = RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                nonce: 0,
                address: Address::default(),
            },
            RecoveredAuthority::Valid(Address::default()),
        );
        let tx = TxEnvBuilder::new()
            .tx_type(Some(4))
            .gas_priority_fee(Some(10))
            .authorization_list(vec![Either::Right(auth)])
            .kind(TxKind::Create)
            .build_fill();

        assert_eq!(tx.tx_type, TransactionType::Eip7702);
        assert_eq!(tx.kind, TxKind::Call(Address::default()));
    }

    #[test]
    fn test_tx_env_builder_derive_tx_type_legacy() {
        // No special fields, should derive Legacy
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_limit(21000)
            .gas_price(20)
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Legacy);
    }

    #[test]
    fn test_tx_env_builder_derive_tx_type_eip2930() {
        // Access list present, should derive EIP-2930
        let access_list = AccessList(vec![AccessListItem {
            address: Address::from([3u8; 20]),
            storage_keys: vec![B256::from([4u8; 32])],
        }]);
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .access_list(access_list)
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip2930);
    }

    #[test]
    fn test_tx_env_builder_derive_tx_type_eip1559() {
        // Gas priority fee present, should derive EIP-1559
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_priority_fee(Some(10))
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip1559);
    }

    #[test]
    fn test_tx_env_builder_derive_tx_type_eip4844() {
        // Blob hashes present, should derive EIP-4844
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_priority_fee(Some(10))
            .blob_hashes(vec![B256::from([5u8; 32])])
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip4844);
    }

    #[test]
    fn test_tx_env_builder_derive_tx_type_eip7702() {
        // Authorization list present, should derive EIP-7702
        let auth = RecoveredAuthorization::new_unchecked(
            Authorization {
                chain_id: U256::from(1),
                nonce: 0,
                address: Address::default(),
            },
            RecoveredAuthority::Valid(Address::default()),
        );
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_priority_fee(Some(10))
            .authorization_list(vec![Either::Right(auth)])
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Eip7702);
    }

    #[test]
    fn test_tx_env_builder_custom_tx_type() {
        // Custom transaction type (0xFF)
        let tx = TxEnvBuilder::new()
            .tx_type(Some(0xFF))
            .caller(Address::from([1u8; 20]))
            .build()
            .unwrap();

        assert_eq!(tx.tx_type, TransactionType::Custom);
    }

    #[test]
    fn test_tx_env_builder_chain_methods() {
        // Test method chaining
        let tx = TxEnvBuilder::new()
            .caller(Address::from([1u8; 20]))
            .gas_limit(50000)
            .gas_price(25)
            .kind(TxKind::Call(Address::from([2u8; 20])))
            .value(U256::from(1000))
            .data(Bytes::from(vec![0x12, 0x34]))
            .nonce(10)
            .chain_id(Some(5))
            .access_list(AccessList(vec![AccessListItem {
                address: Address::from([3u8; 20]),
                storage_keys: vec![],
            }]))
            .gas_priority_fee(Some(5))
            .blob_hashes(vec![B256::from([7u8; 32])])
            .max_fee_per_blob_gas(200)
            .build_fill();

        assert_eq!(tx.caller, Address::from([1u8; 20]));
        assert_eq!(tx.gas_limit, 50000);
        assert_eq!(tx.gas_price, 25);
        assert_eq!(tx.kind, TxKind::Call(Address::from([2u8; 20])));
        assert_eq!(tx.value, U256::from(1000));
        assert_eq!(tx.data, Bytes::from(vec![0x12, 0x34]));
        assert_eq!(tx.nonce, 10);
        assert_eq!(tx.chain_id, Some(5));
        assert_eq!(tx.access_list.len(), 1);
        assert_eq!(tx.gas_priority_fee, Some(5));
        assert_eq!(tx.blob_hashes.len(), 1);
        assert_eq!(tx.max_fee_per_blob_gas, 200);
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
