//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// Mainnet related handlers.

mod execution;
mod frame;
mod frame_data;
mod post_execution;
mod pre_execution;
mod precompile_provider;
mod validation;

// Public exports

pub use execution::{EthExecution, EthExecutionContext, EthExecutionError};
pub use frame::{return_create, return_eofcreate, EthFrame, EthFrameContext, EthFrameError};
pub use frame_data::{FrameData, FrameResult};
pub use post_execution::{EthPostExecution, EthPostExecutionContext, EthPostExecutionError};
pub use pre_execution::{
    apply_eip7702_auth_list, EthPreExecution, EthPreExecutionContext, EthPreExecutionError,
};
use precompile::PrecompileErrors;
pub use precompile_provider::EthPrecompileProvider;
use primitives::Log;
use state::EvmState;
use std::vec::Vec;
pub use validation::{
    validate_eip4844_tx, validate_initial_tx_gas, validate_priority_fee_tx,
    validate_tx_against_account, validate_tx_env, EthValidation, EthValidationContext,
    EthValidationError,
};

// Imports

use context_interface::{
    journaled_state::Journal,
    result::{HaltReason, InvalidHeader, InvalidTransaction},
};
use context_interface::{
    BlockGetter, CfgGetter, ErrorGetter, JournalGetter, JournalDBError,
    TransactionGetter,
};
use handler_interface::{
    ExecutionHandler, Handler, PostExecutionHandler, PreExecutionHandler, ValidationHandler,
};
use interpreter::Host;

#[derive(Default)]
pub struct EthHandler<
    CTX,
    ERROR,
    VAL = EthValidation<CTX, ERROR>,
    PREEXEC = EthPreExecution<CTX, ERROR>,
    EXEC = EthExecution<CTX, ERROR>,
    POSTEXEC = EthPostExecution<CTX, ERROR, HaltReason>,
> {
    pub validation: VAL,
    pub pre_execution: PREEXEC,
    pub execution: EXEC,
    pub post_execution: POSTEXEC,
    _phantom: core::marker::PhantomData<fn() -> (CTX, ERROR)>,
}

impl<CTX, ERROR> Default for EthHandler<CTX, ERROR> {
    fn default() -> Self {
        Self {
            validation: EthValidation::new(),
            pre_execution: EthPreExecution::new(),
            execution: EthExecution::new(),
            post_execution: EthPostExecution::new(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>
    EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>
{
    pub fn new(
        validation: VAL,
        pre_execution: PREEXEC,
        execution: EXEC,
        post_execution: POSTEXEC,
    ) -> Self {
        Self {
            validation,
            pre_execution,
            execution,
            post_execution,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC> Handler
    for EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalGetter<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>
        + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationHandler,
    PREEXEC: PreExecutionHandler,
    EXEC: ExecutionHandler,
    POSTEXEC: PostExecutionHandler,
{
    type Validation = VAL;
    type PreExecution = PREEXEC;
    type Execution = EXEC;
    type PostExecution = POSTEXEC;

    fn validation(&mut self) -> &mut Self::Validation {
        &mut self.validation
    }

    fn pre_execution(&mut self) -> &mut Self::PreExecution {
        &mut self.pre_execution
    }

    fn execution(&mut self) -> &mut Self::Execution {
        &mut self.execution
    }

    fn post_execution(&mut self) -> &mut Self::PostExecution {
        &mut self.post_execution
    }
}
