use revm_interpreter::gas;

use crate::{
    handler::{
        validation::ValidateEnvTrait, ValidateInitialTxGasTrait, ValidateTxAgainstStateTrait,
    },
    primitives::{db::Database, EVMError, Env, InvalidTransaction, Spec},
    Context,
};

#[derive(Clone, Debug)]
pub struct ValidationImpl<SPEC> {
    pub _spec: std::marker::PhantomData<SPEC>,
}

impl<SPEC: Spec> Default for ValidationImpl<SPEC> {
    fn default() -> Self {
        Self {
            _spec: std::marker::PhantomData,
        }
    }
}

impl<DB: Database, SPEC: Spec> ValidateEnvTrait<DB> for ValidationImpl<SPEC> {
    fn validate_env(&self, env: &Env) -> Result<(), EVMError<DB::Error>> {
        // Important: validate block before tx.
        env.validate_block_env::<SPEC>()?;
        env.validate_tx::<SPEC>()?;
        Ok(())
    }
}

impl<EXT, DB: Database, SPEC: Spec> ValidateTxAgainstStateTrait<EXT, DB> for ValidationImpl<SPEC> {
    fn validate_tx_against_state(
        &self,
        context: &mut Context<EXT, DB>,
    ) -> Result<(), EVMError<DB::Error>> {
        // load acc
        let tx_caller = context.evm.env.tx.caller;
        let (caller_account, _) = context
            .evm
            .inner
            .journaled_state
            .load_account(tx_caller, &mut context.evm.inner.db)?;

        context
            .evm
            .inner
            .env
            .validate_tx_against_state::<SPEC>(caller_account)
            .map_err(EVMError::Transaction)?;

        Ok(())
    }
}

impl<DB: Database, SPEC: Spec> ValidateInitialTxGasTrait<DB> for ValidationImpl<SPEC> {
    fn validate_initial_tx_gas(&self, env: &Env) -> Result<u64, EVMError<DB::Error>> {
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
}
