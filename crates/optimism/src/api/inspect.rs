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

use crate::{
    context::OpContext,
    handler::{precompiles::OpPrecompileProvider, OpHandler},
    transaction::{abstraction::OpTxGetter, OpTxTrait},
    L1BlockInfoGetter, OpSpec, OpTransactionError, OptimismHaltReason,
};

// pub trait InspectOpEvm<CTX, INTR: InterpreterTypes>: ExecuteOpEvm {
//     fn inspect<'a, 'b, INSP>(&'a mut self, tx: Self::Transaction, inspector: INSP) -> Self::Output
//     where
//         INSP: Inspector<&'a mut Self, INTR> + 'b,
//     {
//         self.set_tx(tx);
//         self.inspect_previous(inspector)
//     }

//     /// Drawback if inspector overlives the context it will take the mutable reference
//     /// of it and inspector needs to be dropped to release the mutable reference.
//     fn inspect_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::Output
//     where
//         INSP: Inspector<&'a mut Self, INTR> + 'b;
// }

impl<
        BLOCK: Block,
        TX: OpTxTrait,
        CFG: Cfg<Spec = crate::OpSpec>,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    > InspectEvm<&mut Self, EthInterpreter> for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn inspect_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::Output
    where
        INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
    {
        let mut insp = InspectorContext::new(self, inspector);
        inspect_op(&mut insp)
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
        TX: OpTxTrait,
        CFG: Cfg<Spec = OpSpec>,
        DB: Database + DatabaseCommit,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
    > InspectCommitEvm<&mut Self, EthInterpreter> for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
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
) -> Result<ResultAndState<OptimismHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>
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

// pub fn inspect_op<
//     CTX: EthContext + InspectorCtx<IT = EthInterpreter> + JournalExtGetter,
//     ERROR: EthError<CTX> + From<OpTransactionError> + IsTxError + FromStringError,
//     FRAME: Frame<
//             Context = CTX,
//             Error = ERROR,
//             FrameResult = FrameResult,
//             FrameInit = FrameInput,
//             FrameContext = FrameContext<
//                 OpPrecompileProvider<CTX, ERROR>,
//                 InspectorInstructionExecutor<EthInterpreter, CTX>,
//             >,
//         > + FrameInterpreterGetter<IT = EthInterpreter>,
//     HANDLER: EthHandler<
//             Context = CTX,
//             Error = ERROR,
//             Frame = FRAME,
//             Precompiles = OpPrecompileProvider<CTX, ERROR>,
//             HaltReason = OptimismHaltReason,
//         > + Default,
// >(
//     ctx: &mut CTX,
// ) -> Result<ResultAndState<OptimismHaltReason>, ERROR>
// where
//     <CTX as CfgGetter>::Cfg: Cfg<Spec = OpSpec>,
// {
//     //todo!();
//     let mut evm = InspectorHandlerImpl::<
//         CTX,
//         ERROR,
//         _,
//         _,
//         OpPrecompileProvider<CTX, ERROR>,
//         EthInterpreter,
//     >::new(HANDLER::default(), make_instruction_table());

//     evm.run(ctx)
// }

pub fn inspect_op_commit<DB: Database + DatabaseCommit, CTX>(
    ctx: &mut CTX,
) -> Result<
    ExecutionResult<OptimismHaltReason>,
    EVMError<<DB as Database>::Error, OpTransactionError>,
>
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
