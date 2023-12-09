// Includes.
use crate::{
    handler::mainnet,
    interpreter::{Gas, InstructionResult},
    primitives::{db::Database, EVMError, EVMResultGeneric, Output, ResultAndState, Spec},
    Context,
};
use alloc::sync::Arc;

/// Load access list account, precompiles and beneficiary.
/// There is not need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type MainLoadHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> Result<(), EVMError<<DB as Database>::Error>> + 'a>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, EXT, DB> = ReimburseCallerHandle<'a, EXT, DB>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type MainReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            InstructionResult,
            Output,
            &Gas,
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

/// Handles related to main function.
pub struct MainHandler<'a, EXT, DB: Database> {
    /// Main load handle
    pub load: MainLoadHandle<'a, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EXT, DB>,
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<'a, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EXT, DB>,
    /// Main return handle, returns the output of the transact.
    pub main_return: MainReturnHandle<'a, EXT, DB>,
    /// End handle.
    pub end: EndHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> MainHandler<'a, EXT, DB> {
    /// Creates mainnet MainHandles.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            load: Arc::new(mainnet::main_load::<SPEC, EXT, DB>),
            deduct_caller: Arc::new(mainnet::main_deduct_caller::<SPEC, EXT, DB>),
            reimburse_caller: Arc::new(mainnet::main_reimburse_caller::<SPEC, EXT, DB>),
            reward_beneficiary: Arc::new(mainnet::main_reward_beneficiary::<SPEC, EXT, DB>),
            main_return: Arc::new(mainnet::main_return::<EXT, DB>),
            end: Arc::new(mainnet::main_end::<EXT, DB>),
        }
    }
}

impl<'a, EXT, DB: Database> MainHandler<'a, EXT, DB> {
    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reimburse_caller)(context, gas)
    }

    /// Deduct caller to its limit.
    pub fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.deduct_caller)(context)
    }

    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Main return.
    pub fn main_return(
        &self,
        context: &mut Context<EXT, DB>,
        call_result: InstructionResult,
        output: Output,
        gas: &Gas,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.main_return)(context, call_result, output, gas)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EXT, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.end)(context, end_output)
    }

    /// Main load
    pub fn load(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.load)(context)
    }
}
