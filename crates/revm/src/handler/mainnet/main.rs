//! Handles related to the main function of the EVM.
//!
//! They handle initial setup of the EVM, call loop and the final return of the EVM

use crate::{
    interpreter::{return_ok, return_revert, Gas, InstructionResult, SuccessOrHalt},
    primitives::{
        db::Database, EVMError, Env, ExecutionResult, Output, ResultAndState, Spec, SpecId::LONDON,
        U256,
    },
    Context, EvmContext,
};

/// Main return handle, returns the output of the transaction.
#[inline]
pub fn main_return<EXT, DB: Database>(
    context: &mut Context<'_, EXT, DB>,
    call_result: InstructionResult,
    output: Output,
    gas: &Gas,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    // used gas with refund calculated.
    let gas_refunded = gas.refunded() as u64;
    let final_gas_used = gas.spend() - gas_refunded;

    // reset journal and return present state.
    let (state, logs) = context.evm.journaled_state.finalize();

    let result = match call_result.into() {
        SuccessOrHalt::Success(reason) => ExecutionResult::Success {
            reason,
            gas_used: final_gas_used,
            gas_refunded,
            logs,
            output,
        },
        SuccessOrHalt::Revert => ExecutionResult::Revert {
            gas_used: final_gas_used,
            output: match output {
                Output::Call(return_value) => return_value,
                Output::Create(return_value, _) => return_value,
            },
        },
        SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
            reason,
            gas_used: final_gas_used,
        },
        SuccessOrHalt::FatalExternalError => {
            return Err(EVMError::Database(context.evm.error.take().unwrap()));
        }
        // Only two internal return flags.
        SuccessOrHalt::InternalContinue | SuccessOrHalt::InternalCallOrCreate => {
            panic!("Internal return flags should remain internal {call_result:?}")
        }
    };

    Ok(ResultAndState { result, state })
}

/// Mainnet end handle does not change the output.
#[inline]
pub fn end_handle<EXT, DB: Database>(
    _context: &mut Context<'_, EXT, DB>,
    evm_output: Result<ResultAndState, EVMError<DB::Error>>,
) -> Result<ResultAndState, EVMError<DB::Error>> {
    evm_output
}
