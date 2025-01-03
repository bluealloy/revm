use context_interface::{
    journaled_state::Journal,
    result::{ExecutionResult, HaltReasonTrait, ResultAndState},
    Block, BlockGetter, Cfg, CfgGetter, ErrorGetter, JournalDBError, JournalGetter, Transaction,
    TransactionGetter,
};
use handler_interface::PostExecutionHandler;
use interpreter::SuccessOrHalt;
use primitives::{Log, U256};
use specification::hardfork::SpecId;
use state::EvmState;
use std::{boxed::Box, vec::Vec};

use super::frame_data::FrameResult;

#[derive(Default)]
pub struct EthPostExecution<CTX, ERROR, HALTREASON> {
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, HALTREASON)>,
}

impl<CTX, ERROR, HALTREASON> EthPostExecution<CTX, ERROR, HALTREASON> {
    /// Create new instance of post execution handler.
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create new boxed instance of post execution handler.
    ///
    /// Boxed instance is useful to erase FORK type.
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl<CTX, ERROR, HALTREASON> PostExecutionHandler for EthPostExecution<CTX, ERROR, HALTREASON>
where
    CTX: EthPostExecutionContext,
    ERROR: EthPostExecutionError<CTX>,
    HALTREASON: HaltReasonTrait,
{
    type Context = CTX;
    type Error = ERROR;
    type ExecResult = FrameResult;
    type Output = ResultAndState<HALTREASON>;

    fn refund(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
        eip7702_refund: i64,
    ) {
        let gas = exec_result.gas_mut();
        gas.record_refund(eip7702_refund);

        // Calculate gas refund for transaction.
        // If spec is set to london, it will decrease the maximum refund amount to 5th part of
        // gas spend. (Before london it was 2th part of gas spend)
        gas.set_final_refund(context.cfg().spec().into().is_enabled_in(SpecId::LONDON));
    }

    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        // Return balance of not spend gas.
        let caller_account = context.journal().load_account(caller)?;

        let reimbursed =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);
        caller_account.data.info.balance = caller_account
            .data
            .info
            .balance
            .saturating_add(U256::from(reimbursed));

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let block = context.block();
        let tx = context.tx();
        let beneficiary = block.beneficiary();
        let basefee = block.basefee() as u128;
        let effective_gas_price = tx.effective_gas_price(basefee);
        let gas = exec_result.gas();

        // Transfer fee to coinbase/beneficiary.
        // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
        let coinbase_gas_price = if context.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let coinbase_account = context.journal().load_account(beneficiary)?;

        coinbase_account.data.mark_touch();
        coinbase_account.data.info.balance =
            coinbase_account
                .data
                .info
                .balance
                .saturating_add(U256::from(
                    coinbase_gas_price * (gas.spent() - gas.refunded() as u64) as u128,
                ));

        Ok(())
    }

    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error> {
        context.take_error()?;

        // Used gas with refund calculated.
        let gas_refunded = result.gas().refunded() as u64;
        let final_gas_used = result.gas().spent() - gas_refunded;
        let output = result.output();
        let instruction_result = result.into_interpreter_result();

        // Reset journal and return present state.
        let (state, logs) = context.journal().finalize()?;

        let result = match SuccessOrHalt::<HALTREASON>::from(instruction_result.result) {
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

    fn clear(&self, context: &mut Self::Context) {
        // Clear error and journaled state.
        // TODO : Check effects of removal of take_error
        // let _ = context.evm.take_error();
        context.journal().clear();
    }
}

/// Trait for post execution context.
///
// TODO : Generalize FinalOutput.
pub trait EthPostExecutionContext:
    TransactionGetter
    + ErrorGetter<Error = JournalDBError<Self>>
    + BlockGetter
    + JournalGetter<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>
    + CfgGetter
{
}

impl<
        CTX: TransactionGetter
            + ErrorGetter<Error = JournalDBError<CTX>>
            + BlockGetter
            + JournalGetter<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>
            + CfgGetter,
    > EthPostExecutionContext for CTX
{
}

pub trait EthPostExecutionError<CTX: JournalGetter>: From<JournalDBError<CTX>> {}

impl<CTX: JournalGetter, ERROR: From<JournalDBError<CTX>>> EthPostExecutionError<CTX> for ERROR {}
