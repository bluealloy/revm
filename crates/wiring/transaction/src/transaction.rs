use crate::{
    CommonTxFields, Eip1559Tx, Eip2930Tx, Eip4844Tx, Eip7702Tx, LegacyTx, TransactionType,
};
use core::fmt::Debug;
use primitives::{Address, Bytes, TxKind, B256, GAS_PER_BLOB, U256};
use specification::{eip2930, eip7702};

pub trait Transaction {
    type Legacy: LegacyTx;
    type Eip1559: Eip1559Tx;
    type Eip2930: Eip2930Tx;
    type Eip4844: Eip4844Tx;
    type Eip7702: Eip7702Tx;

    fn tx_type(&self) -> impl Into<TransactionType>;

    fn legacy(&self) -> &Self::Legacy {
        unimplemented!("legacy tx not supported")
    }

    fn eip2930(&self) -> &Self::Eip2930 {
        unimplemented!("Eip2930 tx not supported")
    }

    fn eip1559(&self) -> &Self::Eip1559 {
        unimplemented!("Eip1559 tx not supported")
    }

    fn eip4844(&self) -> &Self::Eip4844 {
        unimplemented!("Eip4844 tx not supported")
    }

    fn eip7702(&self) -> &Self::Eip7702 {
        unimplemented!("Eip7702 tx not supported")
    }

    fn common_fields(&self) -> &dyn CommonTxFields {
        match self.tx_type().into() {
            TransactionType::Legacy => self.legacy(),
            TransactionType::Eip2930 => self.eip2930(),
            TransactionType::Eip1559 => self.eip1559(),
            TransactionType::Eip4844 => self.eip4844(),
            TransactionType::Eip7702 => self.eip7702(),
        }
    }
}

/// Trait for retrieving transaction information required for execution.
pub trait TransactionOld {
    /// Caller aka Author aka transaction signer.
    fn caller(&self) -> &Address;
    /// The maximum amount of gas the transaction can use.
    fn gas_limit(&self) -> u64;
    /// The gas price the sender is willing to pay.
    fn gas_price(&self) -> &U256;
    /// Returns what kind of transaction this is.
    fn kind(&self) -> TxKind;
    /// The value sent to the receiver of `TxKind::Call`.
    fn value(&self) -> &U256;
    /// Returns the input data of the transaction.
    fn data(&self) -> &Bytes;
    /// The nonce of the transaction.
    fn nonce(&self) -> u64;
    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    fn chain_id(&self) -> Option<u64>;
    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    fn access_list(&self) -> &[eip2930::AccessListItem];
    /// The maximum priority fee per gas the sender is willing to pay.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    fn max_priority_fee_per_gas(&self) -> Option<&U256>;
    /// The list of blob versioned hashes. Per EIP there should be at least
    /// one blob present if [`Transaction::max_fee_per_blob_gas`] is `Some`.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_hashes(&self) -> &[B256];
    /// The maximum fee per blob gas the sender is willing to pay.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn max_fee_per_blob_gas(&self) -> Option<&U256>;
    /// List of authorizations, that contains the signature that authorizes this
    /// caller to place the code to signer account.
    ///
    /// Set EOA account code for one transaction
    ///
    /// [EIP-Set EOA account code for one transaction](https://eips.ethereum.org/EIPS/eip-7702)
    fn authorization_list(&self) -> Option<&eip7702::AuthorizationList>;

    /// See [EIP-4844], [`crate::default::Env::calc_data_fee`], and [`crate::default::Env::calc_max_data_fee`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn get_total_blob_gas(&self) -> u64 {
        GAS_PER_BLOB * self.blob_hashes().len() as u64
    }
}

pub trait TransactionValidation {
    /// An error that occurs when validating a transaction.
    type ValidationError: Debug + core::error::Error;
}
