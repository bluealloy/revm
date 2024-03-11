//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    handler::{DeductCallerTrait, LoadAccountsTrait, LoadPrecompilesTrait},
    precompile::{PrecompileSpecId, Precompiles},
    primitives::{
        db::Database,
        Account, EVMError, Env, Spec,
        SpecId::{CANCUN, SHANGHAI},
        TransactTo, U256,
    },
    Context, ContextPrecompiles,
};

/// PreExecutionImpl implements all traits related to post execution handles.
#[derive(Clone, Debug)]
pub struct PreExecutionImpl<SPEC> {
    pub _spec: std::marker::PhantomData<SPEC>,
}

impl<SPEC: Spec> Default for PreExecutionImpl<SPEC> {
    fn default() -> Self {
        Self {
            _spec: std::marker::PhantomData,
        }
    }
}

impl<SPEC: Spec, DB: Database> LoadPrecompilesTrait<DB> for PreExecutionImpl<SPEC> {
    #[inline]
    fn load_precompiles(&self) -> ContextPrecompiles<DB> {
        Precompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
            .clone()
            .into()
    }
}

impl<SPEC: Spec, EXT, DB: Database> LoadAccountsTrait<EXT, DB> for PreExecutionImpl<SPEC> {
    #[inline]
    fn load_accounts(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        // set journaling state flag.
        context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

        // load coinbase
        // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
        if SPEC::enabled(SHANGHAI) {
            context.evm.inner.journaled_state.initial_account_load(
                context.evm.inner.env.block.coinbase,
                &[],
                &mut context.evm.inner.db,
            )?;
        }

        context.evm.load_access_list()?;
        Ok(())
    }
}

impl<SPEC: Spec, EXT, DB: Database> DeductCallerTrait<EXT, DB> for PreExecutionImpl<SPEC> {
    fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        // load caller's account.
        let (caller_account, _) = context
            .evm
            .inner
            .journaled_state
            .load_account(context.evm.inner.env.tx.caller, &mut context.evm.inner.db)?;

        // deduct gas cost from caller's account.
        deduct_caller_inner::<SPEC>(caller_account, &context.evm.inner.env);

        Ok(())
    }
}

/// Helper function that deducts the caller balance.
#[inline]
pub fn deduct_caller_inner<SPEC: Spec>(caller_account: &mut Account, env: &Env) {
    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if matches!(env.tx.transact_to, TransactTo::Call(_)) {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();
}
