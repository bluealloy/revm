use crate::{Context, EvmWiring, FrameResult};
use interpreter::{Gas, SuccessOrHalt};
use primitives::U256;
use specification::hardfork::{Spec, SpecId};
use wiring::{
    result::{EVMError, EVMResult, EVMResultGeneric, ExecutionResult, ResultAndState},
    Block, Transaction,
};

/// Mainnet end handle does not change the output.
#[inline]
pub fn end<EvmWiringT: EvmWiring>(
    _context: &mut Context<EvmWiringT>,
    evm_output: EVMResult<EvmWiringT>,
) -> EVMResult<EvmWiringT> {
    evm_output
}

/// Clear handle clears error and journal state.
#[inline]
pub fn clear<EvmWiringT: EvmWiring>(context: &mut Context<EvmWiringT>) {
    // clear error and journaled state.
    let _ = context.evm.take_error();
    context.evm.inner.journaled_state.clear();
}

/// Reward beneficiary with gas fee.
#[inline]
pub fn reward_beneficiary<EvmWiringT: EvmWiring, SPEC: Spec>(
    context: &mut Context<EvmWiringT>,
    gas: &Gas,
) -> EVMResultGeneric<(), EvmWiringT> {
    let beneficiary = *context.evm.env.block.coinbase();
    let effective_gas_price = context.evm.env.effective_gas_price();

    // transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if SPEC::enabled(SpecId::LONDON) {
        effective_gas_price.saturating_sub(*context.evm.env.block.basefee())
    } else {
        effective_gas_price
    };

    let coinbase_account = context
        .evm
        .inner
        .journaled_state
        .load_account(beneficiary, &mut context.evm.inner.db)
        .map_err(EVMError::Database)?;

    coinbase_account.data.mark_touch();
    coinbase_account.data.info.balance = coinbase_account
        .data
        .info
        .balance
        .saturating_add(coinbase_gas_price * U256::from(gas.spent() - gas.refunded() as u64));

    Ok(())
}

pub fn refund<EvmWiringT: EvmWiring, SPEC: Spec>(
    _context: &mut Context<EvmWiringT>,
    gas: &mut Gas,
    eip7702_refund: i64,
) {
    gas.record_refund(eip7702_refund);

    // Calculate gas refund for transaction.
    // If spec is set to london, it will decrease the maximum refund amount to 5th part of
    // gas spend. (Before london it was 2th part of gas spend)
    gas.set_final_refund(SPEC::SPEC_ID.is_enabled_in(SpecId::LONDON));
}

#[inline]
pub fn reimburse_caller<EvmWiringT: EvmWiring>(
    context: &mut Context<EvmWiringT>,
    gas: &Gas,
) -> EVMResultGeneric<(), EvmWiringT> {
    let caller = context.evm.env.tx.common_fields().caller();
    let effective_gas_price = context.evm.env.effective_gas_price();

    // return balance of not spend gas.
    let caller_account = context
        .evm
        .inner
        .journaled_state
        .load_account(caller, &mut context.evm.inner.db)
        .map_err(EVMError::Database)?;

    caller_account.data.info.balance =
        caller_account.data.info.balance.saturating_add(
            effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64),
        );

    Ok(())
}

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn output<EvmWiringT: EvmWiring>(
    context: &mut Context<EvmWiringT>,
    result: FrameResult,
) -> EVMResult<EvmWiringT> {
    context.evm.take_error().map_err(EVMError::Database)?;

    // used gas with refund calculated.
    let gas_refunded = result.gas().refunded() as u64;
    let final_gas_used = result.gas().spent() - gas_refunded;
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // reset journal and return present state.
    let (state, logs) = context.evm.journaled_state.finalize();

    let result = match SuccessOrHalt::<EvmWiringT::HaltReason>::from(instruction_result.result) {
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
