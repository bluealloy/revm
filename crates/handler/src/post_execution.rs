use super::frame_data::FrameResult;
use context_interface::ContextTr;
use context_interface::{
    journaled_state::JournalTr,
    result::{ExecutionResult, HaltReasonTr},
    Block, Cfg, Database, Transaction,
};
use interpreter::{Gas, InitialAndFloorGas, SuccessOrHalt};
use primitives::{hardfork::SpecId, U256};
use state::EvmState;

pub fn eip7623_check_gas_floor(gas: &mut Gas, init_and_floor_gas: InitialAndFloorGas) {
    // EIP-7623: Increase calldata cost
    // spend at least a gas_floor amount of gas.
    if gas.spent_sub_refunded() < init_and_floor_gas.floor_gas {
        gas.set_spent(init_and_floor_gas.floor_gas);
        // clear refund
        gas.set_refund(0);
    }
}

pub fn refund(spec: SpecId, gas: &mut Gas, eip7702_refund: i64) {
    gas.record_refund(eip7702_refund);
    // Calculate gas refund for transaction.
    // If spec is set to london, it will decrease the maximum refund amount to 5th part of
    // gas spend. (Before london it was 2th part of gas spend)
    gas.set_final_refund(spec.is_enabled_in(SpecId::LONDON));
}

#[inline]
pub fn reimburse_caller<CTX: ContextTr>(
    context: &mut CTX,
    gas: &mut Gas,
    additional_refund: U256,
) -> Result<(), <CTX::Db as Database>::Error> {
    let basefee = context.block().basefee() as u128;
    let caller = context.tx().caller();
    let effective_gas_price = context.tx().effective_gas_price(basefee);

    // Return balance of not spend gas.
    context.journal().balance_incr(
        caller,
        U256::from(
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128),
        ) + additional_refund,
    )?;

    Ok(())
}

#[inline]
pub fn reward_beneficiary<CTX: ContextTr>(
    context: &mut CTX,
    gas: &mut Gas,
) -> Result<(), <CTX::Db as Database>::Error> {
    let beneficiary = context.block().beneficiary();
    let basefee = context.block().basefee() as u128;
    let effective_gas_price = context.tx().effective_gas_price(basefee);

    // Transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if context.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
        effective_gas_price.saturating_sub(basefee)
    } else {
        effective_gas_price
    };

    // reward beneficiary
    context.journal().balance_incr(
        beneficiary,
        U256::from(coinbase_gas_price * (gas.spent() - gas.refunded() as u64) as u128),
    )?;

    Ok(())
}

/// Calculate last gas spent and transform internal reason to external.
///
/// TODO make Journal FinalOutput more generic.
pub fn output<CTX: ContextTr<Journal: JournalTr<State = EvmState>>, HALTREASON: HaltReasonTr>(
    context: &mut CTX,
    // TODO, make this more generic and nice.
    // FrameResult should be a generic that returns gas and interpreter result.
    result: FrameResult,
) -> ExecutionResult<HALTREASON> {
    // Used gas with refund calculated.
    let gas_refunded = result.gas().refunded() as u64;
    let final_gas_used = result.gas().spent() - gas_refunded;
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // take logs from journal.
    let logs = context.journal().take_logs();

    match SuccessOrHalt::<HALTREASON>::from(instruction_result.result) {
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
    }
}
