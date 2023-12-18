//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use revm_precompile::{PrecompileSpecId, Precompiles};

use crate::{
    interpreter::{Gas, InstructionResult, SuccessOrHalt},
    primitives::{
        db::Database,
        Account, EVMError, Env, ExecutionResult, Output, ResultAndState, Spec,
        SpecId::{CANCUN, LONDON, SHANGHAI},
        TransactTo, U256,
    },
    Context,
};

/// Main precompile load
pub fn main_load_precompiles<SPEC: Spec>() -> Precompiles {
    Precompiles::new(PrecompileSpecId::from_spec_id(SPEC::SPEC_ID)).clone()
}

/// Main load handle
#[inline]
pub fn main_load<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // set journaling state flag.
    context.evm.journaled_state.set_spec_id(SPEC::SPEC_ID);

    // the L1-cost fee is only computed for Optimism non-deposit transactions.
    #[cfg(feature = "optimism")]
    if env.cfg.optimism && env.tx.optimism.source_hash.is_none() {
        let l1_block_info =
            optimism::L1BlockInfo::try_fetch(self.context.evm.db).map_err(EVMError::Database)?;

        // storage l1 block info for later use.
        self.context.evm.l1_block_info = Some(l1_block_info);

        tx_l1_cost
    }

    // load coinbase
    // EIP-3651: Warm COINBASE. Starts the `COINBASE` address warm
    if SPEC::enabled(SHANGHAI) {
        context
            .evm
            .journaled_state
            .initial_account_load(context.evm.env.block.coinbase, &[], &mut context.evm.db)
            .map_err(EVMError::Database)?;
    }

    context.evm.load_access_list()?;
    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn main_return<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    call_result: InstructionResult,
    output: Output,
    gas: &Gas,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    // used gas with refund calculated.
    let gas_refunded = gas.refunded() as u64;
    let final_gas_used = gas.spend() - gas_refunded;

    // reset journal and return present state.
    let (state, logs) = context.evm.journaled_state.finalize();

    let result = match call_result.into() {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas_used: final_gas_used,
            gas_refunded,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas_used: final_gas_used,
            output: match output {
                Output::Call(return_value) => return_value,
                Output::Create(return_value, _) => return_value,
            },
        },
        SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
            reason,
            gas_used: final_gas_used,
        },
        SuccessOrHalt::FatalExternalError => {
            return Err(EVMError::Database(context.evm.error.take().unwrap()));
        }
        // Only two internal return flags.
        SuccessOrHalt::InternalContinue | SuccessOrHalt::InternalCallOrCreate => {
            panic!("Internal return flags should remain internal {call_result:?}")
        }
    };

    Ok(ResultAndState { result, state })
}

/// Mainnet end handle does not change the output.
#[inline]
pub fn main_end<EXT, DB: Database>(
    _context: &mut Context<EXT, DB>,
    evm_output: Result<ResultAndState, EVMError<DB::Error>>,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    evm_output
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn main_reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let beneficiary = context.evm.env.block.coinbase;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if SPEC::enabled(LONDON) {
        effective_gas_price.saturating_sub(context.evm.env.block.basefee)
    } else {
        effective_gas_price
    };

    let (coinbase_account, _) = context
        .evm
        .journaled_state
        .load_account(beneficiary, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spend() - gas.refunded() as u64));

    Ok(())
}

/// Helper function that deducts the caller balance.
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

#[inline]
pub fn main_deduct_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
) -> Result<(), EVMError<DB::Error>> {
    // load caller's account.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(context.evm.env.tx.caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    // deduct gas cost from caller's account.
    deduct_caller_inner::<SPEC>(caller_account, &context.evm.env);

    Ok(())
}

#[inline]
pub fn main_reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let caller = context.evm.env.tx.caller;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) = context
        .evm
        .journaled_state
        .load_account(caller, &mut context.evm.db)
        .map_err(EVMError::Database)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64));

    Ok(())
}
