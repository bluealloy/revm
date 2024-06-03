// Includes.
use crate::{
    handler::mainnet,
    interpreter::Gas,
    primitives::{db::Database, EVMError, EVMResultGeneric, ResultAndState, Spec},
    Context, FrameResult,
};
use std::sync::Arc;

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, EXT, DB> = ReimburseCallerHandle<'a, EXT, DB>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type OutputHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            FrameResult,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub type EndHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Result<ResultAndState, EVMError<<DB as Database>::Error>>,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Clear handle, doesn't have output, its purpose is to clear the
/// context. It will be always called even on failed validation.
pub type ClearHandle<'a, EXT, DB> = Arc<dyn Fn(&mut Context<EXT, DB>) + 'a>;

/// Handles related to post execution after the stack loop is finished.
pub struct PostExecutionHandler<'a, EXT, DB: Database> {
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<'a, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EXT, DB>,
    /// Main return handle, returns the output of the transact.
    pub output: OutputHandle<'a, EXT, DB>,
    /// Called when execution ends.
    /// End handle in comparison to output handle will be called every time after execution.
    /// Output in case of error will not be called.
    pub end: EndHandle<'a, EXT, DB>,
    /// Clear handle will be called always. In comparison to end that
    /// is called only on execution end, clear handle is called even if validation fails.
    pub clear: ClearHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> PostExecutionHandler<'a, EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            reimburse_caller: Arc::new(mainnet::reimburse_caller::<SPEC, EXT, DB>),
            reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<SPEC, EXT, DB>),
            output: Arc::new(mainnet::output::<EXT, DB>),
            end: Arc::new(mainnet::end::<EXT, DB>),
            clear: Arc::new(mainnet::clear::<EXT, DB>),
        }
    }
}

impl<'a, EXT, DB: Database> PostExecutionHandler<'a, EXT, DB> {
    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reimburse_caller)(context, gas)
    }
    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Returns the output of transaction.
    pub fn output(
        &self,
        context: &mut Context<EXT, DB>,
        result: FrameResult,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.output)(context, result)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EXT, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.end)(context, end_output)
    }

    /// Clean handler.
    pub fn clear(&self, context: &mut Context<EXT, DB>) {
        (self.clear)(context)
    }
}
