use crate::FrameResult;
use context::journaled_state::account::JournaledAccountTr;
use context_interface::{
    journaled_state::JournalTr,
    result::{ExecutionResult, HaltReason, HaltReasonTr, ResultGas},
    Block, Cfg, ContextTr, Database, LocalContextTr, Transaction,
};
use interpreter::{Gas, InitialAndFloorGas, SuccessOrHalt};
use primitives::{hardfork::SpecId, U256};

/// Builds a [`ResultGas`] from the execution [`Gas`] struct and [`InitialAndFloorGas`].
pub fn build_result_gas(
    gas: &Gas,
    refund: i64,
    init_and_floor_gas: InitialAndFloorGas,
) -> ResultGas {
    let state_gas = gas
        .state_gas_spent()
        .saturating_add(init_and_floor_gas.initial_state_gas)
        .saturating_sub(init_and_floor_gas.eip7702_reservoir_refund);

    ResultGas::default()
        .with_total_gas_spent(
            gas.limit()
                .saturating_sub(gas.remaining())
                .saturating_sub(gas.reservoir()),
        )
        .with_refunded(refund as u64)
        .with_floor_gas(init_and_floor_gas.floor_gas)
        .with_state_gas_spent(state_gas)
}

/// Ensures minimum gas floor is spent according to EIP-7623.
///
/// Per EIP-8037, gas used before refund is `tx.gas - gas_left - state_gas_reservoir`.
/// The floor applies to this combined total, not just regular gas.
pub fn eip7623_check_gas_floor(
    gas: &mut Gas,
    refund: &mut i64,
    init_and_floor_gas: InitialAndFloorGas,
) {
    // EIP-7623: Increase calldata cost
    // EIP-8037: tx_gas_used_before_refund = tx.gas - gas_left - reservoir
    // The floor must apply to this combined value, not just (limit - remaining).
    let gas_used_before_refund = gas.total_gas_spent().saturating_sub(gas.reservoir());
    let gas_used_after_refund = gas_used_before_refund.saturating_sub(*refund as u64);
    if gas_used_after_refund < init_and_floor_gas.floor_gas {
        // Set spent so that (limit - remaining - reservoir) = floor_gas
        // i.e. remaining = limit - floor_gas - reservoir
        gas.set_spent(init_and_floor_gas.floor_gas + gas.reservoir());
        // clear refund
        *refund = 0;
    }
}

/// Calculates final gas refunds based on the specification.
pub fn refund(spec: SpecId, gas: &Gas, journal_refund: i64, eip7702_refund: i64) -> i64 {
    let mut refund = journal_refund + eip7702_refund;
    // Calculate gas refund for transaction.
    // If spec is set to london, it will decrease the maximum refund amount to 5th part of
    // gas spend. (Before london it was 2th part of gas spend)
    let max_refund_quotient = if spec.is_enabled_in(SpecId::LONDON) { 5 } else { 2 };
    // EIP-8037: gas_used = total_gas_spent - reservoir (reservoir is unused state gas)
    let gas_used = gas.total_gas_spent().saturating_sub(gas.reservoir());
    refund = (refund as u64).min(gas_used / max_refund_quotient) as i64;
    refund
}

/// Reimburses the caller for unused gas.
#[inline]
pub fn reimburse_caller<CTX: ContextTr>(
    context: &mut CTX,
    gas: &Gas,
    refund: i64,
    additional_refund: U256,
) -> Result<(), <CTX::Db as Database>::Error> {
    // If fee charge was disabled (e.g. eth_call simulations), no gas was
    // deducted from the caller upfront so there is nothing to reimburse.
    if context.cfg().is_fee_charge_disabled() {
        return Ok(());
    }
    let basefee = context.block().basefee() as u128;
    let caller = context.tx().caller();
    let effective_gas_price = context.tx().effective_gas_price(basefee);

    // Return balance of not spent gas.
    // Include reservoir gas (EIP-8037) which is also unused and must be reimbursed.
    let reimbursable = gas.remaining() + gas.reservoir() + refund as u64;
    context
        .journal_mut()
        .load_account_mut(caller)?
        .incr_balance(
            U256::from(effective_gas_price.saturating_mul(reimbursable as u128))
                + additional_refund,
        );

    Ok(())
}

/// Rewards the beneficiary with transaction fees.
#[inline]
pub fn reward_beneficiary<CTX: ContextTr>(
    context: &mut CTX,
    gas: &Gas,
    refund: i64,
) -> Result<(), <CTX::Db as Database>::Error> {
    // If fee charge was disabled (e.g. eth_call simulations), the caller was
    // never charged for gas so there are no fees to transfer to the beneficiary.
    if context.cfg().is_fee_charge_disabled() {
        return Ok(());
    }
    let (block, tx, cfg, journal, _, _) = context.all_mut();
    let basefee = block.basefee() as u128;
    let effective_gas_price = tx.effective_gas_price(basefee);

    // Transfer fee to coinbase/beneficiary.
    // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
    let coinbase_gas_price = if cfg.spec().into().is_enabled_in(SpecId::LONDON) {
        effective_gas_price.saturating_sub(basefee)
    } else {
        effective_gas_price
    };

    // Reward beneficiary.
    // Exclude reservoir gas (EIP-8037) from the used gas — reservoir is unused and reimbursed.
    let gas_used = gas.total_gas_spent().saturating_sub(gas.reservoir());
    let effective_used = gas_used.saturating_sub(refund as u64);
    journal
        .load_account_mut(block.beneficiary())?
        .incr_balance(U256::from(coinbase_gas_price * effective_used as u128));

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
    result_gas: ResultGas,
) -> ExecutionResult<HALTREASON> {
    let output = result.output();
    let instruction_result = result.into_interpreter_result();

    // take logs from journal.
    let logs = context.journal_mut().take_logs();

    match SuccessOrHalt::<HALTREASON>::from(instruction_result.result) {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas: result_gas,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas: result_gas,
            logs,
            output: output.into_data(),
        },
        SuccessOrHalt::Halt(reason) => {
            // Bubble up precompile errors from context when available
            if matches!(
                instruction_result.result,
                interpreter::InstructionResult::PrecompileError
            ) {
                if let Some(message) = context.local_mut().take_precompile_error_context() {
                    return ExecutionResult::Halt {
                        reason: HALTREASON::from(HaltReason::PrecompileErrorWithContext(message)),
                        gas: result_gas,
                        logs,
                    };
                }
            }
            ExecutionResult::Halt {
                reason,
                gas: result_gas,
                logs,
            }
        }
        // Only two internal return flags.
        flag @ (SuccessOrHalt::FatalExternalError | SuccessOrHalt::Internal(_)) => {
            panic!(
                "Encountered unexpected internal return flag: {flag:?} with instruction result: {instruction_result:?}"
            )
        }
    }
}
