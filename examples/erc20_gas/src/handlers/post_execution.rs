use crate::error::Erc20Error;
use crate::{token_operation, TREASURY};
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
    Context,
};

pub struct Erc20PostExecution {
    inner: EthPostExecution<Context, Erc20Error, HaltReason>,
}

impl Erc20PostExecution {
    pub fn new() -> Self {
        Self {
            inner: EthPostExecution::new(),
        }
    }
}

impl PostExecutionHandler for Erc20PostExecution {
    type Context = Context;
    type Error = Erc20Error;
    type ExecResult = FrameResult;
    type Output = ResultAndState<HaltReason>;

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
        let basefee = context.block.basefee();
        let caller = context.tx().common_fields().caller();
        let effective_gas_price = context.tx().effective_gas_price(*basefee);
        let gas = exec_result.gas();

        let reimbursement =
            effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64);
        token_operation(context, TREASURY, caller, reimbursement).unwrap();

        Ok(())
    }

    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error> {
        let tx = context.tx();
        let beneficiary = context.block.beneficiary();
        let basefee = context.block.basefee();
        let effective_gas_price = tx.effective_gas_price(*basefee);
        let gas = exec_result.gas();

        let coinbase_gas_price = if context.cfg.spec().is_enabled_in(SpecId::LONDON) {
            effective_gas_price.saturating_sub(*basefee)
        } else {
            effective_gas_price
        };

        let reward = coinbase_gas_price * U256::from(gas.spent() - gas.refunded() as u64);
        token_operation(context, TREASURY, *beneficiary, reward).unwrap();

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
