// Includes.
use crate::{
    handler::mainnet::PostExecutionImpl,
    interpreter::Gas,
    primitives::{db::Database, EVMError, EVMResultGeneric, LatestSpec, ResultAndState, Spec},
    Context, FrameResult,
};

/// Reimburse the caller with ethereum it didn't spent.
pub trait ReimburseCallerTrait<EXT, DB: Database> {
    fn reimburse_caller(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), <DB as Database>::Error>;
}

/// Reward beneficiary with transaction rewards.
pub trait RewardBeneficiaryTrait<EXT, DB: Database> {
    fn reward_beneficiary(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), <DB as Database>::Error>;
}

/// Main return handle, takes state from journal and transforms internal result to external.
pub trait OutputTrait<EXT, DB: Database> {
    fn output(
        &self,
        context: &mut Context<EXT, DB>,
        result: FrameResult,
    ) -> EVMResultGeneric<ResultAndState, <DB as Database>::Error>;
}

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub trait EndTrait<EXT, DB: Database> {
    fn end(
        &self,
        context: &mut Context<EXT, DB>,
        end_output: Result<ResultAndState, EVMError<<DB as Database>::Error>>,
    ) -> EVMResultGeneric<ResultAndState, <DB as Database>::Error>;
}

/// Handles related to post execution after the stack loop is finished.
pub struct PostExecutionHandler<EXT, DB: Database> {
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: Box<dyn ReimburseCallerTrait<EXT, DB>>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: Box<dyn RewardBeneficiaryTrait<EXT, DB>>,
    /// Main return handle, returns the output of the transact.
    pub output: Box<dyn OutputTrait<EXT, DB>>,
    /// End handle.
    pub end: Box<dyn EndTrait<EXT, DB>>,
}

impl<EXT, DB: Database> Default for PostExecutionHandler<EXT, DB> {
    fn default() -> Self {
        Self::new::<LatestSpec>()
    }
}

impl<EXT, DB: Database> PostExecutionHandler<EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec>() -> Self {
        Self {
            reimburse_caller: Box::<PostExecutionImpl<SPEC>>::default(),
            reward_beneficiary: Box::<PostExecutionImpl<SPEC>>::default(),
            output: Box::<PostExecutionImpl<SPEC>>::default(),
            end: Box::<PostExecutionImpl<SPEC>>::default(),
        }
    }
}

impl<EXT, DB: Database> PostExecutionHandler<EXT, DB> {
    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        self.reimburse_caller.reimburse_caller(context, gas)
    }
    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        self.reward_beneficiary.reward_beneficiary(context, gas)
    }

    /// Returns the output of transaction.
    pub fn output(
        &self,
        context: &mut Context<EXT, DB>,
        result: FrameResult,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        self.output.output(context, result)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EXT, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        self.end.end(context, end_output)
    }
}
