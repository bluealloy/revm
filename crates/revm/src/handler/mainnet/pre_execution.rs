//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    precompile::PrecompileSpecId,
    primitives::{
        db::Database, Account, Block, ChainSpec, EVMError, Env, Spec, SpecId, Transaction as _,
        BLOCKHASH_STORAGE_ADDRESS, U256,
    },
    Context, ContextPrecompiles,
};

/// Main precompile load
#[inline]
pub fn load_precompiles<ChainSpecT: ChainSpec, SPEC: Spec, DB: Database>(
) -> ContextPrecompiles<ChainSpecT, DB> {
    ContextPrecompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
}

/// Main load handle
#[inline]
pub fn load_accounts<ChainSpecT: ChainSpec, SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<ChainSpecT, EXT, DB>,
) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
    // set journaling state flag.
    context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

    // load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if SPEC::enabled(SpecId::SHANGHAI) {
        context
            .evm
            .inner
            .journaled_state
            .initial_account_load(
                *context.evm.inner.env.block.coinbase(),
                [],
                &mut context.evm.inner.db,
            )
            .map_err(EVMError::Database)?;
    }

    // Load blockhash storage address
    // EIP-2935: Serve historical block hashes from state
    if SPEC::enabled(SpecId::PRAGUE) {
        context
            .evm
            .inner
            .journaled_state
            .initial_account_load(BLOCKHASH_STORAGE_ADDRESS, [], &mut context.evm.inner.db)
            .map_err(EVMError::Database)?;
    }

    context.evm.load_access_list().map_err(EVMError::Database)?;
    Ok(())
}

/// Helper function that deducts the caller balance.
#[inline]
pub fn deduct_caller_inner<ChainSpecT: ChainSpec, SPEC: Spec>(
    caller_account: &mut Account,
    env: &Env<ChainSpecT>,
) {
    // Subtract gas costs from the caller's account.
    // We need to saturate the gas cost to prevent underflow in case that `disable_balance_check` is enabled.
    let mut gas_cost = U256::from(env.tx.gas_limit()).saturating_mul(env.effective_gas_price());

    // EIP-4844
    if SPEC::enabled(SpecId::CANCUN) {
        let data_fee = env.calc_data_fee().expect("already checked");
        gas_cost = gas_cost.saturating_add(data_fee);
    }

    // set new caller account balance.
    caller_account.info.balance = caller_account.info.balance.saturating_sub(gas_cost);

    // bump the nonce for calls. Nonce for CREATE will be bumped in `handle_create`.
    if env.tx.transact_to().is_call() {
        // Nonce is already checked
        caller_account.info.nonce = caller_account.info.nonce.saturating_add(1);
    }

    // touch account so we know it is changed.
    caller_account.mark_touch();
}

/// Deducts the caller balance to the transaction limit.
#[inline]
pub fn deduct_caller<ChainSpecT: ChainSpec, SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<ChainSpecT, EXT, DB>,
) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(
            *context.evm.inner.env.tx.caller(),
            &mut context.evm.inner.db,
        )
        .map_err(EVMError::Database)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<ChainSpecT, SPEC>(caller_account, &context.evm.inner.env);

    Ok(())
}
