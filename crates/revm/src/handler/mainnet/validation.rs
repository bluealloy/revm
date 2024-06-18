use revm_interpreter::gas;

use crate::{
    primitives::{db::Database, ChainSpec, EVMError, Env, InvalidTransaction, Spec, Transaction},
    Context,
};

/// Validate environment for the mainnet.
pub fn validate_env<ChainSpecT: ChainSpec, SPEC: Spec, DB: Database>(
    env: &Env<ChainSpecT>,
) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()
        .map_err(|error| EVMError::Transaction(error.into()))?;
    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<ChainSpecT: ChainSpec, SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<ChainSpecT, EXT, DB>,
) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
    // load acc
    let tx_caller = context.evm.env.tx.caller();
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(*tx_caller, &mut context.evm.inner.db)
        .map_err(EVMError::Database)?;

    context
        .evm
        .inner
        .env
        .validate_tx_against_state::<SPEC>(caller_account)
        .map_err(|error| EVMError::Transaction(error.into()))?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<ChainSpecT: ChainSpec, SPEC: Spec, DB: Database>(
    env: &Env<ChainSpecT>,
) -> Result<u64, EVMError<ChainSpecT, DB::Error>> {
    let input = &env.tx.data();
    let is_create = env.tx.kind().is_create();
    let access_list = &env.tx.access_list();
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
