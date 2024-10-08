use crate::{Context, EvmWiring};
use interpreter::gas;
use specification::hardfork::Spec;
use wiring::{
    default::EnvWiring,
    result::{EVMError, EVMResultGeneric, InvalidTransaction},
    transaction::{Transaction, TransactionValidation},
};

/// Validate environment for the mainnet.
pub fn validate_env<EvmWiringT: EvmWiring, SPEC: Spec>(
    env: &EnvWiring<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
{
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()
        .map_err(|error| EVMError::Transaction(error.into()))?;
    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
) -> EVMResultGeneric<(), EvmWiringT>
where
    <EvmWiringT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
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
    <EvmWiringT::Transaction as TransactionValidation>::ValidationError: From<InvalidTransaction>,
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
