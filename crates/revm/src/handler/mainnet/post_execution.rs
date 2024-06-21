use crate::{
    interpreter::{Gas, SuccessOrHalt},
    primitives::{
        db::Database, EVMError, ExecutionResult, ResultAndState, Spec, SpecId::LONDON,
        KECCAK_EMPTY, U256,
    },
    Context, FrameResult,
};

/// Mainnet end handle does not change the output.
#[inline]
pub fn end<EXT, DB: Database>(
    _context: &mut Context<EXT, DB>,
    evm_output: Result<ResultAndState, EVMError<DB::Error>>,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    evm_output
}

/// Clear handle clears error and journal state.
#[inline]
pub fn clear<EXT, DB: Database>(context: &mut Context<EXT, DB>) {
    // clear error and journaled state.
    let _ = context.evm.take_error();
    context.evm.inner.journaled_state.clear();
    // Clear valid authorizations after each transaction.
    // If transaction is valid they are consumed in `output` handler.
    context.evm.inner.valid_authorizations.clear();
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<SPEC: Spec, EXT, DB: Database>(
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
        .inner
        .journaled_state
        .load_account(beneficiary, &mut context.evm.inner.db)?;

    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spent() - gas.refunded() as u64));

    Ok(())
}

#[inline]
pub fn reimburse_caller<SPEC: Spec, EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    gas: &Gas,
) -> Result<(), EVMError<DB::Error>> {
    let caller = context.evm.env.tx.caller;
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let (caller_account, _) = context
        .evm
        .inner
        .journaled_state
        .load_account(caller, &mut context.evm.inner.db)?;

    caller_account.info.balance = caller_account
        .info
        .balance
        .saturating_add(effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64));

    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn output<EXT, DB: Database>(
    context: &mut Context<EXT, DB>,
    result: FrameResult,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    context.evm.take_error()?;
    // used gas with refund calculated.
    let gas_refunded = result.gas().refunded() as u64;
    let final_gas_used = result.gas().spent() - gas_refunded;
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // reset journal and return present state.
    let (mut state, logs) = context.evm.journaled_state.finalize();

    // clear code of authorized accounts.
    for authorized in core::mem::take(&mut context.evm.inner.valid_authorizations).into_iter() {
        let account = state
            .get_mut(&authorized)
            .expect("Authorized account must exist");
        account.info.code = None;
        account.info.code_hash = KECCAK_EMPTY;
        account.storage.clear();
    }

    let result = match instruction_result.result.into() {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas_used: final_gas_used,
            gas_refunded,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas_used: final_gas_used,
            output: output.into_data(),
        },
        SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
            reason,
            gas_used: final_gas_used,
        },
        // Only two internal return flags.
        flag @ (SuccessOrHalt::FatalExternalError | SuccessOrHalt::Internal(_)) => {
            panic!(
                "Encountered unexpected internal return flag: {:?} with instruction result: {:?}",
                flag, instruction_result
            )
        }
    };

    Ok(ResultAndState { result, state })
}
