//! EEST (Ethereum Execution Spec Tests) Exception Mapping
//!
//! This module provides mapping between REVM's `InvalidTransaction` errors
//! and EEST's `TransactionException` strings used in state tests.
//!
//! Reference: https://github.com/ethereum/execution-spec-tests
//! Exception definitions: src/ethereum_test_exceptions/exceptions/transaction.py

use revm::context_interface::result::InvalidTransaction;

/// Maps an EEST exception string to check if it matches a REVM InvalidTransaction error.
///
/// EEST exceptions can be combined with `|` (pipe) to indicate multiple acceptable exceptions.
/// This function returns `true` if the error matches ANY of the expected exceptions.
pub fn error_matches_exception(error: &InvalidTransaction, expected_exception: &str) -> bool {
    // Split by pipe to handle multiple acceptable exceptions
    expected_exception
        .split('|')
        .any(|exception| error_matches_single_exception(error, exception.trim()))
}

/// Maps a single EEST exception string to a REVM InvalidTransaction error.
fn error_matches_single_exception(error: &InvalidTransaction, exception: &str) -> bool {
    match exception {
        // === EIP-4844 Blob Transaction Errors ===
        "TransactionException.TYPE_3_TX_CONTRACT_CREATION" => {
            matches!(error, InvalidTransaction::BlobCreateTransaction)
        }
        "TransactionException.TYPE_3_TX_ZERO_BLOBS" => {
            matches!(error, InvalidTransaction::EmptyBlobs)
        }
        "TransactionException.TYPE_3_TX_BLOB_COUNT_EXCEEDED"
        | "TransactionException.TYPE_3_TX_MAX_BLOB_GAS_ALLOWANCE_EXCEEDED" => {
            // Both exceptions can map to TooManyBlobs - the difference is semantic:
            // - BLOB_COUNT_EXCEEDED: transaction has too many blobs for the tx limit
            // - MAX_BLOB_GAS_ALLOWANCE_EXCEEDED: transaction would exceed block blob gas
            // In REVM, both are returned as TooManyBlobs since we check against max_blobs_per_tx
            matches!(error, InvalidTransaction::TooManyBlobs { .. })
        }
        "TransactionException.TYPE_3_TX_INVALID_BLOB_VERSIONED_HASH" => {
            matches!(error, InvalidTransaction::BlobVersionNotSupported)
        }
        "TransactionException.TYPE_3_TX_PRE_FORK" => {
            matches!(error, InvalidTransaction::Eip4844NotSupported)
        }
        "TransactionException.INSUFFICIENT_MAX_FEE_PER_BLOB_GAS" => {
            matches!(error, InvalidTransaction::BlobGasPriceGreaterThanMax { .. })
        }

        // === EIP-7702 SetCode Transaction Errors ===
        "TransactionException.TYPE_4_TX_CONTRACT_CREATION" => {
            matches!(error, InvalidTransaction::Eip7702CreateTransaction)
        }
        "TransactionException.TYPE_4_EMPTY_AUTHORIZATION_LIST" => {
            matches!(error, InvalidTransaction::EmptyAuthorizationList)
        }
        "TransactionException.TYPE_4_TX_PRE_FORK" => {
            matches!(error, InvalidTransaction::Eip7702NotSupported)
        }
        "TransactionException.TYPE_4_INVALID_AUTHORITY_SIGNATURE"
        | "TransactionException.TYPE_4_INVALID_AUTHORITY_SIGNATURE_S_TOO_HIGH"
        | "TransactionException.TYPE_4_INVALID_AUTHORIZATION_FORMAT" => {
            matches!(error, InvalidTransaction::AuthorizationListInvalidFields)
        }

        // === EIP-1559 Fee Errors ===
        "TransactionException.PRIORITY_GREATER_THAN_MAX_FEE_PER_GAS"
        | "TransactionException.PRIORITY_GREATER_THAN_MAX_FEE_PER_GAS_2" => {
            matches!(error, InvalidTransaction::PriorityFeeGreaterThanMaxFee)
        }
        "TransactionException.INSUFFICIENT_MAX_FEE_PER_GAS" => {
            matches!(error, InvalidTransaction::GasPriceLessThanBasefee)
        }

        // === Gas Limit Errors ===
        "TransactionException.INTRINSIC_GAS_TOO_LOW" => {
            matches!(error, InvalidTransaction::CallGasCostMoreThanGasLimit { .. })
        }
        "TransactionException.INTRINSIC_GAS_BELOW_FLOOR_GAS_COST" => {
            matches!(error, InvalidTransaction::GasFloorMoreThanGasLimit { .. })
        }
        "TransactionException.GAS_ALLOWANCE_EXCEEDED" => {
            matches!(error, InvalidTransaction::CallerGasLimitMoreThanBlock)
        }
        "TransactionException.GAS_LIMIT_EXCEEDS_MAXIMUM" => {
            matches!(error, InvalidTransaction::TxGasLimitGreaterThanCap { .. })
        }

        // === Nonce Errors ===
        "TransactionException.NONCE_IS_MAX" | "TransactionException.NONCE_OVERFLOW" => {
            matches!(error, InvalidTransaction::NonceOverflowInTransaction)
        }
        "TransactionException.NONCE_MISMATCH_TOO_HIGH" | "TransactionException.NONCE_TOO_BIG" => {
            matches!(error, InvalidTransaction::NonceTooHigh { .. })
        }
        "TransactionException.NONCE_MISMATCH_TOO_LOW" => {
            matches!(error, InvalidTransaction::NonceTooLow { .. })
        }

        // === Account/Balance Errors ===
        "TransactionException.INSUFFICIENT_ACCOUNT_FUNDS" => {
            matches!(error, InvalidTransaction::LackOfFundForMaxFee { .. })
        }
        "TransactionException.SENDER_NOT_EOA" => {
            matches!(error, InvalidTransaction::RejectCallerWithCode)
        }

        // === Contract Creation Errors ===
        "TransactionException.INITCODE_SIZE_EXCEEDED" => {
            matches!(error, InvalidTransaction::CreateInitCodeSizeLimit)
        }

        // === Chain ID Errors ===
        "TransactionException.INVALID_CHAINID" => {
            matches!(
                error,
                InvalidTransaction::InvalidChainId | InvalidTransaction::MissingChainId
            )
        }

        // === Transaction Type Support Errors ===
        "TransactionException.TYPE_NOT_SUPPORTED" => {
            matches!(
                error,
                InvalidTransaction::Eip2930NotSupported
                    | InvalidTransaction::Eip1559NotSupported
                    | InvalidTransaction::Eip4844NotSupported
                    | InvalidTransaction::Eip7702NotSupported
                    | InvalidTransaction::Eip7873NotSupported
            )
        }

        // === Overflow Errors ===
        "TransactionException.GASLIMIT_PRICE_PRODUCT_OVERFLOW"
        | "TransactionException.VALUE_OVERFLOW"
        | "TransactionException.GASPRICE_OVERFLOW"
        | "TransactionException.PRIORITY_OVERFLOW" => {
            matches!(error, InvalidTransaction::OverflowPaymentInTransaction)
        }

        // === EIP-7873 Initcode Transaction Errors ===
        "TransactionException.INITCODE_TX_CONTRACT_CREATION" => {
            matches!(error, InvalidTransaction::Eip7873MissingTarget)
        }

        // Fallback: unknown exception, no match
        _ => false,
    }
}

/// Converts a REVM InvalidTransaction error to its corresponding EEST exception string.
///
/// This is useful for generating test output that matches EEST format.
pub fn error_to_exception_string(error: &InvalidTransaction) -> &'static str {
    match error {
        // EIP-4844 Blob Transactions
        InvalidTransaction::BlobCreateTransaction => {
            "TransactionException.TYPE_3_TX_CONTRACT_CREATION"
        }
        InvalidTransaction::EmptyBlobs => "TransactionException.TYPE_3_TX_ZERO_BLOBS",
        InvalidTransaction::TooManyBlobs { .. } => {
            "TransactionException.TYPE_3_TX_BLOB_COUNT_EXCEEDED"
        }
        InvalidTransaction::BlobVersionNotSupported => {
            "TransactionException.TYPE_3_TX_INVALID_BLOB_VERSIONED_HASH"
        }
        InvalidTransaction::BlobGasPriceGreaterThanMax { .. } => {
            "TransactionException.INSUFFICIENT_MAX_FEE_PER_BLOB_GAS"
        }
        InvalidTransaction::Eip4844NotSupported
        | InvalidTransaction::MaxFeePerBlobGasNotSupported
        | InvalidTransaction::BlobVersionedHashesNotSupported => {
            "TransactionException.TYPE_3_TX_PRE_FORK"
        }

        // EIP-7702 SetCode Transactions
        InvalidTransaction::Eip7702CreateTransaction => {
            "TransactionException.TYPE_4_TX_CONTRACT_CREATION"
        }
        InvalidTransaction::EmptyAuthorizationList => {
            "TransactionException.TYPE_4_EMPTY_AUTHORIZATION_LIST"
        }
        InvalidTransaction::Eip7702NotSupported
        | InvalidTransaction::AuthorizationListNotSupported => {
            "TransactionException.TYPE_4_TX_PRE_FORK"
        }
        InvalidTransaction::AuthorizationListInvalidFields => {
            "TransactionException.TYPE_4_INVALID_AUTHORIZATION_FORMAT"
        }

        // EIP-1559 Fees
        InvalidTransaction::PriorityFeeGreaterThanMaxFee => {
            "TransactionException.PRIORITY_GREATER_THAN_MAX_FEE_PER_GAS"
        }
        InvalidTransaction::GasPriceLessThanBasefee => {
            "TransactionException.INSUFFICIENT_MAX_FEE_PER_GAS"
        }

        // Gas Limits
        InvalidTransaction::CallGasCostMoreThanGasLimit { .. } => {
            "TransactionException.INTRINSIC_GAS_TOO_LOW"
        }
        InvalidTransaction::GasFloorMoreThanGasLimit { .. } => {
            "TransactionException.INTRINSIC_GAS_BELOW_FLOOR_GAS_COST"
        }
        InvalidTransaction::CallerGasLimitMoreThanBlock => {
            "TransactionException.GAS_ALLOWANCE_EXCEEDED"
        }
        InvalidTransaction::TxGasLimitGreaterThanCap { .. } => {
            "TransactionException.GAS_LIMIT_EXCEEDS_MAXIMUM"
        }

        // Nonces
        InvalidTransaction::NonceOverflowInTransaction => "TransactionException.NONCE_IS_MAX",
        InvalidTransaction::NonceTooHigh { .. } => "TransactionException.NONCE_MISMATCH_TOO_HIGH",
        InvalidTransaction::NonceTooLow { .. } => "TransactionException.NONCE_MISMATCH_TOO_LOW",

        // Account/Balance
        InvalidTransaction::LackOfFundForMaxFee { .. } => {
            "TransactionException.INSUFFICIENT_ACCOUNT_FUNDS"
        }
        InvalidTransaction::RejectCallerWithCode => "TransactionException.SENDER_NOT_EOA",

        // Contract Creation
        InvalidTransaction::CreateInitCodeSizeLimit => {
            "TransactionException.INITCODE_SIZE_EXCEEDED"
        }

        // Chain ID
        InvalidTransaction::InvalidChainId | InvalidTransaction::MissingChainId => {
            "TransactionException.INVALID_CHAINID"
        }

        // Transaction Type Support
        InvalidTransaction::Eip2930NotSupported
        | InvalidTransaction::Eip1559NotSupported
        | InvalidTransaction::AccessListNotSupported => {
            "TransactionException.TYPE_NOT_SUPPORTED"
        }

        // Overflows
        InvalidTransaction::OverflowPaymentInTransaction => {
            "TransactionException.GASLIMIT_PRICE_PRODUCT_OVERFLOW"
        }

        // EIP-7873
        InvalidTransaction::Eip7873NotSupported => "TransactionException.TYPE_NOT_SUPPORTED",
        InvalidTransaction::Eip7873MissingTarget => {
            "TransactionException.INITCODE_TX_CONTRACT_CREATION"
        }

        // Custom/Unknown
        InvalidTransaction::Str(_) => "TransactionException.UNDEFINED",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_create_transaction() {
        let error = InvalidTransaction::BlobCreateTransaction;
        assert!(error_matches_exception(
            &error,
            "TransactionException.TYPE_3_TX_CONTRACT_CREATION"
        ));
    }

    #[test]
    fn test_eip7702_create_transaction() {
        let error = InvalidTransaction::Eip7702CreateTransaction;
        assert!(error_matches_exception(
            &error,
            "TransactionException.TYPE_4_TX_CONTRACT_CREATION"
        ));
    }

    #[test]
    fn test_multiple_exceptions_pipe_separated() {
        let error = InvalidTransaction::LackOfFundForMaxFee {
            fee: Box::new(1000u64.into()),
            balance: Box::new(100u64.into()),
        };
        assert!(error_matches_exception(
            &error,
            "TransactionException.INSUFFICIENT_ACCOUNT_FUNDS|TransactionException.INTRINSIC_GAS_TOO_LOW"
        ));
    }

    #[test]
    fn test_error_to_exception_roundtrip() {
        let error = InvalidTransaction::EmptyBlobs;
        let exception = error_to_exception_string(&error);
        assert!(error_matches_exception(&error, exception));
    }
}
