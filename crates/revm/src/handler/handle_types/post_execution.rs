// Includes.
use crate::{
    handler::mainnet,
    interpreter::Gas,
    primitives::{db::Database, EVMResultGeneric, ResultAndState, Spec},
    Context, EvmWiring, FrameResult,
};
use std::sync::Arc;

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            &Gas,
        ) -> EVMResultGeneric<(), EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, EvmWiringT, EXT, DB> =
    ReimburseCallerHandle<'a, EvmWiringT, EXT, DB>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type OutputHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            FrameResult,
        )
            -> EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub type EndHandle<'a, EvmWiringT, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT, EXT, DB>,
            EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, <DB as Database>::Error>,
        )
            -> EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, <DB as Database>::Error>
        + 'a,
>;

/// Clear handle, doesn't have output, its purpose is to clear the
/// context. It will always be called even on failed validation.
pub type ClearHandle<'a, EvmWiringT, EXT, DB> = Arc<dyn Fn(&mut Context<EvmWiringT, EXT, DB>) + 'a>;

/// Handles related to post execution after the stack loop is finished.
pub struct PostExecutionHandler<'a, EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Reimburse the caller with ethereum it didn't spend.
    pub reimburse_caller: ReimburseCallerHandle<'a, EvmWiringT, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EvmWiringT, EXT, DB>,
    /// Main return handle, returns the output of the transact.
    pub output: OutputHandle<'a, EvmWiringT, EXT, DB>,
    /// Called when execution ends.
    /// End handle in comparison to output handle will be called every time after execution.
    /// Output in case of error will not be called.
    pub end: EndHandle<'a, EvmWiringT, EXT, DB>,
    /// Clear handle will be called always. In comparison to end that
    /// is called only on execution end, clear handle is called even if validation fails.
    pub clear: ClearHandle<'a, EvmWiringT, EXT, DB>,
}

impl<'a, EvmWiringT: EvmWiring, EXT: 'a, DB: Database + 'a>
    PostExecutionHandler<'a, EvmWiringT, EXT, DB>
{
    /// Creates mainnet MainHandles.
    pub fn mainnet<SPEC: Spec + 'a>() -> Self {
        Self {
            reimburse_caller: Arc::new(mainnet::reimburse_caller::<EvmWiringT, EXT, DB>),
            reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<EvmWiringT, SPEC, EXT, DB>),
            output: Arc::new(mainnet::output::<EvmWiringT, EXT, DB>),
            end: Arc::new(mainnet::end::<EvmWiringT, EXT, DB>),
            clear: Arc::new(mainnet::clear::<EvmWiringT, EXT, DB>),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring, EXT, DB: Database> PostExecutionHandler<'a, EvmWiringT, EXT, DB> {
    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.reimburse_caller)(context, gas)
    }
    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), EvmWiringT, DB::Error> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Returns the output of transaction.
    pub fn output(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        result: FrameResult,
    ) -> EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, DB::Error> {
        (self.output)(context, result)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EvmWiringT, EXT, DB>,
        end_output: EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, DB::Error>,
    ) -> EVMResultGeneric<ResultAndState<EvmWiringT>, EvmWiringT, DB::Error> {
        (self.end)(context, end_output)
    }

    /// Clean handler.
    pub fn clear(&self, context: &mut Context<EvmWiringT, EXT, DB>) {
        (self.clear)(context)
    }
}
