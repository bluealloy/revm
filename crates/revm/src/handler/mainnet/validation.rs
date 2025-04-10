use crate::{
    handler::SpecId,
    primitives::{db::Database, EVMError, Env, InvalidTransaction, Spec},
    Context,
};
use revm_interpreter::gas::{self, InitialAndFloorGas};

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
    let caller_account = context
        .evm
        .inner
        .journaled_state
        .load_code(tx_caller, &mut context.evm.inner.db)?;

    context
        .evm
        .inner
        .env
        .validate_tx_against_state::<SPEC>(caller_account.data)
        .map_err(EVMError::Transaction)?;

    Ok(())
}

/// Validate initial transaction gas.
pub fn validate_initial_tx_gas<SPEC: Spec, DB: Database>(
    env: &Env,
) -> Result<InitialAndFloorGas, EVMError<DB::Error>> {
    let input = &env.tx.data;
    let is_create = env.tx.transact_to.is_create();
    let access_list = &env.tx.access_list;
    let authorization_list_num = env
        .tx
        .authorization_list
        .as_ref()
        .map(|l| l.len() as u64)
        .unwrap_or_default();

    let gas = gas::calculate_initial_tx_gas(
        SPEC::SPEC_ID,
        input,
        is_create,
        access_list,
        authorization_list_num,
    );

    // Additional check to see if limit is big enough to cover initial gas.
    if gas.initial_gas > env.tx.gas_limit {
        return Err(InvalidTransaction::CallGasCostMoreThanGasLimit.into());
    }

    // EIP-7623
    if SPEC::SPEC_ID.is_enabled_in(SpecId::PRAGUE) && gas.floor_gas > env.tx.gas_limit {
        // return Err(InvalidTransaction::GasFloorMoreThanGasLimit.into());
    };

    Ok(gas)
}
