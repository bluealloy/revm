// Modules

use interpreter::table::InstructionTables;

pub trait ValidationWire {
    type Context;
    type Error;

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error>;

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error>;

    /// Validate initial gas.
    fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error>;
}

pub trait PreExecutionWire {
    type Context;
    type Precompiles;
    type Error;

    fn load_precompiles(&self) -> Self::Precompiles;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error>;

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error>;

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error>;
}

pub trait PostExecutionWire {
    type Context;
    type Error;
    type ExecResult;
    type Output;

    /// Calculate final refund
    fn refund(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
        eip7702_refund: i64,
    );

    /// Reimburse the caller with balance it didn't spent.
    fn reimburse_caller(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error>;

    /// Reward beneficiary with transaction rewards.
    fn reward_beneficiary(
        &self,
        ctx: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error>;

    /// Main return handle, takes state from journal and transforms internal result to [`PostExecutionWire::Output`].
    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when execution ends.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    /// While [`PostExecutionWire::output`] will be omitted in case of the error.
    fn end(
        &self,
        _context: &mut Self::Context,
        end_output: Result<Self::Output, Self::Error>,
    ) -> Result<Self::Output, Self::Error> {
        end_output
    }

    /// Clean handler. This handle is called every time regardless
    /// of the result of the transaction.
    fn clear(&self, context: &mut Self::Context);
}

pub trait ExecutionWire {
    type Context;
    type Error;
    type Frame: Frame<Context = Self::Context, Error = Self::Error>;
    type ExecResult;

    /// Execute call.
    fn init_first_frame(
        &self,
        context: &mut Self::Context,
        gas_limit: u64,
    ) -> Result<FrameOrResultGen<Self::Frame, <Self::Frame as Frame>::FrameResult>, Self::Error>;

    /// Execute create.
    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        frame: <Self::Frame as Frame>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error>;

    fn run(
        &self,
        context: &mut Self::Context,
        instructions: &InstructionTables<'_, Self::Context>,
        frame: Self::Frame,
    ) -> Result<Self::ExecResult, Self::Error> {
        let mut frame_stack: Vec<<Self as ExecutionWire>::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = frame.run(/*instructions,*/ context)?;

            let result = match call_or_result {
                FrameOrResultGen::Frame(init) => match frame.init(init, context)? {
                    FrameOrResultGen::Frame(new_frame) => {
                        frame_stack.push(new_frame);
                        continue;
                    }
                    FrameOrResultGen::Result(result) => result,
                },
                FrameOrResultGen::Result(result) => result,
            };

            frame_stack.pop();

            let Some(frame) = frame_stack.last_mut() else {
                return self.last_frame_result(context, result);
            };

            frame.return_result(context, result)?;
        }
    }
}

pub enum FrameOrResultGen<Frame, Result> {
    Frame(Frame),
    Result(Result),
}

/// Makes sense
pub trait Frame: Sized {
    type Context;
    type FrameInit;
    type FrameResult;
    type Error;

    fn init(
        &self,
        frame_action: Self::FrameInit,
        cxt: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self, Self::FrameResult>, Self::Error>;

    fn run(
        &mut self,
        //instructions: &InstructionTables<'_, Self::Context>,
        context: &mut Self::Context,
    ) -> Result<FrameOrResultGen<Self::FrameInit, Self::FrameResult>, Self::Error>;

    fn return_result(
        &mut self,
        cxt: &mut Self::Context,
        result: Self::FrameResult,
    ) -> Result<(), Self::Error>;
}
