use core::cmp::{self, Ordering};

use crate::{Context, EvmWiring};
use interpreter::gas;
use primitives::{B256, U256};
use specification::{
    constants::MAX_INITCODE_SIZE,
    eip4844,
    hardfork::{Spec, SpecId},
};
use state::Account;
use std::boxed::Box;
use transaction::{
    eip7702::Authorization, Eip1559CommonTxFields, Eip2930Tx, Eip4844Tx, Eip7702Tx, LegacyTx,
    Transaction,
};
use wiring::{
    default::{CfgEnv, EnvWiring},
    result::{EVMError, EVMResultGeneric, InvalidHeader, InvalidTransaction},
    Block, TransactionType,
};

/// Validate environment (block and transaction) for the mainnet.
pub fn validate_env<EvmWiringT: EvmWiring, SPEC: Spec>(
    env: &EnvWiring<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    // Important: validate block before tx as some field are used in transaction validation.
    validate_block_env::<EvmWiringT, SPEC>(&env.block).map_err(EVMError::Header)?;

    // validate transaction.
    validate_tx_env::<EvmWiringT, SPEC>(&env.tx, &env.block, &env.cfg)
        .map_err(|e| EVMError::Transaction(e.into()))?;
    Ok(())
}

/// Validate the block environment.
#[inline]
pub fn validate_block_env<EvmWiringT: EvmWiring, SPEC: Spec>(
    block: &EvmWiringT::Block,
) -> Result<(), InvalidHeader> {
    // `prevrandao` is required for the merge
    if SPEC::enabled(SpecId::MERGE) && block.prevrandao().is_none() {
        return Err(InvalidHeader::PrevrandaoNotSet);
    }
    // `excess_blob_gas` is required for Cancun
    if SPEC::enabled(SpecId::CANCUN) && block.blob_excess_gas_and_price().is_none() {
        return Err(InvalidHeader::ExcessBlobGasNotSet);
    }
    Ok(())
}

/// Validate transaction that has EIP-1559 priority fee
pub fn validate_priority_fee_tx(
    max_fee: u128,
    max_priority_fee: u128,
    base_fee: Option<U256>,
) -> Result<(), InvalidTransaction> {
    if max_priority_fee > max_fee {
        // or gas_max_fee for eip1559
        return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
    }

    // check minimal cost against basefee
    if let Some(base_fee) = base_fee {
        let effective_gas_price = cmp::min(
            U256::from(max_fee),
            base_fee.saturating_add(U256::from(max_priority_fee)),
        );
        if effective_gas_price < base_fee {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }

    Ok(())
}

/// Validate EIP-4844 transaction.
pub fn validate_eip4844_tx(
    blobs: &[B256],
    max_blob_fee: u128,
    block_blob_gas_price: u128,
) -> Result<(), InvalidTransaction> {
    // ensure that the user was willing to at least pay the current blob gasprice
    if block_blob_gas_price > max_blob_fee {
        return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
    }

    // there must be at least one blob
    if blobs.is_empty() {
        return Err(InvalidTransaction::EmptyBlobs);
    }

    // all versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
    for blob in blobs {
        if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
            return Err(InvalidTransaction::BlobVersionNotSupported);
        }
    }

    // ensure the total blob gas spent is at most equal to the limit
    // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
    if blobs.len() > eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize {
        return Err(InvalidTransaction::TooManyBlobs {
            have: blobs.len(),
            max: eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize,
        });
    }
    Ok(())
}

/// Validate transaction against block and configuration for mainnet.
pub fn validate_tx_env<EvmWiringT: EvmWiring, SPEC: Spec>(
    tx: &EvmWiringT::Transaction,
    block: &EvmWiringT::Block,
    cfg: &CfgEnv,
) -> Result<(), InvalidTransaction> {
    // Check if the transaction's chain id is correct
    let common_field = tx.common_fields();
    let tx_type = tx.tx_type().into();

    let base_fee = if cfg.is_base_fee_check_disabled() {
        None
    } else {
        Some(*block.basefee())
    };

    match tx_type {
        TransactionType::Legacy => {
            let tx = tx.legacy();
            // check chain_id only if it is present in the legacy transaction.
            // EIP-155: Simple replay attack protection
            if let Some(chain_id) = tx.chain_id() {
                if chain_id != cfg.chain_id {
                    return Err(InvalidTransaction::InvalidChainId);
                }
            }
            // gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if U256::from(tx.gas_price()) < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee);
                }
            }
        }
        TransactionType::Eip2930 => {
            // enabled in BERLIN hardfork
            if !SPEC::enabled(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported);
            }
            let tx = tx.eip2930();

            if cfg.chain_id != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            // gas price must be at least the basefee.
            if let Some(base_fee) = base_fee {
                if U256::from(tx.gas_price()) < base_fee {
                    return Err(InvalidTransaction::GasPriceLessThanBasefee);
                }
            }
        }
        TransactionType::Eip1559 => {
            if !SPEC::enabled(SpecId::LONDON) {
                return Err(InvalidTransaction::Eip1559NotSupported);
            }
            let tx = tx.eip1559();

            if cfg.chain_id != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;
        }
        TransactionType::Eip4844 => {
            if !SPEC::enabled(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported);
            }
            let tx = tx.eip4844();

            if cfg.chain_id != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;

            validate_eip4844_tx(
                tx.blob_versioned_hashes(),
                tx.max_fee_per_blob_gas(),
                block.blob_gasprice().unwrap_or_default(),
            )?;
        }
        TransactionType::Eip7702 => {
            // check if EIP-7702 transaction is enabled.
            if !SPEC::enabled(SpecId::PRAGUE) {
                return Err(InvalidTransaction::Eip7702NotSupported);
            }
            let tx = tx.eip7702();

            if cfg.chain_id != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(
                tx.max_fee_per_gas(),
                tx.max_priority_fee_per_gas(),
                base_fee,
            )?;

            let auth_list_len = tx.authorization_list_len();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list_len == 0 {
                return Err(InvalidTransaction::EmptyAuthorizationList);
            }

            // TODO temporary here as newest EIP have removed this check.
            for auth in tx.authorization_list_iter() {
                if auth.is_invalid() {
                    return Err(InvalidTransaction::Eip7702NotSupported);
                }
            }
        }
        TransactionType::Custom => {
            // custom transaction type check is not done here.
        }
    };

    // Check if gas_limit is more than block_gas_limit
    if !cfg.is_block_gas_limit_disabled()
        && U256::from(common_field.gas_limit()) > *block.gas_limit()
    {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
    }

    // EIP-3860: Limit and meter initcode
    if SPEC::enabled(SpecId::SHANGHAI) && tx.kind().is_create() {
        let max_initcode_size = cfg
            .limit_contract_code_size
            .map(|limit| limit.saturating_mul(2))
            .unwrap_or(MAX_INITCODE_SIZE);
        if tx.common_fields().input().len() > max_initcode_size {
            return Err(InvalidTransaction::CreateInitCodeSizeLimit);
        }
    }

    Ok(())
}

/// Validate account against the transaction.
pub fn validate_tx_against_account<EvmWiringT: EvmWiring, SPEC: Spec>(
    account: &mut Account,
    tx: &EvmWiringT::Transaction,
    cfg: &CfgEnv,
) -> Result<(), InvalidTransaction>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    let tx_type = tx.tx_type().into();
    // EIP-3607: Reject transactions from senders with deployed code
    // This EIP is introduced after london but there was no collision in past
    // so we can leave it enabled always
    if !cfg.is_eip3607_disabled() {
        let bytecode = &account.info.code.as_ref().unwrap();
        // allow EOAs whose code is a valid delegation designation,
        // i.e. 0xef0100 || address, to continue to originate transactions.
        if !bytecode.is_empty() && !bytecode.is_eip7702() {
            return Err(InvalidTransaction::RejectCallerWithCode);
        }
    }

    // Check that the transaction's nonce is correct
    if !cfg.is_nonce_check_disabled() {
        let tx = tx.common_fields().nonce();
        let state = account.info.nonce;
        match tx.cmp(&state) {
            Ordering::Greater => {
                return Err(InvalidTransaction::NonceTooHigh { tx, state });
            }
            Ordering::Less => {
                return Err(InvalidTransaction::NonceTooLow { tx, state });
            }
            _ => {}
        }
    }

    // gas_limit * max_fee + value
    let mut balance_check = U256::from(tx.common_fields().gas_limit())
        .checked_mul(U256::from(tx.max_fee()))
        .and_then(|gas_cost| gas_cost.checked_add(tx.common_fields().value()))
        .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

    if tx_type == TransactionType::Eip4844 {
        let tx = tx.eip4844();
        // if the tx is not a blob tx, this will be None, so we add zero
        let data_fee = tx.calc_max_data_fee();
        balance_check = balance_check
            .checked_add(U256::from(data_fee))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;
    }

    // Check if account has enough balance for `gas_limit * max_fee`` and value transfer.
    // Transfer will be done inside `*_inner` functions.
    if balance_check > account.info.balance {
        if cfg.is_balance_check_disabled() {
            // Add transaction cost to balance to ensure execution doesn't fail.
            account.info.balance = account.info.balance.saturating_add(balance_check);
        } else {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: Box::new(balance_check),
                balance: Box::new(account.info.balance),
            });
        }
    }

    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    let tx_caller = context.evm.env.tx.common_fields().caller();
    // load acc

    let inner = &mut context.evm.inner;

    let caller_account = inner
        .journaled_state
        .load_code(tx_caller, &mut inner.db)
        .map_err(EVMError::Database)?;

    validate_tx_against_account::<EvmWiringT, SPEC>(
        caller_account.data,
        &inner.env.tx,
        &inner.env.cfg,
    )
    .map_err(|e| EVMError::Transaction(e.into()))?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<EvmWiringT: EvmWiring, SPEC: Spec>(
    env: &EnvWiring<EvmWiringT>,
) -> EVMResultGeneric<u64, EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    let tx_type = env.tx.tx_type().into();

    let authorization_list_num = if tx_type == TransactionType::Eip7702 {
        env.tx.eip7702().authorization_list_len() as u64
    } else {
        0
    };

    let common_fields = env.tx.common_fields();
    let is_create = env.tx.kind().is_create();
    let input = common_fields.input();
    let access_list = env.tx.access_list();

    let initial_gas_spend = gas::validate_initial_tx_gas(
        SPEC::SPEC_ID,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > common_fields.gas_limit() {
        return Err(EVMError::Transaction(
            InvalidTransaction::CallGasCostMoreThanGasLimit.into(),
        ));
    }
    Ok(initial_gas_spend)
}
