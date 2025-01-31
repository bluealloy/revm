use crate::{
    context::OpContext,
    handler::{precompiles::OpPrecompileProvider, OpHandler},
    transaction::{abstraction::OpTxGetter, OpTxTrait},
    L1BlockInfoGetter, OpHaltReason, OpSpec, OpTransactionError,
};
use inspector::{
    exec::InspectEvm,
    inspector_context::InspectorContext,
    inspector_instruction::InspectorInstructionExecutor,
    journal::{JournalExt, JournalExtGetter},
    Inspector, InspectorCtx, InspectorHandlerImpl,
};
use revm::{
    context::Cfg,
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Block, CfgGetter, DatabaseGetter, Journal,
    },
    database_interface::Database,
    handler::{handler::EthContext, EthFrame, EthHandler, FrameContext},
    interpreter::{interpreter::EthInterpreter, table::make_instruction_table, InterpreterTypes},
    primitives::Log,
    state::EvmState,
    DatabaseCommit, ExecuteCommitEvm,
};
use std::vec::Vec;

impl<
        BLOCK: Block,
        TX: OpTxTrait,
        CFG: Cfg<Spec = crate::OpSpec>,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    > InspectEvm<EthInterpreter> for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn inspect_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::Output
    where
        INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
    {
        let mut insp = InspectorContext::new(self, inspector);
        inspect_op(&mut insp)
    }
}

pub trait InspectCommitEvm<INTR: InterpreterTypes>: InspectEvm<INTR> + ExecuteCommitEvm {
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
        TX: OpTxTrait,
        CFG: Cfg<Spec = OpSpec>,
        DB: Database + DatabaseCommit,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    > InspectCommitEvm<EthInterpreter> for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn inspect_commit_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::CommitOutput
    where
        INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
    {
        let mut insp = InspectorContext::new(self, inspector);
        inspect_op(&mut insp).map(|res| {
            insp.db().commit(res.state);
            res.result
        })
    }
}

pub fn inspect_op<DB, CTX>(
    ctx: &mut CTX,
) -> Result<ResultAndState<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>
where
    DB: Database,
    CTX: EthContext
        + OpTxGetter
        + L1BlockInfoGetter
        + JournalExtGetter
        + DatabaseGetter<Database = DB>
        + InspectorCtx<IT = EthInterpreter>,
    // Have Cfg with OpSpec
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
{
    InspectorHandlerImpl::<_, _, EthFrame<_, _, _, _>, _, _, EthInterpreter>::new(
        OpHandler::<
            CTX,
            _,
            EthFrame<
                CTX,
                _,
                _,
                FrameContext<
                    OpPrecompileProvider<CTX, _>,
                    InspectorInstructionExecutor<EthInterpreter, CTX>,
                >,
            >,
            //+ FrameInterpreterGetter<IT = INTR>,
            OpPrecompileProvider<CTX, EVMError<<DB as Database>::Error, OpTransactionError>>,
            InspectorInstructionExecutor<EthInterpreter, CTX>,
        >::default(),
        make_instruction_table(),
    )
    .run(ctx)
}

pub fn inspect_op_commit<DB: Database + DatabaseCommit, CTX>(
    ctx: &mut CTX,
) -> Result<ExecutionResult<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>
where
    CTX: EthContext
        + OpTxGetter
        + JournalExtGetter
        + DatabaseGetter<Database = DB>
        + InspectorCtx<IT = EthInterpreter>
        + L1BlockInfoGetter,
    // Have Cfg with OpSpec
    <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
{
    inspect_op(ctx).map(|res| {
        ctx.db().commit(res.state);
        res.result
    })
}
