use crate::{
    gas::Gas,
    mainnet,
    primitives::{db::Database, EVMError, EVMResultGeneric, Env, Output, ResultAndState, Spec},
    EVMData,
};
use fluentbase_types::ExitCode;

/// Handle call return and return final gas value.
type CallReturnHandle = fn(&Env, ExitCode, Gas) -> Gas;

/// Reimburse the caller with ethereum it didn't spent.
type ReimburseCallerHandle<DB> =
    fn(&mut EVMData<'_, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error>;

/// Reward beneficiary with transaction rewards.
type RewardBeneficiaryHandle<DB> = ReimburseCallerHandle<DB>;

/// Calculate gas refund for transaction.
type CalculateGasRefundHandle = fn(&Env, &Gas) -> u64;

/// Main return handle, takes state from journal and transforms internal result to external.
type MainReturnHandle<DB> = fn(
    &mut EVMData<'_, DB>,
    ExitCode,
    Output,
    &Gas,
) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
type EndHandle<DB> = fn(
    &mut EVMData<'_, DB>,
    evm_output: Result<ResultAndState, EVMError<<DB as Database>::Error>>,
) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>;

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<DB: Database> {
    // Uses env, call result and returned gas from the call to determine the gas
    // that is returned from transaction execution..
    pub call_return: CallReturnHandle,
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<DB>,
    /// Calculate gas refund for transaction.
    /// Some chains have it disabled.
    pub calculate_gas_refund: CalculateGasRefundHandle,
    /// Main return handle, returns the output of the transact.
    pub main_return: MainReturnHandle<DB>,
    /// End handle.
    pub end: EndHandle<DB>,
}

impl<DB: Database> Handler<DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec>() -> Self {
        Self {
            call_return: mainnet::handle_call_return::<SPEC>,
            calculate_gas_refund: mainnet::calculate_gas_refund::<SPEC>,
            reimburse_caller: mainnet::handle_reimburse_caller::<SPEC, DB>,
            reward_beneficiary: mainnet::reward_beneficiary::<SPEC, DB>,
            main_return: mainnet::main_return::<DB>,
            end: mainnet::end_handle::<DB>,
        }
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    pub fn call_return(&self, env: &Env, call_result: ExitCode, returned_gas: Gas) -> Gas {
        (self.call_return)(env, call_result, returned_gas)
    }

    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        data: &mut EVMData<'_, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reimburse_caller)(data, gas)
    }

    /// Calculate gas refund for transaction. Some chains have it disabled.
    pub fn calculate_gas_refund(&self, env: &Env, gas: &Gas) -> u64 {
        (self.calculate_gas_refund)(env, gas)
    }

    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        data: &mut EVMData<'_, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.reward_beneficiary)(data, gas)
    }

    /// Main return.
    pub fn main_return(
        &self,
        data: &mut EVMData<'_, DB>,
        call_result: ExitCode,
        output: Output,
        gas: &Gas,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.main_return)(data, call_result, output, gas)
    }

    /// End handler.
    pub fn end(
        &self,
        data: &mut EVMData<'_, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.end)(data, end_output)
    }
}
