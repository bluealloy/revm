use crate::{Context, EvmWiring};
use interpreter::gas;
use primitives::U256;
use specification::{
    constantans::MAX_INITCODE_SIZE,
    eip4844,
    hardfork::{Spec, SpecId},
};
use transaction::Transaction;
use wiring::{
    default::{CfgEnv, EnvWiring},
    result::{EVMError, EVMResultGeneric, InvalidTransaction},
    Block,
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

pub fn validate_env_tx<EvmWiringT: EvmWiring, SPEC: Spec>(
    tx: &EvmWiringT::Transaction,
    block: &EvmWiringT::Block,
    cfg: &CfgEnv,
) -> Result<(), InvalidTransaction> {
    // Check if the transaction's chain id is correct
    if let Some(tx_chain_id) = tx.chain_id() {
        if tx_chain_id != cfg.chain_id {
            return Err(InvalidTransaction::InvalidChainId);
        }
    }

    // Check if gas_limit is more than block_gas_limit
    if !cfg.is_block_gas_limit_disabled() && U256::from(tx.gas_limit()) > *block.gas_limit() {
        return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
    }

    // Check that access list is empty for transactions before BERLIN
    if !SPEC::enabled(SpecId::BERLIN) && !tx.access_list().is_empty() {
        return Err(InvalidTransaction::AccessListNotSupported);
    }

    // BASEFEE tx check
    if SPEC::enabled(SpecId::LONDON) {
        if let Some(priority_fee) = tx.max_priority_fee_per_gas() {
            if priority_fee > tx.gas_price() {
                // or gas_max_fee for eip1559
                return Err(InvalidTransaction::PriorityFeeGreaterThanMaxFee);
            }
        }

        // check minimal cost against basefee
        let base_fee = *block.basefee();
        if !cfg.is_base_fee_check_disabled() && tx.effective_gas_price(base_fee) < *block.basefee()
        {
            return Err(InvalidTransaction::GasPriceLessThanBasefee);
        }
    }

    // EIP-3860: Limit and meter initcode
    if SPEC::enabled(SpecId::SHANGHAI) && tx.kind().is_create() {
        let max_initcode_size = cfg
            .limit_contract_code_size
            .map(|limit| limit.saturating_mul(2))
            .unwrap_or(MAX_INITCODE_SIZE);
        if tx.data().len() > max_initcode_size {
            return Err(InvalidTransaction::CreateInitCodeSizeLimit);
        }
    }

    // - For before CANCUN, check that `blob_hashes` and `max_fee_per_blob_gas` are empty / not set
    if !SPEC::enabled(SpecId::CANCUN)
        && (tx.max_fee_per_blob_gas().is_some() || !tx.blob_hashes().is_empty())
    {
        return Err(InvalidTransaction::BlobVersionedHashesNotSupported);
    }

    // Presence of max_fee_per_blob_gas means that this is blob transaction.
    if let Some(max) = tx.max_fee_per_blob_gas() {
        // ensure that the user was willing to at least pay the current blob gasprice
        let price = block.get_blob_gasprice().expect("already checked");
        if U256::from(*price) > *max {
            return Err(InvalidTransaction::BlobGasPriceGreaterThanMax);
        }

        // there must be at least one blob
        if tx.blob_hashes().is_empty() {
            return Err(InvalidTransaction::EmptyBlobs);
        }

        // The field `to` deviates slightly from the semantics with the exception
        // that it MUST NOT be nil and therefore must always represent
        // a 20-byte address. This means that blob transactions cannot
        // have the form of a create transaction.
        if tx.kind().is_create() {
            return Err(InvalidTransaction::BlobCreateTransaction);
        }

        // all versioned blob hashes must start with VERSIONED_HASH_VERSION_KZG
        for blob in tx.blob_hashes() {
            if blob[0] != eip4844::VERSIONED_HASH_VERSION_KZG {
                return Err(InvalidTransaction::BlobVersionNotSupported);
            }
        }

        // ensure the total blob gas spent is at most equal to the limit
        // assert blob_gas_used <= MAX_BLOB_GAS_PER_BLOCK
        let num_blobs = tx.blob_hashes().len();
        if num_blobs > eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize {
            return Err(InvalidTransaction::TooManyBlobs {
                have: num_blobs,
                max: eip4844::MAX_BLOB_NUMBER_PER_BLOCK as usize,
            });
        }
    } else {
        // if max_fee_per_blob_gas is not set, then blob_hashes must be empty
        if !tx.blob_hashes().is_empty() {
            return Err(InvalidTransaction::BlobVersionedHashesNotSupported);
        }
    }

    // check if EIP-7702 transaction is enabled.
    if !SPEC::enabled(SpecId::PRAGUE) && tx.authorization_list().is_some() {
        return Err(InvalidTransaction::AuthorizationListNotSupported);
    }

    if let Some(auth_list) = &tx.authorization_list() {
        // The transaction is considered invalid if the length of authorization_list is zero.
        if auth_list.is_empty() {
            return Err(InvalidTransaction::EmptyAuthorizationList);
        }

        // Check validity of authorization_list
        auth_list.is_valid(cfg.chain_id)?;

        // Check if other fields are unset.
        if tx.max_fee_per_blob_gas().is_some() || !tx.blob_hashes().is_empty() {
            return Err(InvalidTransaction::AuthorizationListInvalidFields);
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
    let tx_caller = *context.evm.env.tx.caller();
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
    let input = &env.tx.data();
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
