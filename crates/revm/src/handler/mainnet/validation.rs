use revm_interpreter::gas;

use crate::{
    primitives::{db::Database, EVMError, Env, InvalidTransaction, Spec},
    Context,
};

/// Validate environment for the mainnet.
pub fn validate_env<SPEC: Spec, DB: Database>(env: &Env) -> Result<(), EVMError<DB::Error>> {
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()?;
    Ok(())
}

/// Validates transaction against the state.
pub fn validate_tx_against_state<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load acc
    let tx_caller = context.evm.env.tx.caller;
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(tx_caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    context
        .evm
        .env
        .validate_tx_against_state::<SPEC>(caller_account)
        .map_err(EVMError::Transaction)?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<SPEC: Spec, DB: Database>(
    env: &Env,
) -> Result<u64, EVMError<DB::Error>> {
    let input = &env.tx.data;
    let is_create = env.tx.transact_to.is_create();
    let access_list = &env.tx.access_list;

    let initial_gas_spend = gas::validate_initial_tx_gas::<SPEC>(input, is_create, access_list);

    // Additional check to see if limit is big enough to cover initial gas.
    if initial_gas_spend > env.tx.gas_limit {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
    }
    Ok(initial_gas_spend)
}
