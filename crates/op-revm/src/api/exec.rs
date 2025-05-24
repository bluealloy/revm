use crate::{
    evm::OpEvm, handler::OpHandler, transaction::OpTxTr, L1BlockInfo, OpHaltReason, OpSpecId,
    OpTransactionError,
};
use revm::{
    context::{result::ResultAndState, ContextSetters},
    context_interface::{
        result::{EVMError, ExecutionResult},
        Cfg, ContextTr, Database, JournalTr,
    },
    handler::{
        instructions::EthInstructions, system_call::SystemCallEvm, EthFrame, Handler,
        PrecompileProvider, SystemCallTx,
    },
    inspector::{InspectCommitEvm, InspectEvm, Inspector, InspectorHandler, JournalExt},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    primitives::{Address, Bytes},
    state::EvmState,
    DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

// Type alias for Optimism context
pub trait OpContextTr:
    ContextTr<
    Journal: JournalTr<State = EvmState>,
    Tx: OpTxTr,
    Cfg: Cfg<Spec = OpSpecId>,
    Chain = L1BlockInfo,
>
{
}

impl<T> OpContextTr for T where
    T: ContextTr<
        Journal: JournalTr<State = EvmState>,
        Tx: OpTxTr,
        Cfg: Cfg<Spec = OpSpecId>,
        Chain = L1BlockInfo,
    >
{
}

/// Type alias for the error type of the OpEvm.
type OpError<CTX> = EVMError<<<CTX as ContextTr>::Db as Database>::Error, OpTransactionError>;

impl<CTX, INSP, PRECOMPILE> ExecuteEvm
    for OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: OpContextTr + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Tx = <CTX as ContextTr>::Tx;
    type Block = <CTX as ContextTr>::Block;
    type State = EvmState;
    type Error = OpError<CTX>;
    type ExecutionResult = ExecutionResult<OpHaltReason>;

    fn set_block(&mut self, block: Self::Block) {
        self.0.ctx.set_block(block);
    }

    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }

    fn finalize(&mut self) -> Self::State {
        self.0.ctx.journal().finalize()
    }

    fn replay(
        &mut self,
    ) -> Result<ResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self).map(|result| {
            let state = self.finalize();
            ResultAndState::new(result, state)
        })
    }
}

impl<CTX, INSP, PRECOMPILE> ExecuteCommitEvm
    for OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: OpContextTr<Db: DatabaseCommit> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn commit(&mut self, state: Self::State) {
        self.0.ctx.db().commit(state);
    }
}

impl<CTX, INSP, PRECOMPILE> InspectEvm
    for OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: OpContextTr<Journal: JournalExt> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.inspect_run(self)
    }
}

impl<CTX, INSP, PRECOMPILE> InspectCommitEvm
    for OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: OpContextTr<Journal: JournalExt, Db: DatabaseCommit> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
}

impl<CTX, INSP, PRECOMPILE> SystemCallEvm
    for OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: OpContextTr<Tx: SystemCallTx> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.0
            .ctx
            .set_tx(CTX::Tx::new_system_tx(data, system_contract_address));
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run_system_call(self)
    }
}
