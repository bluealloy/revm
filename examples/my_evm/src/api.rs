use crate::{evm::MyEvm, handler::MyHandler};
use revm::{
    context::{
        result::{HaltReason, InvalidTransaction},
        ContextSetters, JournalOutput,
    },
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        ContextTr, Database, JournalTr,
    },
    handler::{EvmTr, Handler},
    inspector::{InspectCommitEvm, InspectEvm, Inspector, InspectorHandler, JournalExt},
    interpreter::interpreter::EthInterpreter,
    DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

/// Type alias for the error type of the OpEvm.
type MyError<CTX> = EVMError<<<CTX as ContextTr>::Db as Database>::Error, InvalidTransaction>;

// Trait that allows to replay and transact the transaction.
impl<CTX, INSP> ExecuteEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<FinalOutput = JournalOutput>>,
{
    type Output = Result<ResultAndState, MyError<CTX>>;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.0.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.0.ctx.set_block(block);
    }

    fn replay(&mut self) -> Self::Output {
        MyHandler::default().run(self)
    }
}

// Trait allows replay_commit and transact_commit functionality.
impl<CTX, INSP> ExecuteCommitEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Db: DatabaseCommit, Journal: JournalTr<FinalOutput = JournalOutput>>,
{
    type CommitOutput = Result<ExecutionResult<HaltReason>, MyError<CTX>>;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

// Inspection trait.
impl<CTX, INSP> InspectEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_replay(&mut self) -> Self::Output {
        MyHandler::default().inspect_run(self)
    }
}

// Inspect
impl<CTX, INSP> InspectCommitEvm for MyEvm<CTX, INSP>
where
    CTX: ContextSetters<
        Db: DatabaseCommit,
        Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt,
    >,
    INSP: Inspector<CTX, EthInterpreter>,
{
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput {
        self.inspect_replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}
