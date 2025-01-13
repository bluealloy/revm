use context_interface::{
    result::{HaltReason, InvalidHeader, InvalidTransaction, ResultAndState},
    JournalDBError, Transaction, TransactionGetter,
};
use handler_interface::{
    util::FrameOrFrameResult, ExecutionHandler, Frame, FrameOrResultGen, InitialAndFloorGas,
    PostExecutionHandler, PreExecutionHandler, ValidationHandler,
};
use interpreter::FrameInput;

use crate::{
    EthExecution, EthExecutionContext, EthPostExecution, EthPostExecutionContext, EthPreExecution,
    EthPreExecutionContext, EthValidation, EthValidationContext, FrameResult,
};

pub trait EthHandler {
    type Context: EthValidationContext
        + EthPreExecutionContext
        + EthExecutionContext
        + EthPostExecutionContext;
    type Error: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalDBError<Self::Context>>
        + From<InvalidTransaction>;
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    type Frame: Frame<
        Context = Self::Context,
        Error = Self::Error,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
    >;
    type SharedContext;

    fn execute(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(context)?;
        let eip7702_refund = self.pre_execution(context)? as i64;
        let exec_result = self.execution(context, &init_and_floor_gas)?;
        let post_execution_gas = (init_and_floor_gas, eip7702_refund);
        self.post_execution(context, exec_result, post_execution_gas)
    }

    /// Call all validation functions
    fn validate(&self, context: &mut Self::Context) -> Result<InitialAndFloorGas, Self::Error> {
        self.validate_env(context)?;
        self.validate_tx_against_state(context)?;
        self.validate_initial_tx_gas(context)
    }

    /// Call all Pre execution functions.
    fn pre_execution(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        self.load_accounts(context)?;
        let gas = self.apply_eip7702_auth_list(context)?;
        self.deduct_caller(context)?;
        Ok(gas)
    }

    fn execution(
        &mut self,
        context: &mut Self::Context,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = context.tx().gas_limit() - init_and_floor_gas.initial_gas;
        // Create first frame action
        let first_frame = self.init_first_frame(context, gas_limit)?;
        let frame_result = match first_frame {
            FrameOrResultGen::Frame(frame) => self.run_loop(context, frame)?,
            FrameOrResultGen::Result(result) => result,
        };

        self.last_frame_result(context, frame_result)
    }

    fn post_execution(
        &self,
        context: &mut Self::Context,
        mut exec_result: FrameResult,
        post_execution_gas: (InitialAndFloorGas, i64),
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        let init_and_floor_gas = post_execution_gas.0;
        let eip7702_gas_refund = post_execution_gas.1;

        self.eip7623_check_gas_floor(context, &mut exec_result, init_and_floor_gas);
        // Calculate final refund and add EIP-7702 refund to gas.
        self.refund(context, &mut exec_result, eip7702_gas_refund);
        // Reimburse the caller
        self.reimburse_caller(context, &mut exec_result)?;
        // Reward beneficiary
        self.reward_beneficiary(context, &mut exec_result)?;
        // Returns output of transaction.
        self.output(context, exec_result)
    }

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        EthValidation::new().validate_env(context)
    }

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        EthValidation::new().validate_tx_against_state(context)
    }

    /// Validate initial gas.
    fn validate_initial_tx_gas(
        &self,
        context: &Self::Context,
    ) -> Result<InitialAndFloorGas, Self::Error> {
        EthValidation::new().validate_initial_tx_gas(context)
    }

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        EthPreExecution::new().load_accounts(context)
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        EthPreExecution::new().apply_eip7702_auth_list(context)
    }

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        EthPreExecution::new().deduct_caller(context)
    }

    /* EXECUTION */
    fn init_first_frame(
        &mut self,
        context: &mut Self::Context,
        gas_limit: u64,
    ) -> Result<FrameOrFrameResult<Self::Frame>, Self::Error> {
        EthExecution::new().init_first_frame(context, gas_limit)
    }

    fn init(
        &self,
        frame: &Self::Frame,
        context: &mut Self::Context,
        frame_input: <Self as EthHandler>::Frame::FrameInit,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error> {
    }

    fn run_loop(
        &self,
        context: &mut Self::Context,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = frame.run(context)?;

            let mut result = match call_or_result {
                FrameOrResultGen::Frame(init) => match frame.init(context, init)? {
                    FrameOrResultGen::Frame(new_frame) => {
                        frame_stack.push(new_frame);
                        continue;
                    }
                    // Dont pop the frame as new frame was not created.
                    FrameOrResultGen::Result(result) => result,
                },
                FrameOrResultGen::Result(result) => {
                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                Self::Frame::final_return(context, &mut result)?;
                return self.last_frame_result(context, result);
            };
            frame.return_result(context, result)?;
        }
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        frame_result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<<Self::Frame as Frame>::FrameResult, Self::Error> {
        EthExecution::<
            <Self as EthHandler>::Context,
            <Self as EthHandler>::Error,
            <Self as EthHandler>::Frame,
        >::new()
        .last_frame_result(context, frame_result)
    }

    /* POST EXECUTION */

    /// Calculate final refund.
    fn eip7623_check_gas_floor(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        EthPostExecution::<_, Self::Error, HaltReason>::new().eip7623_check_gas_floor(
            context,
            exec_result,
            init_and_floor_gas,
        )
    }

    /// Calculate final refund.
    fn refund(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        eip7702_refund: i64,
    ) {
        EthPostExecution::<_, Self::Error, HaltReason>::new().refund(
            context,
            exec_result,
            eip7702_refund,
        )
    }

    /// Reimburse the caller with balance it didn't spent.
    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        EthPostExecution::<_, Self::Error, HaltReason>::new().reimburse_caller(context, exec_result)
    }

    /// Reward beneficiary with transaction rewards.
    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        EthPostExecution::<_, Self::Error, HaltReason>::new()
            .reward_beneficiary(context, exec_result)
    }

    /// Main return handle, takes state from journal and transforms internal result to [`Output`][PostExecutionHandler::Output].
    fn output(
        &self,
        context: &mut Self::Context,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        EthPostExecution::<_, Self::Error, HaltReason>::new().output(context, result)
    }

    /// Called when execution ends.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    ///
    /// While [`output`][PostExecutionHandler::output] will be omitted in case of the error.
    fn end(
        &self,
        _context: &mut Self::Context,
        end_output: Result<ResultAndState<HaltReason>, Self::Error>,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        end_output
    }

    /// Clean handler.
    ///
    /// This handle is called every time regardless of the result of the transaction.
    fn clear(&self, context: &mut Self::Context) {
        EthPostExecution::<_, Self::Error, HaltReason>::new().clear(context)
    }
}
