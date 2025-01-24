use crate::{
    inspector_context::InspectorContext,
    inspector_instruction::InspectorInstructionExecutor,
    journal::{JournalExt, JournalExtGetter},
    Inspector, InspectorCtx, InspectorHandlerImpl,
};
use revm::{
    context::Cfg,
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
        Block, DatabaseGetter, Journal, Transaction,
    },
    database_interface::Database,
    handler::{
        handler::{EthContext, EthHandler, MainnetHandler},
        EthFrame,
    },
    interpreter::{interpreter::EthInterpreter, table::make_instruction_table, InterpreterTypes},
    primitives::Log,
    state::EvmState,
    Context, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};
use std::vec::Vec;

pub trait InspectEvm<CTX, INTR: InterpreterTypes>: ExecuteEvm {
    fn inspect<'a, 'b, INSP>(&'a mut self, tx: Self::Transaction, inspector: INSP) -> Self::Output
    where
        INSP: Inspector<&'a mut Self, INTR> + 'b,
    {
        self.set_tx(tx);
        self.inspect_previous(inspector)
    }

    /// Drawback if inspector overlives the context it will take the mutable reference
    /// of it and inspector needs to be dropped to release the mutable reference.
    fn inspect_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::Output
    where
        INSP: Inspector<&'a mut Self, INTR> + 'b;
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
        CHAIN,
    > InspectEvm<&mut Self, EthInterpreter> for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    fn inspect_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::Output
    where
        INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
    {
        let mut insp = InspectorContext::new(self, inspector);
        inspect_main(&mut insp)
    }
}

pub trait InspectCommitEvm<CTX, INTR: InterpreterTypes>:
    InspectEvm<CTX, INTR> + ExecuteCommitEvm
{
    fn inspect_commit<'a, 'b, INSP>(
        &'a mut self,
        tx: Self::Transaction,
        inspector: INSP,
    ) -> Self::CommitOutput
    where
        INSP: Inspector<&'a mut Self, INTR> + 'b,
    {
        self.set_tx(tx);
        self.inspect_commit_previous(inspector)
    }

    fn inspect_commit_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::CommitOutput
    where
        INSP: Inspector<&'a mut Self, INTR> + 'b;
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database + DatabaseCommit,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
        CHAIN,
    > InspectCommitEvm<&mut Self, EthInterpreter> for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    fn inspect_commit_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::CommitOutput
    where
        INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
    {
        let mut insp = InspectorContext::new(self, inspector);
        inspect_main_commit(&mut insp)
    }
}

pub fn inspect_main<
    DB: Database,
    CTX: EthContext
        + JournalExtGetter
        + DatabaseGetter<Database = DB>
        + InspectorCtx<IT = EthInterpreter>,
>(
    ctx: &mut CTX,
) -> Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    InspectorHandlerImpl::<_, _, EthFrame<_, _, _, _>, _, _, EthInterpreter>::new(
        MainnetHandler::<_, _, _, _, InspectorInstructionExecutor<EthInterpreter, CTX>>::default(),
        make_instruction_table(),
    )
    .run(ctx)
}

pub fn inspect_main_commit<
    DB: Database + DatabaseCommit,
    CTX: EthContext
        + JournalExtGetter
        + DatabaseGetter<Database = DB>
        + InspectorCtx<IT = EthInterpreter>,
>(
    ctx: &mut CTX,
) -> Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    inspect_main(ctx).map(|res| {
        ctx.db().commit(res.state);
        res.result
    })
}
