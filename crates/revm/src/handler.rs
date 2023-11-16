pub mod mainnet;
#[cfg(feature = "optimism")]
pub mod optimism;

use core::ops::Range;

use revm_interpreter::{CallInputs, CreateInputs, SharedMemory};

use crate::{
    interpreter::{Gas, InstructionResult},
    primitives::{db::Database, EVMError, EVMResultGeneric, Env, Output, ResultAndState, Spec},
    CallStackFrame, Context, Evm,
};

/// Handle call return and return final gas value.
type CallReturnHandle = fn(&Env, InstructionResult, Gas) -> Gas;

/// Reimburse the caller with ethereum it didn't spent.
type ReimburseCallerHandle<EXT, DB> =
    fn(&mut Context<'_, EXT, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error>;

/// Reward beneficiary with transaction rewards.
type RewardBeneficiaryHandle<EXT, DB> = ReimburseCallerHandle<EXT, DB>;

/// Calculate gas refund for transaction.
type CalculateGasRefundHandle = fn(&Env, &Gas) -> u64;

/// Main return handle, takes state from journal and transforms internal result to external.
type MainReturnHandle<EXT, DB> = fn(
    &mut Context<'_, EXT, DB>,
    InstructionResult,
    Output,
    &Gas,
) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
type EndHandle<EXT, DB> = fn(
    &mut Context<'_, EXT, DB>,
    evm_output: Result<ResultAndState, EVMError<<DB as Database>::Error>>,
) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>;

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
pub struct Handler<EXT, DB: Database> {
    // Uses env, call result and returned gas from the call to determine the gas
    // that is returned from transaction execution..
    pub call_return: CallReturnHandle,
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<EXT, DB>,
    /// Calculate gas refund for transaction.
    /// Some chains have it disabled.
    pub calculate_gas_refund: CalculateGasRefundHandle,
    /// Main return handle, returns the output of the transact.
    pub main_return: MainReturnHandle<EXT, DB>,
    /// End handle.
    pub end: EndHandle<EXT, DB>,
    // Called on sub call.
    //pub sub_call: SubCall,
}

impl<EXT, DB: Database> Handler<EXT, DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec>() -> Self {
        Self {
            call_return: mainnet::handle_call_return::<SPEC>,
            calculate_gas_refund: mainnet::calculate_gas_refund::<SPEC>,
            reimburse_caller: mainnet::handle_reimburse_caller::<SPEC, EXT, DB>,
            reward_beneficiary: mainnet::reward_beneficiary::<SPEC, EXT, DB>,
            main_return: mainnet::main::main_return::<EXT, DB>,
            end: mainnet::main::end_handle::<EXT, DB>,
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
}
