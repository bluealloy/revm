use crate::{token_operation, TREASURY};
use revm::context_interface::result::{HaltReasonTrait, InvalidHeader, InvalidTransaction};
use revm::context_interface::JournalDBError;
use revm::handler::{EthPostExecutionContext, EthPostExecutionError};
use revm::precompile::PrecompileErrors;
use revm::{
    context::Cfg,
    context_interface::{
        result::{HaltReason, ResultAndState},
        Block, Transaction, TransactionGetter,
    },
    handler::{EthPostExecution, FrameResult},
    handler_interface::PostExecutionHandler,
    primitives::U256,
    specification::hardfork::SpecId,
};

pub struct Erc20PostExecution<CTX, ERROR, HALTREASON = HaltReason> {
    inner: EthPostExecution<CTX, ERROR, HALTREASON>,
}

impl<CTX, ERROR, HALTREASON> Erc20PostExecution<CTX, ERROR, HALTREASON> {
    pub fn new() -> Self {
        Self {
            inner: EthPostExecution::new(),
        }
    }
}

impl<CTX, ERROR, HALTREASON> Default for Erc20PostExecution<CTX, ERROR, HALTREASON> {
    fn default() -> Self {
        Self::new()
    }
}

impl<CTX, ERROR, HALTREASON> PostExecutionHandler for Erc20PostExecution<CTX, ERROR, HALTREASON>
where
    CTX: EthPostExecutionContext<ERROR>,
    ERROR: EthPostExecutionError<CTX>
        + From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalDBError<CTX>>
        + From<PrecompileErrors>,
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
        self.inner.refund(context, exec_result, eip7702_refund)
    }

    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let basefee = context.block().basefee() as u128;
        let caller = context.tx().common_fields().caller();
        let effective_gas_price = context.tx().effective_gas_price(basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price.saturating_mul((gas.remaining() + gas.refunded() as u64) as u128);
        token_operation::<CTX, ERROR>(context, TREASURY, caller, U256::from(reimbursement))?;

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let tx = context.tx();
        let beneficiary = context.block().beneficiary();
        let basefee = context.block().basefee() as u128;
        let effective_gas_price = tx.effective_gas_price(basefee);
        let gas = exec_result.gas();

        let coinbase_gas_price = if context.cfg().spec().into().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(basefee)
        } else {
            effective_gas_price
        };

        let reward =
            coinbase_gas_price.saturating_mul((gas.spent() - gas.refunded() as u64) as u128);
        token_operation::<CTX, ERROR>(context, TREASURY, beneficiary, U256::from(reward))?;

        Ok(())
    }

    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error> {
        self.inner.output(context, result)
    }

    fn clear(&self, context: &mut Self::Context) {
        self.inner.clear(context)
    }
}
