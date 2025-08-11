use crate::FrameResult;
use context_interface::{
    journaled_state::JournalTr,
    result::{ExecutionResult, HaltReasonTr},
    Block, Cfg, ContextTr, Database, Transaction,
};
use interpreter::{Gas, InitialAndFloorGas, SuccessOrHalt};
use primitives::{hardfork::SpecId, U256};

/// Ensures minimum gas floor is spent according to EIP-7623.
pub fn eip7623_check_gas_floor(gas: &mut Gas, init_and_floor_gas: InitialAndFloorGas) {
    // EIP-7623: Increase calldata cost
    // spend at least a gas_floor amount of gas.
    if gas.spent_sub_refunded() < init_and_floor_gas.floor_gas {
        gas.set_spent(init_and_floor_gas.floor_gas);
        // clear refund
        gas.set_refund(0);
    }
}

/// Calculates the final gas refund amount based on the specification and spent gas.
///
/// This applies EIP-3529 (Reduction in refunds) which limits the maximum refund to:
/// - 1/5 of gas spent (London and later)
/// - 1/2 of gas spent (before London)
pub fn calculate_final_refund(total_refund: i64, gas_spent: u64, spec: SpecId) -> i64 {
    let max_refund_quotient = if spec.is_enabled_in(SpecId::LONDON) {
        5
    } else {
        2
    };
    let final_refund = (total_refund as u64).min(gas_spent / max_refund_quotient) as i64;

    // Ensure refund is non-negative
    final_refund.max(0)
}

/// Calculates and applies gas refunds based on the specification.
pub fn refund<CTX: ContextTr>(
    context: &mut CTX,
    gas: &mut Gas,
    spec: SpecId,
    eip7702_refund: i64,
) -> i64 {
    // Add EIP-7702 refund to the journal and get total refund
    let journal = context.journal_mut();
    journal.record_refund(eip7702_refund);
    let total_refund = journal.refund();

    // Calculate final refund with EIP-3529 limits
    let final_refund = calculate_final_refund(total_refund, gas.spent(), spec);
    
    // Set the final refund back to the gas object for API compatibility
    gas.set_refund(final_refund);
    
    final_refund
}

/// Reimburses the caller for unused gas.
#[inline]
pub fn reimburse_caller<CTX: ContextTr>(
    context: &mut CTX,
    gas: &Gas,
    gas_refund: i64,
    additional_refund: U256,
) -> Result<(), <CTX::Db as Database>::Error> {
    let basefee = context.block().basefee() as u128;
    let caller = context.tx().caller();
    let effective_gas_price = context.tx().effective_gas_price(basefee);

    // Return balance of not spend gas.
    context.journal_mut().balance_incr(
        caller,
        U256::from(
            effective_gas_price.saturating_mul((gas.remaining() + gas_refund as u64) as u128),
        ) + additional_refund,
    )?;

    Ok(())
}

/// Rewards the beneficiary with transaction fees.
#[inline]
pub fn reward_beneficiary<CTX: ContextTr>(
    context: &mut CTX,
    gas: &Gas,
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
    context.journal_mut().balance_incr(
        beneficiary,
        U256::from(coinbase_gas_price * gas.used() as u128),
    )?;

    Ok(())
}

/// Calculate last gas spent and transform internal reason to external.
///
/// TODO make Journal FinalOutput more generic.
pub fn output<CTX: ContextTr<Journal: JournalTr>, HALTREASON: HaltReasonTr>(
    context: &mut CTX,
    // TODO, make this more generic and nice.
    // FrameResult should be a generic that returns gas and interpreter result.
    result: FrameResult,
) -> ExecutionResult<HALTREASON> {
    // Used gas with refund calculated.
    let gas_refunded = result.gas().refunded() as u64;
    let gas_used = result.gas().used();
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // take logs from journal.
    let logs = context.journal_mut().take_logs();

    match SuccessOrHalt::<HALTREASON>::from(instruction_result.result) {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas_used,
            gas_refunded,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas_used,
            output: output.into_data(),
        },
        SuccessOrHalt::Halt(reason) => ExecutionResult::Halt { reason, gas_used },
        // Only two internal return flags.
        flag @ (SuccessOrHalt::FatalExternalError | SuccessOrHalt::Internal(_)) => {
            panic!(
                "Encountered unexpected internal return flag: {flag:?} with instruction result: {instruction_result:?}"
            )
        }
    }
}
