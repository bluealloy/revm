//! Mainnet related handlers.

mod execution;
mod frame;
mod frame_data;
mod post_execution;
mod pre_execution;
mod validation;

// Public exports

pub use execution::EthExecution;
pub use frame::{return_create, return_eofcreate, EthFrame};
pub use frame_data::{FrameData, FrameResult};
use interpreter::Host;
pub use post_execution::EthPostExecution;
pub use pre_execution::{apply_eip7702_auth_list, /*load_precompiles,*/ EthPreExecution};
use precompile::PrecompileErrors;
use primitives::Log;
use state::EvmState;
pub use validation::{
    validate_eip4844_tx, validate_initial_tx_gas, validate_priority_fee_tx,
    validate_tx_against_account, validate_tx_env, EthValidation,
};

// Imports

use crate::handler::{
    ExecutionWire, Frame, FrameOrResultGen, Handler, PostExecutionWire, PreExecutionWire,
    ValidationWire,
};
use context::{BlockGetter, CfgGetter, ErrorGetter, JournalStateGetter, JournalStateGetterDBError, TransactionGetter};
use wiring::{journaled_state::JournaledState, result::{HaltReason, InvalidHeader, InvalidTransaction}};

/// TODO Halt needs to be generalized.
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
    _phantom: std::marker::PhantomData<fn() -> (CTX, ERROR)>,
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
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC> Handler
    for EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<Journal: JournaledState<FinalOutput = (EvmState, Vec<Log>)>>
        + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
    VAL: ValidationWire,
    PREEXEC: PreExecutionWire,
    EXEC: ExecutionWire,
    POSTEXEC: PostExecutionWire,
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
