use crate::{
    handler::{EndTrait, OutputTrait, ReimburseCallerTrait, RewardBeneficiaryTrait},
    interpreter::{Gas, SuccessOrHalt},
    primitives::{
        db::Database, EVMError, ExecutionResult, ResultAndState, Spec, SpecId::LONDON, U256,
    },
    Context, FrameResult,
};

/// PostExecutionImpl implements all traits related to post execution handles.
#[derive(Clone, Debug)]
pub struct PostExecutionImpl<SPEC> {
    pub _spec: std::marker::PhantomData<SPEC>,
}

impl<SPEC: Spec> Default for PostExecutionImpl<SPEC> {
    fn default() -> Self {
        Self {
            _spec: std::marker::PhantomData,
        }
    }
}

impl<SPEC: Spec, EXT, DB: Database> EndTrait<EXT, DB> for PostExecutionImpl<SPEC> {
    #[inline]
    fn end(
        &self,
        _context: &mut Context<EXT, DB>,
        evm_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        evm_output
    }
}

impl<SPEC: Spec, EXT, DB: Database> RewardBeneficiaryTrait<EXT, DB> for PostExecutionImpl<SPEC> {
    #[inline]
    fn reward_beneficiary(
        &self,
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
            .saturating_add(coinbase_gas_price * U256::from(gas.spend() - gas.refunded() as u64));

        Ok(())
    }
}

impl<SPEC: Spec, EXT, DB: Database> ReimburseCallerTrait<EXT, DB> for PostExecutionImpl<SPEC> {
    fn reimburse_caller(
        &self,
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

        caller_account.info.balance = caller_account.info.balance.saturating_add(
            effective_gas_price * U256::from(gas.remaining() + gas.refunded() as u64),
        );

        Ok(())
    }
}

impl<SPEC: Spec, EXT, DB: Database> OutputTrait<EXT, DB> for PostExecutionImpl<SPEC> {
    #[inline]
    fn output(
        &self,
        context: &mut Context<EXT, DB>,
        result: FrameResult,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        core::mem::replace(&mut context.evm.error, Ok(()))?;
        // used gas with refund calculated.
        let gas_refunded = result.gas().refunded() as u64;
        let final_gas_used = result.gas().spend() - gas_refunded;
        let output = result.output();
        let instruction_result = result.into_interpreter_result();

        // reset journal and return present state.
        let (state, logs) = context.evm.journaled_state.finalize();

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
            SuccessOrHalt::FatalExternalError
            | SuccessOrHalt::InternalContinue
            | SuccessOrHalt::InternalCallOrCreate => {
                panic!("Internal return flags should remain internal {instruction_result:?}")
            }
        };

        Ok(ResultAndState { result, state })
    }
}
