use crate::{handler::PostExecutionWire, Context, EvmWiring, FrameResult};
use context::{
    BlockGetter, CfgGetter, ErrorGetter, JournalStateGetter, JournalStateGetterDBError,
    TransactionGetter,
};
use interpreter::{Gas, InternalResult, SuccessOrHalt};
use primitives::{Log, U256};
use specification::hardfork::{Spec, SpecId};
use state::EvmState;
use wiring::{
    journaled_state::JournaledState,
    result::{
        EVMError, EVMResult, EVMResultGeneric, ExecutionResult, InvalidTransaction, ResultAndState,
    },
    Block, HaltReasonTrait, Transaction,
};

pub struct EthPostExecution<CTX, ERROR, HALT> {
    pub spec_id: SpecId,
    pub _phantom: std::marker::PhantomData<(CTX, ERROR, HALT)>,
}

impl<CTX, ERROR, HALT> EthPostExecution<CTX, ERROR, HALT> {
    /// Create new instance of post execution handler.
    pub fn new(spec_id: SpecId) -> Self {
        Self {
            spec_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create new boxed instance of post execution handler.
    ///
    /// Boxed instance is useful to erase FORK type.
    pub fn new_boxed(spec_id: SpecId) -> Box<Self> {
        Box::new(Self::new(spec_id))
    }
}

impl<CTX, ERROR, HALT: HaltReasonTrait> PostExecutionWire for EthPostExecution<CTX, ERROR, HALT>
where
    CTX: TransactionGetter
        + ErrorGetter<Error = ERROR>
        + BlockGetter
        + JournalStateGetter<Journal: JournaledState<FinalOutput = (EvmState, Vec<Log>)>>
        + CfgGetter,
    ERROR: From<InvalidTransaction> + From<JournalStateGetterDBError<CTX>>,
{
    type Context = CTX;
    type Error = ERROR;
    type ExecResult = FrameResult;
    type Output = ResultAndState<HALT>;

    fn refund(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
        eip7702_refund: i64,
    ) {
        let gas = exec_result.gas_mut();
        gas.record_refund(eip7702_refund);

        // Calculate gas refund for transaction.
        // If spec is set to london, it will decrease the maximum refund amount to 5th part of
        // gas spend. (Before london it was 2th part of gas spend)
        gas.set_final_refund(self.spec_id.is_enabled_in(SpecId::LONDON));
    }

    fn reimburse_caller(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let basefee = *ctx.block().basefee();
        let caller = ctx.tx().common_fields().caller();
        let effective_gas_price = ctx.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        // return balance of not spend gas.
        let caller_account = ctx.journal().load_account(caller)?;

        caller_account.data.info.balance = caller_account.data.info.balance.saturating_add(
            effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64),
        );

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let block = ctx.block();
        let tx = ctx.tx();
        let beneficiary = *block.beneficiary();
        let basefee = *block.basefee();
        let effective_gas_price = tx.effective_gas_price(basefee);
        let gas = exec_result.gas();

        // transfer fee to coinbase/beneficiary.
        // EIP-1559 discard basefee for coinbase transfer. Basefee amount of gas is discarded.
        let coinbase_gas_price = if self.spec_id.is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let coinbase_account = ctx.journal().load_account(beneficiary)?;

        coinbase_account.data.mark_touch();
        coinbase_account.data.info.balance =
            coinbase_account.data.info.balance.saturating_add(
                coinbase_gas_price * U256::from(gas.spent() - gas.refunded() as u64),
            );

        Ok(())
    }

    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error> {
        context.take_error()?;

        // used gas with refund calculated.
        let gas_refunded = result.gas().refunded() as u64;
        let final_gas_used = result.gas().spent() - gas_refunded;
        let output = result.output();
        let instruction_result = result.into_interpreter_result();

        // reset journal and return present state.
        let (state, logs) = context.journal().finalize()?;

        let result = match SuccessOrHalt::<HALT>::from(instruction_result.result) {
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
        // clear error and journaled state.
        // TODO check effects of removal of take_error
        // let _ = context.evm.take_error();
        context.journal().clear();
    }
}
