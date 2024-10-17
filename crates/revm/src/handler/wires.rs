// Modules

pub mod execution;
pub mod generic;
pub mod post_execution;

// Exports

pub use execution::{
    ExecutionHandler, FrameCallHandle, FrameCallReturnHandle, FrameCreateHandle,
    FrameCreateReturnHandle, InsertCallOutcomeHandle, InsertCreateOutcomeHandle,
};
pub use generic::{GenericContextHandle, GenericContextHandleRet};
use interpreter::Gas;
pub use post_execution::{
    EndHandle, OutputHandle, PostExecutionHandler, ReimburseCallerHandle, RewardBeneficiaryHandle,
};

pub trait ValidationWire {
    type Context;
    type Error;

    /// Validate env.
    fn validate_env(&self, env: &Self::Context) -> Result<(), Self::Error>;

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
    fn refund(&self, ctx: &mut Self::Context, gas: &mut Gas, eip7702_refund: i64);

    /// Reimburse the caller with gas that were not spend.  
    fn reimburse_caller(&self, ctx: &mut Self::Context, gas: &Gas) -> Result<(), Self::Error>;

    /// Reward beneficiary
    fn reward_beneficiary(&self, ctx: &mut Self::Context, gas: &Gas) -> Result<(), Self::Error>;

    /// Returns the output of transaction.
    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<(), Self::Error>;

    /// Called when execution ends.
    /// End handle in comparison to output handle will be called every time after execution.
    /// While [`PostExecutionWire::output`] will be omitted in case of the error.
    fn end(
        &self,
        context: &mut Self::Context,
        end_output: Self::Output,
    ) -> Result<Self::Output, Self::Error>;

    /// Clean handler. This handle is called every time regardless
    /// of the result of the transaction.
    fn clear(&self, context: &mut Self::Context);
}

pub trait ExecutionWire {
    type Context;
    type Error;
    type Frame: Frame<Context = Self::Context>;
    type ExecResult;

    /// Execute call.
    fn first_frame(
        &self,
        context: &mut Self::Context,
    ) -> Result<<Self::Frame as Frame>::FrameInit, Self::Error>;

    /// Execute create.
    fn last_frame(
        &self,
        context: &mut Self::Context,
        frame: <Self::Frame as Frame>::FrameResult,
    ) -> Result<Self::ExecResult, Self::Error>;

    fn run(
        &self,
        context: &mut Self::Context,
        frame: Self::Frame,
    ) -> Result<Self::ExecResult, Self::Error> {
        let mut frame_stack: Vec<<Self as ExecutionWire>::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = frame.run((), context);

            let result = match call_or_result {
                FrameOrResult::Frame(init) => match frame.init(init, context) {
                    FrameOrResult::Frame(new_frame) => {
                        frame_stack.push(new_frame);
                        continue;
                    }
                    FrameOrResult::Result(result) => result,
                },
                FrameOrResult::Result(result) => result,
            };

            frame_stack.pop();

            let Some(frame) = frame_stack.last_mut() else {
                return self.last_frame(context, result);
            };

            frame.return_result(result);
        }
    }
}

pub enum FrameOrResult<Frame, Result> {
    Frame(Frame),
    Result(Result),
}

/// Makes sense
pub trait Frame: Sized {
    type Context;
    type FrameInit;
    type FrameResult;

    fn init(
        &self,
        frame_action: Self::FrameInit,
        cxt: &mut Self::Context,
    ) -> FrameOrResult<Self, Self::FrameResult>;

    fn run(
        &mut self,
        instructions: (),
        context: &mut Self::Context,
    ) -> FrameOrResult<Self::FrameInit, Self::FrameResult>;

    fn return_result(&mut self, result: Self::FrameResult);
}
