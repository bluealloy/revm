pub mod mainnet;
#[cfg(feature = "optimism")]
pub mod optimism;

mod register;

pub use register::{ExternalData, MainnetHandle, RegisterHandler};

use crate::{
    interpreter::{Gas, InstructionResult},
    primitives::{db::Database, EVMError, EVMResultGeneric, Env, Output, ResultAndState, Spec},
    CallStackFrame, Context, Evm,
};
use alloc::sync::Arc;
use core::ops::Range;
use revm_interpreter::{CallInputs, CreateInputs, InterpreterResult, SharedMemory};

/// Handle call return and return final gas value.
type CallReturnHandle<'a> = Arc<dyn Fn(&Env, InstructionResult, Gas) -> Gas + 'a>;

/// Reimburse the caller with ethereum it didn't spent.
type ReimburseCallerHandle<'a, EXT, DB> = Arc<
    dyn Fn(&mut Context<'_, EXT, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a,
>;

/// Reward beneficiary with transaction rewards.
type RewardBeneficiaryHandle<'a, EXT, DB> = ReimburseCallerHandle<'a, EXT, DB>;

/// Calculate gas refund for transaction.
type CalculateGasRefundHandle<'a> = Arc<dyn Fn(&Env, &Gas) -> u64 + 'a>;

/// Main return handle, takes state from journal and transforms internal result to external.
type MainReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<'_, EXT, DB>,
            InstructionResult,
            Output,
            &Gas,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// After subcall is finished, call this function to handle return result.
///
/// Return Some if we want to halt execution. This can be done on any stack frame.
type FrameReturn<'a, EXT, DB> = Arc<
    dyn Fn(
            // context
            &mut Context<'_, EXT, DB>,
            // returned frame
            Box<CallStackFrame>,
            // parent frame if it exist.
            Option<&mut Box<CallStackFrame>>,
            // shared memory to insert output of the call.
            &mut SharedMemory,
            // output of frame execution.
            InterpreterResult,
        ) -> Option<InterpreterResult>
        + 'a,
>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
type EndHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<'_, EXT, DB>,
            Result<ResultAndState, EVMError<<DB as Database>::Error>>,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

// Sub call
// type SubCall<DB: Database> = fn(
//     evm: &mut Evm<'_, SPEC, DB>,
//     inputs: Box<CallInputs>,
//     curent_stake_frame: &mut CallStackFrame,
//     shared_memory: &mut SharedMemory,
//     return_memory_offset: Range<usize>,
// ) -> Option<Box<CallStackFrame>>;

// /// sub create call
// type SubCreateCall<SPEC: Spec, DB: Database> = fn(
//     evm: &mut Evm<'_, SPEC, DB>,
//     curent_stack_frame: &mut CallStackFrame,
//     inputs: Box<CreateInputs>,
// ) -> Option<Box<CallStackFrame>>;

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<'a, EXT, DB: Database> {
    // Uses env, call result and returned gas from the call to determine the gas
    // that is returned from transaction execution..
    pub call_return: CallReturnHandle<'a>,
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<'a, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EXT, DB>,
    /// Calculate gas refund for transaction.
    /// Some chains have it disabled.
    pub calculate_gas_refund: CalculateGasRefundHandle<'a>,
    /// Main return handle, returns the output of the transact.
    pub main_return: MainReturnHandle<'a, EXT, DB>,
    /// End handle.
    pub end: EndHandle<'a, EXT, DB>,
    // Called on sub call.
    //pub sub_call: SubCall,
    /// Frame return
    pub frame_return: FrameReturn<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> Handler<'a, EXT, DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec + 'a>() -> Self {
        Self {
            call_return: Arc::new(mainnet::handle_call_return::<SPEC>),
            calculate_gas_refund: Arc::new(mainnet::calculate_gas_refund::<SPEC>),
            reimburse_caller: Arc::new(mainnet::handle_reimburse_caller::<SPEC, EXT, DB>),
            reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<SPEC, EXT, DB>),
            main_return: Arc::new(mainnet::main::main_return::<EXT, DB>),
            end: Arc::new(mainnet::main::end_handle::<EXT, DB>),
            frame_return: Arc::new(mainnet::frames::handle_frame_return::<SPEC, EXT, DB>),
        }
    }

    /// Handler for the optimism
    #[cfg(feature = "optimism")]
    pub fn optimism<SPEC: Spec>() -> Self {
        Self {
            call_return: optimism::handle_call_return::<SPEC>,
            calculate_gas_refund: optimism::calculate_gas_refund::<SPEC>,
            // we reinburse caller the same was as in mainnet.
            // Refund is calculated differently then mainnet.
            reimburse_caller: mainnet::handle_reimburse_caller::<SPEC, DB>,
            reward_beneficiary: optimism::reward_beneficiary::<SPEC, DB>,
            // In case of halt of deposit transaction return Error.
            main_return: optimism::main_return::<SPEC, DB>,
            end: optimism::end_handle::<SPEC, DB>,
            frame_return: Arc::new(mainnet::frames::handle_frame_return::<SPEC, EXT, DB>),
        }
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    pub fn call_return(&self, env: &Env, call_result: InstructionResult, returned_gas: Gas) -> Gas {
        (self.call_return)(env, call_result, returned_gas)
    }

    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<'_, EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reimburse_caller)(context, gas)
    }

    /// Calculate gas refund for transaction. Some chains have it disabled.
    pub fn calculate_gas_refund(&self, env: &Env, gas: &Gas) -> u64 {
        (self.calculate_gas_refund)(env, gas)
    }

    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<'_, EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reward_beneficiary)(context, gas)
    }

    /// Main return.
    pub fn main_return(
        &self,
        context: &mut Context<'_, EXT, DB>,
        call_result: InstructionResult,
        output: Output,
        gas: &Gas,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.main_return)(context, call_result, output, gas)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<'_, EXT, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.end)(context, end_output)
    }

    /// Frame return
    pub fn frame_return(
        &self,
        context: &mut Context<'_, EXT, DB>,
        child_stack_frame: Box<CallStackFrame>,
        parent_stack_frame: Option<&mut Box<CallStackFrame>>,
        shared_memory: &mut SharedMemory,
        result: InterpreterResult,
    ) -> Option<InterpreterResult> {
        (self.frame_return)(
            context,
            child_stack_frame,
            parent_stack_frame,
            shared_memory,
            result,
        )
    }
}
