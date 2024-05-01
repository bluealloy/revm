// Includes.
use crate::{
    handler::mainnet,
    interpreter::Gas,
    primitives::{db::Database, ChainSpec, EVMError, EVMResultGeneric, ResultAndState, Spec},
    Context, FrameResult,
};
use std::sync::Arc;

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            &Gas,
        ) -> EVMResultGeneric<(), ChainSpecT, <DB as Database>::Error>
        + 'a,
>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, ChainSpecT, EXT, DB> =
    ReimburseCallerHandle<'a, ChainSpecT, EXT, DB>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type OutputHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            FrameResult,
        )
            -> Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub type EndHandle<'a, ChainSpecT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<ChainSpecT, EXT, DB>,
            Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, <DB as Database>::Error>>,
        )
            -> Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, <DB as Database>::Error>>
        + 'a,
>;

/// Clear handle, doesn't have output, its purpose is to clear the
/// context. It will be always called even on failed validation.
pub type ClearHandle<'a, ChainSpecT, EXT, DB> = Arc<dyn Fn(&mut Context<ChainSpecT, EXT, DB>) + 'a>;

/// Handles related to post execution after the stack loop is finished.
pub struct PostExecutionHandler<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<'a, ChainSpecT, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, ChainSpecT, EXT, DB>,
    /// Main return handle, returns the output of the transact.
    pub output: OutputHandle<'a, ChainSpecT, EXT, DB>,
    /// Called when execution ends.
    /// End handle in comparison to output handle will be called every time after execution.
    /// Output in case of error will not be called.
    pub end: EndHandle<'a, ChainSpecT, EXT, DB>,
    /// Clear handle will be called always. In comparison to end that
    /// is called only on execution end, clear handle is called even if validation fails.
    pub clear: ClearHandle<'a, ChainSpecT, EXT, DB>,
}

impl<'a, ChainSpecT: ChainSpec, EXT: 'a, DB: Database + 'a>
    PostExecutionHandler<'a, ChainSpecT, EXT, DB>
{
    /// Creates mainnet MainHandles.
    pub fn mainnet<SPEC: Spec + 'a>() -> Self {
        Self {
            reimburse_caller: Arc::new(mainnet::reimburse_caller::<ChainSpecT, EXT, DB>),
            reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<ChainSpecT, SPEC, EXT, DB>),
            output: Arc::new(mainnet::output::<ChainSpecT, EXT, DB>),
            end: Arc::new(mainnet::end::<ChainSpecT, EXT, DB>),
            clear: Arc::new(mainnet::clear::<ChainSpecT, EXT, DB>),
        }
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> PostExecutionHandler<'a, ChainSpecT, EXT, DB> {
    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.reimburse_caller)(context, gas)
    }
    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<ChainSpecT, DB::Error>> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Returns the output of transaction.
    pub fn output(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        result: FrameResult,
    ) -> Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, DB::Error>> {
        (self.output)(context, result)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<ChainSpecT, EXT, DB>,
        end_output: Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, DB::Error>>,
    ) -> Result<ResultAndState<ChainSpecT>, EVMError<ChainSpecT, DB::Error>> {
        (self.end)(context, end_output)
    }

    /// Clean handler.
    pub fn clear(&self, context: &mut Context<ChainSpecT, EXT, DB>) {
        (self.clear)(context)
    }
}
