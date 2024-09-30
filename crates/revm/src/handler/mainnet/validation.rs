use core::cmp;

use crate::{Context, EvmWiring};
use interpreter::gas;
use primitives::{B256, U256};
use specification::{
    constantans::MAX_INITCODE_SIZE,
    eip4844,
    hardfork::{Spec, SpecId},
};
use transaction::{Eip1559CommonTxFields, Eip2930Tx, Eip4844Tx, Eip7702Tx, LegacyTx, Transaction};
use wiring::{
    default::{CfgEnv, EnvWiring},
    result::{EVMError, EVMResultGeneric, InvalidTransaction},
    Block, TransactionType,
};

/// Validate environment for the mainnet.
pub fn validate_env<EvmWiringT: EvmWiring, SPEC: Spec>(
    env: &EnvWiring<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    // Important: validate block before tx.
    validate_env_block::<EvmWiringT, SPEC>(&env.block, &env.cfg)?;
    validate_env_tx::<EvmWiringT, SPEC>(&env.tx, &env.block, &env.cfg)
        .map_err(|error| EVMError::Transaction(error.into()))?;
    Ok(())
}
pub fn validate_env_block<EvmWiringT: EvmWiring, SPEC: Spec>(
    block: &EvmWiringT::Block,
    cfg: &CfgEnv,
) -> EVMResultGeneric<(), EvmWiringT> {
    Ok(())
}

// pub fn validate_legacy_tx(
//     tx: &Eip7702Tx,
//     block: &Block,
//     cfg: &CfgEnv,
// ) -> Result<(), InvalidTransaction> {
//     Ok(())
// }

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

pub fn validate_env_tx<EvmWiringT: EvmWiring, SPEC: Spec>(
    tx: &EvmWiringT::Transaction,
    block: &EvmWiringT::Block,
    cfg: &CfgEnv,
) -> Result<(), InvalidTransaction> {
    // Check if the transaction's chain id is correct
    let common_field = tx.common_fields();
    let tx_type = tx.tx_type().into();

    let basefee = if cfg.is_base_fee_check_disabled() {
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
        }
        TransactionType::Eip2930 => {
            if !SPEC::enabled(SpecId::BERLIN) {
                return Err(InvalidTransaction::Eip2930NotSupported);
            }
            if cfg.chain_id != tx.eip2930().chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
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

            validate_priority_fee_tx(tx.max_fee_per_gas(), tx.max_priority_fee_per_gas(), basefee)?;
        }
        TransactionType::Eip4844 => {
            if !SPEC::enabled(SpecId::CANCUN) {
                return Err(InvalidTransaction::Eip4844NotSupported);
            }
            let tx = tx.eip4844();

            if cfg.chain_id != tx.chain_id() {
                return Err(InvalidTransaction::InvalidChainId);
            }

            validate_priority_fee_tx(tx.max_fee_per_gas(), tx.max_priority_fee_per_gas(), basefee)?;

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

            validate_priority_fee_tx(tx.max_fee_per_gas(), tx.max_priority_fee_per_gas(), basefee)?;

            let auth_list = tx.authorization_list();
            // The transaction is considered invalid if the length of authorization_list is zero.
            if auth_list.is_empty() {
                return Err(InvalidTransaction::EmptyAuthorizationList);
            }

            // Check validity of authorization_list
            auth_list.is_valid(cfg.chain_id)?;
        }
    }

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

/// Validates transaction against the state.
pub fn validate_tx_against_state<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as Transaction>::TransactionError: From<InvalidTransaction>,
{
    // load acc
    let tx_caller = context.evm.env.tx.common_fields().caller();
    let caller_account = context
        .evm
        .inner
        .journaled_state
        .load_code(tx_caller, &mut context.evm.inner.db)
        .map_err(EVMError::Database)?;

    context
        .evm
        .inner
        .env
        .validate_tx_against_state::<SPEC>(caller_account.data)
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
    let input = env.tx.common_fields().input();
    let is_create = env.tx.kind().is_create();
    let access_list = env.tx.access_list();
    let authorization_list_num = env
        .tx
        .authorization_list()
        .as_ref()
        .map(|l| l.len() as u64)
        .unwrap_or_default();

    let initial_gas_spend = gas::validate_initial_tx_gas(
        SPEC::SPEC_ID,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > env.tx.gas_limit() {
        return Err(EVMError::Transaction(
            InvalidTransaction::CallGasCostMoreThanGasLimit.into(),
        ));
    }
    Ok(initial_gas_spend)
}
