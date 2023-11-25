use revm_interpreter::gas;

use crate::{
    interpreter::{Gas, InstructionResult, SuccessOrHalt},
    primitives::{db::Database, EVMError, ExecutionResult, Output, ResultAndState},
    primitives::{Env, Spec},
    Context,
};

/// Validate environment for the mainnet.
pub fn validate_env<SPEC: Spec, DB: Database>(env: &Env) -> Result<(), EVMError<DB::Error>> {
    // Important: validate block before tx.
    env.validate_block_env::<SPEC>()?;
    env.validate_tx::<SPEC>()?;
    Ok(())
}

pub fn initial_tx_gas<SPEC: Spec>(env: &Env) -> u64 {
    let input = &env.tx.data;
    let is_create = env.tx.transact_to.is_create();
    let access_list = &env.tx.access_list;

    gas::initial_tx_gas::<SPEC>(input, is_create, access_list)
}
