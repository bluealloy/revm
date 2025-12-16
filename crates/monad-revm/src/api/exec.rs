// ExecuteEvm implementations for MonadEvm.

use crate::{
    evm::MonadEvm,
    handler::MonadHandler,
    instructions::MonadInstructions,
    MonadSpecId,
};
use revm::{
    context::{result::ExecResultAndState, ContextSetters},
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason},
        Cfg, ContextTr, Database, JournalTr, Transaction,
    },
    handler::{
        system_call::SystemCallEvm, EthFrame, Handler, PrecompileProvider, SystemCallTx,
    },
    inspector::{
        InspectCommitEvm, InspectEvm, InspectSystemCallEvm, Inspector, InspectorHandler,
        JournalExt,
    },
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    primitives::{Address, Bytes},
    state::EvmState,
    DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

/// Trait alias for Monad context requirements.
pub trait MonadContextTr:
    ContextTr<Journal: JournalTr<State = EvmState>, Tx: Transaction, Cfg: Cfg<Spec = MonadSpecId>>
{
}

impl<T> MonadContextTr for T where
    T: ContextTr<Journal: JournalTr<State = EvmState>, Tx: Transaction, Cfg: Cfg<Spec = MonadSpecId>>
{
}

/// Type alias for MonadEvm error type.
pub type MonadError<CTX> =
    EVMError<<<CTX as ContextTr>::Db as Database>::Error, revm::context_interface::result::InvalidTransaction>;

impl<CTX, INSP, PRECOMPILE> ExecuteEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Tx = <CTX as ContextTr>::Tx;
    type Block = <CTX as ContextTr>::Block;
    type State = EvmState;
    type Error = MonadError<CTX>;
    type ExecutionResult = ExecutionResult<HaltReason>;

    fn set_block(&mut self, block: Self::Block) {
        self.0.ctx.set_block(block);
    }

    fn transact_one(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut h = MonadHandler::<_, _, EthFrame<EthInterpreter>>::new();
        h.run(self)
    }

    fn finalize(&mut self) -> Self::State {
        self.0.ctx.journal_mut().finalize()
    }

    fn replay(
        &mut self,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let mut h = MonadHandler::<_, _, EthFrame<EthInterpreter>>::new();
        h.run(self).map(|result| {
            let state = self.finalize();
            ExecResultAndState::new(result, state)
        })
    }
}

impl<CTX, INSP, PRECOMPILE> ExecuteCommitEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr<Db: DatabaseCommit> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn commit(&mut self, state: Self::State) {
        self.0.ctx.db_mut().commit(state);
    }
}

impl<CTX, INSP, PRECOMPILE> InspectEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr<Journal: JournalExt> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_one_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut h = MonadHandler::<_, _, EthFrame<EthInterpreter>>::new();
        h.inspect_run(self)
    }
}

impl<CTX, INSP, PRECOMPILE> InspectCommitEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr<Journal: JournalExt, Db: DatabaseCommit> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
}

impl<CTX, INSP, PRECOMPILE> SystemCallEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr<Tx: SystemCallTx> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn system_call_one_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(CTX::Tx::new_system_tx_with_caller(
            caller,
            system_contract_address,
            data,
        ));
        let mut h = MonadHandler::<_, _, EthFrame<EthInterpreter>>::new();
        h.run_system_call(self)
    }
}

impl<CTX, INSP, PRECOMPILE> InspectSystemCallEvm
    for MonadEvm<CTX, INSP, MonadInstructions<CTX>, PRECOMPILE>
where
    CTX: MonadContextTr<Journal: JournalExt, Tx: SystemCallTx> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn inspect_one_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(CTX::Tx::new_system_tx_with_caller(
            caller,
            system_contract_address,
            data,
        ));
        let mut h = MonadHandler::<_, _, EthFrame<EthInterpreter>>::new();
        h.inspect_run_system_call(self)
    }
}