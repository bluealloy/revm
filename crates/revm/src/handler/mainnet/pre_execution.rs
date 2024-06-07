//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    precompile::{PrecompileSpecId, Precompiles},
    primitives::{
        db::Database,
        Account, EVMError, Env, Spec,
        SpecId::{CANCUN, PRAGUE, SHANGHAI},
        TransactTo, BLOCKHASH_STORAGE_ADDRESS, U256,
    },
    Context, ContextPrecompiles,
};

/// Main precompile load
#[inline]
pub fn load_precompiles<SPEC: Spec, DB: Database>() -> ContextPrecompiles<DB> {
    Precompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID))
        .clone()
        .into()
}

/// Main load handle
#[inline]
pub fn load_accounts<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
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

    // Load blockhash storage address
    // EIP-2935: Serve historical block hashes from state
    if SPEC::enabled(PRAGUE) {
        context.evm.inner.journaled_state.initial_account_load(
            BLOCKHASH_STORAGE_ADDRESS,
            &[],
            &mut context.evm.inner.db,
        )?;
    }

    // Load code into EOAs
    // EIP-7702: Set EOA account code for one transaction
    if SPEC::enabled(PRAGUE) {
        // TODO(eip7702): This is currently UNTESTED and needs to be checked for correctness

        // These authorizations are fallible, so if we encounter an invalid authorization
        // or an error, skip it and continue
        for authorization in context.evm.inner.env.tx.authorization_list.iter() {
            // Recover the signer address if possible
            let Some(authority) = authorization.recovered_authority() else {
                continue;
            };

            // Optionally match the chain id
            if authorization.chain_id != 0
                && authorization.chain_id != context.evm.inner.env.cfg.chain_id
            {
                continue;
            }

            // Verify that the code of authority is empty
            // TODO(eip7702): better way to do this?
            let Ok(authority_account) = context.evm.inner.journaled_state.initial_account_load(
                authority,
                &[],
                &mut context.evm.inner.db,
            ) else {
                continue;
            };
            if authority_account.info.code.is_some() {
                continue;
            };

            // Check nonce if present
            if let Some(nonce) = authorization.nonce {
                if authority_account.info.nonce != nonce {
                    continue;
                }
            }

            // Load the code into the account
            // This touches the access list for `address` but not for the `from_address`
            // TODO(eip7702): better way to do this?
            _ = context.evm.inner.journaled_state.load_code_into(
                authority,
                authorization.address,
                &mut context.evm.inner.db,
            );
        }
    }

    context.evm.load_access_list()?;
    Ok(())
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

/// Deducts the caller balance to the transaction limit.
#[inline]
pub fn deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
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
