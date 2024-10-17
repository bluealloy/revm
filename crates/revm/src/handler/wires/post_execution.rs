// Includes.
use crate::{handler::mainnet, Context, EvmWiring, FrameResult};
use interpreter::Gas;
use specification::hardfork::Spec;
use std::sync::Arc;
use wiring::result::{EVMResult, EVMResultGeneric, ResultAndState};

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>, &Gas) -> EVMResultGeneric<(), EvmWiringT> + 'a>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, EvmWiringT> = ReimburseCallerHandle<'a, EvmWiringT>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type OutputHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>, FrameResult) -> EVMResult<EvmWiringT> + 'a>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub type EndHandle<'a, EvmWiringT> =
    Arc<dyn Fn(&mut Context<EvmWiringT>, EVMResult<EvmWiringT>) -> EVMResult<EvmWiringT> + 'a>;

/// Clear handle, doesn't have output, its purpose is to clear the
/// context. It will always be called even on failed validation.
pub type ClearHandle<'a, EvmWiringT> = Arc<dyn Fn(&mut Context<EvmWiringT>) + 'a>;

/// Refund handle, calculates the final refund.
pub type RefundHandle<'a, EvmWiringT> = Arc<dyn Fn(&mut Context<EvmWiringT>, &mut Gas, i64) + 'a>;
/// Handles related to post execution after the stack loop is finished.
pub struct PostExecutionHandler<'a, EvmWiringT: EvmWiring> {
    /// Calculate final refund
    pub refund: RefundHandle<'a, EvmWiringT>,
    /// Reimburse the caller with ethereum it didn't spend.
    pub reimburse_caller: ReimburseCallerHandle<'a, EvmWiringT>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EvmWiringT>,
    /// Main return handle, returns the output of the transact.
    pub output: OutputHandle<'a, EvmWiringT>,
    /// Called when execution ends.
    /// End handle in comparison to output handle will be called every time after execution.
    /// Output in case of error will not be called.
    pub end: EndHandle<'a, EvmWiringT>,
    /// Clear handle will be called always. In comparison to end that
    /// is called only on execution end, clear handle is called even if validation fails.
    pub clear: ClearHandle<'a, EvmWiringT>,
}

impl<'a, EvmWiringT: EvmWiring + 'a> PostExecutionHandler<'a, EvmWiringT> {
    /// Creates mainnet MainHandles.
    pub fn mainnet<SPEC: Spec + 'a>() -> Self {
        Self {
            refund: Arc::new(mainnet::refund::<EvmWiringT, SPEC>),
            reimburse_caller: Arc::new(mainnet::reimburse_caller::<EvmWiringT>),
            reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<EvmWiringT, SPEC>),
            output: Arc::new(mainnet::output::<EvmWiringT>),
            end: Arc::new(mainnet::end::<EvmWiringT>),
            clear: Arc::new(mainnet::clear::<EvmWiringT>),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring> PostExecutionHandler<'a, EvmWiringT> {
    /// Calculate final refund
    pub fn refund(&self, context: &mut Context<EvmWiringT>, gas: &mut Gas, eip7702_refund: i64) {
        (self.refund)(context, gas, eip7702_refund)
    }

    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EvmWiringT>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.reimburse_caller)(context, gas)
    }
    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EvmWiringT>,
        gas: &Gas,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Returns the output of transaction.
    pub fn output(
        &self,
        context: &mut Context<EvmWiringT>,
        result: FrameResult,
    ) -> EVMResult<EvmWiringT> {
        (self.output)(context, result)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EvmWiringT>,
        end_output: EVMResultGeneric<ResultAndState<EvmWiringT::HaltReason>, EvmWiringT>,
    ) -> EVMResult<EvmWiringT> {
        (self.end)(context, end_output)
    }

    /// Clean handler.
    pub fn clear(&self, context: &mut Context<EvmWiringT>) {
        (self.clear)(context)
    }
}
