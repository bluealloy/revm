use crate::{evm::MyEvm, handler::MyHandler};
use revm::{
    context::{
        result::{HaltReason, InvalidTransaction, ResultAndState},
        ContextSetters,
    },
    context_interface::{
        result::{EVMError, ExecutionResult},
        ContextTr, Database, JournalTr,
    },
    handler::{EvmTr, Handler},
    inspector::{InspectCommitEvm, InspectEvm, Inspector, InspectorHandler, JournalExt},
    interpreter::interpreter::EthInterpreter,
    state::EvmState,
    DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

/// Type alias for the error type of the OpEvm.
type MyError<CTX> = EVMError<<<CTX as ContextTr>::Db as Database>::Error, InvalidTransaction>;

// Trait that allows to replay and transact the transaction.
impl<CTX, INSP> ExecuteEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<State = EvmState>>,
{
    type State = EvmState;
    type ExecutionResult = ExecutionResult<HaltReason>;
    type Error = MyError<CTX>;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn set_block(&mut self, block: Self::Block) {
        self.0.ctx.set_block(block);
    }

    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut handler = MyHandler::default();
        handler.run(self)
    }

    fn finalize(&mut self) -> Self::State {
        self.ctx().journal().finalize()
    }

    fn replay(
        &mut self,
    ) -> Result<ResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let mut handler = MyHandler::default();
        handler.run(self).map(|result| {
            let state = self.finalize();
            ResultAndState::new(result, state)
        })
    }
}

// Trait allows replay_commit and transact_commit functionality.
impl<CTX, INSP> ExecuteCommitEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Db: DatabaseCommit, Journal: JournalTr<State = EvmState>>,
{
    fn commit(&mut self, state: Self::State) {
        self.ctx().db().commit(state);
    }
}

// Inspection trait.
impl<CTX, INSP> InspectEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<State = EvmState> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.0.ctx.set_tx(tx);
        let mut handler = MyHandler::default();
        handler.inspect_run(self)
    }
}

// Inspect
impl<CTX, INSP> InspectCommitEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Db: DatabaseCommit, Journal: JournalTr<State = EvmState> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
}
