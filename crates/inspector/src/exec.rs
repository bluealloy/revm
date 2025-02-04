use crate::{
    inspector_context::{InspectorContext, InspectorInnerCtx},
    journal::{JournalExt, JournalExtGetter},
    Inspector, InspectorHandlerImpl,
};
use revm::{
    context::{Cfg, MEVM},
    context_interface::{
        result::{EVMError, HaltReason, InvalidTransaction, ResultAndState},
        Block, CfgGetter, DatabaseGetter, Journal, Transaction,
    },
    database_interface::Database,
    handler::{
        handler::{EthContext, EthHandler, FrameContextTrait, MainnetHandler},
        instructions::{EthInstructionExecutor, InstructionExecutor},
        EthFrame, EthPrecompileProvider, FrameContext, PrecompileProvider,
    },
    interpreter::{
        interpreter::EthInterpreter, table::InstructionTable, Host, Interpreter, InterpreterAction,
        InterpreterTypes,
    },
    precompile::PrecompileErrors,
    primitives::{Address, Bytes, Log},
    state::EvmState,
    Context, ExecuteEvm,
};
use std::rc::Rc;

// pub trait InspectEvm<INTR: InterpreterTypes, CTX>: ExecuteEvm {
//     fn inspect<'a, INSP>(&'a mut self, tx: Self::Transaction, inspector: INSP) -> Self::Output
//     where
//         INSP: Inspector<&'a mut CTX, INTR> + 'a,
//         CTX: 'a,
//     {
//         self.set_tx(tx);
//         self.inspect_previous(inspector)
//     }

//     /// Drawback if inspector overlives the context it will take the mutable reference
//     /// of it and inspector needs to be dropped to release the mutable reference.
//     fn inspect_previous<'a, INSP>(&'a mut self, inspector: INSP) -> Self::Output
//     where
//         INSP: Inspector<&'a mut CTX, INTR> + 'a,
//         CTX: 'a;
// }

// impl<
//         BLOCK: Block,
//         TX: Transaction,
//         CFG: Cfg,
//         DB: Database,
//         JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
//         CHAIN,
//         INSP,
//     > InspectEvm<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>
//     for MEVM<
//         Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
//         FrameContext<
//             EthPrecompileProvider<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//             EthInstructionExecutor<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>,
//         >,
//         INSP,
//     >
// {
//     fn inspect_previous(&mut self, inspector: INSP) -> Self::Output {
//         inspect_main(&mut self.ctx, &mut self.frame_ctx, inspector)
//     }
// }

// pub trait InspectCommitEvm<INTR: InterpreterTypes>: InspectEvm<INTR> + ExecuteCommitEvm {
//     fn inspect_commit<'a, 'b, INSP>(
//         &'a mut self,
//         tx: Self::Transaction,
//         inspector: INSP,
//     ) -> Self::CommitOutput
//     where
//         INSP: Inspector<&'a mut Self, INTR> + 'b,
//     {
//         self.set_tx(tx);
//         self.inspect_commit_previous(inspector)
//     }

//     fn inspect_commit_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::CommitOutput
//     where
//         INSP: Inspector<&'a mut Self, INTR> + 'b;
// }

// impl<
//         BLOCK: Block,
//         TX: Transaction,
//         CFG: Cfg,
//         DB: Database + DatabaseCommit,
//         JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)> + JournalExt,
//         CHAIN,
//     > InspectCommitEvm<EthInterpreter> for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
// {
//     fn inspect_commit_previous<'a, 'b, INSP>(&'a mut self, inspector: INSP) -> Self::CommitOutput
//     where
//         INSP: Inspector<&'a mut Self, EthInterpreter> + 'b,
//     {
//         let mut insp = InspectorContext::new(self, inspector);
//         inspect_main_commit(&mut insp)
//     }
// }

pub fn on_memv<CTX, INSP>(
    evm: &mut MEVM<
        CTX,
        FrameContext<EthPrecompileProvider<CTX>, EthInstructionExecutor<EthInterpreter, CTX>>,
        INSP,
    >,
) -> Result<
    ResultAndState<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
>
where
    CTX: EthContext + JournalExtGetter,
    INSP: Inspector<CTX, EthInterpreter>,
{
    let (ctx, framectx) = evm.all_mut();

    let re = inspect_main(ctx, framectx, inspector);

    re
}

pub fn inspect_main<'a, CTX, INSP>(
    ctx: &'a mut CTX,
    frame_context: &mut FrameContext<
        EthPrecompileProvider<CTX>,
        EthInstructionExecutor<EthInterpreter, CTX>,
    >,
    inspector: INSP,
) -> Result<
    ResultAndState<HaltReason>,
    EVMError<<<CTX as DatabaseGetter>::Database as Database>::Error, InvalidTransaction>,
>
where
    CTX: EthContext + JournalExtGetter + 'a,
    INSP: Inspector<CTX, EthInterpreter>,
{
    //let t = transact_main(evm);
    let mut insp = InspectorHandlerImpl::<_, _, _, _, _>::new(MainnetHandler::<
        _,
        _,
        EthFrame<_, _, _, InspectorFrameContext<_, _>>,
        _,
    > {
        _phantom: core::marker::PhantomData,
    });

    let mut insp_ctx = InspectorContext::new(ctx, inspector);
    let mut inspector_frame_ctx = InspectorFrameContext::new(frame_context);

    insp.run_split(&mut insp_ctx, &mut inspector_frame_ctx)
}

pub trait InstructionGetter {
    type Context: Host;
    type IT: InterpreterTypes;

    fn get_instruction(&self) -> Rc<InstructionTable<Self::IT, Self::Context>>;
}

impl<CTX: Host, INTR: InterpreterTypes> InstructionGetter for EthInstructionExecutor<INTR, CTX> {
    type Context = CTX;
    type IT = INTR;
    fn get_instruction(&self) -> Rc<InstructionTable<Self::IT, Self::Context>> {
        self.instruction_table.clone()
    }
}

pub struct InspectorFrameContext<
    CTX: InspectorInnerCtx,
    FRAMECTX: FrameContextTrait<
        Instructions: InstructionGetter<Context = <CTX as InspectorInnerCtx>::Context>,
    >,
> {
    pub frame_ctx: FRAMECTX,
    pub table: InstructionEXEC<CTX, <FRAMECTX::Instructions as InstructionGetter>::IT>,
    pub precompiles: InspectorPrecompile<CTX, FRAMECTX::Precompiles>,
    pub _phantom: core::marker::PhantomData<CTX>,
}

impl<
        CTX: InspectorInnerCtx,
        FRAMECTX: FrameContextTrait<
            Instructions: InstructionExecutor
                              + InstructionGetter<Context = <CTX as InspectorInnerCtx>::Context>,
        >,
    > InspectorFrameContext<CTX, FRAMECTX>
{
    pub fn new(mut frame_ctx: FRAMECTX) -> Self {
        let instruction_table = frame_ctx.instructions().get_instruction();
        let table = InstructionEXEC {
            instruction_table,
            _phantom: core::marker::PhantomData,
        };

        let precompiles = InspectorPrecompile {
            // TODO need a comment in PrecompileProvider to explain that clonining
            // should be very cheap, it is already done as static inside OP.
            precompile: frame_ctx.precompiles().clone(),
            _phantom: core::marker::PhantomData,
        };
        Self {
            frame_ctx,
            table,
            precompiles,
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct InstructionEXEC<CTX: InspectorInnerCtx, WIRE: InterpreterTypes> {
    instruction_table: Rc<InstructionTable<WIRE, CTX::Context>>,
    _phantom: core::marker::PhantomData<CTX>,
}

impl<CTX: InspectorInnerCtx, IT: InterpreterTypes> InstructionExecutor for InstructionEXEC<CTX, IT>
where
    CTX::Context: Host,
{
    type InterpreterTypes = IT;
    type CTX = CTX;
    type Output = InterpreterAction;

    fn run(
        &mut self,
        context: &mut Self::CTX,
        interpreter: &mut Interpreter<Self::InterpreterTypes>,
    ) -> Self::Output {
        // TODO need to copy code from InspectorInstructionExecutor to here.
        interpreter.run_plain(self.instruction_table.as_ref(), context.inner_ctx())
    }
}

pub struct InspectorPrecompile<CTX, P> {
    precompile: P,
    _phantom: core::marker::PhantomData<CTX>,
}

impl<CTX, P: Clone> Clone for InspectorPrecompile<CTX, P> {
    fn clone(&self) -> Self {
        Self {
            precompile: self.precompile.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<CTX, P> PrecompileProvider for InspectorPrecompile<CTX, P>
where
    CTX: InspectorInnerCtx<Context = P::Context> + CfgGetter<Cfg = <P::Context as CfgGetter>::Cfg>,
    P: PrecompileProvider,
{
    type Context = CTX;
    type Output = P::Output;

    fn set_spec(&mut self, spec: <<Self::Context as CfgGetter>::Cfg as Cfg>::Spec) {
        self.precompile.set_spec(spec);
    }

    fn run(
        &mut self,
        context: &mut Self::Context,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, PrecompileErrors> {
        self.precompile
            .run(context.inner_ctx(), address, bytes, gas_limit)
    }
    /// Get the warm addresses.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address> + '_> {
        self.precompile.warm_addresses()
    }

    /// Check if the address is a precompile.
    fn contains(&self, address: &Address) -> bool {
        self.precompile.contains(address)
    }
}

impl<CTX, FRAMECTX> FrameContextTrait for InspectorFrameContext<CTX, FRAMECTX>
where
    CTX: InspectorInnerCtx<Context: Host> + CfgGetter<Cfg = <FRAMECTX::Context as CfgGetter>::Cfg>, //CfgGetter<Cfg = <InspectorInnerCtx as CfgGetter>::Cfg> + ,
    FRAMECTX: FrameContextTrait<
        Context = <CTX as InspectorInnerCtx>::Context,
        Instructions: InstructionExecutor
                          + InstructionGetter<Context = <CTX as InspectorInnerCtx>::Context>,
    >,
{
    type Context = CTX;
    type Instructions = InstructionEXEC<CTX, <FRAMECTX::Instructions as InstructionGetter>::IT>;
    type Precompiles = InspectorPrecompile<CTX, FRAMECTX::Precompiles>;

    fn instructions(&mut self) -> &mut Self::Instructions {
        &mut self.table
    }

    fn precompiles(&mut self) -> &mut Self::Precompiles {
        &mut self.precompiles
    }
}

/*
let mut evm = Context::mainnet().build_main();

let o = evm.exec().unwrap();

let o = evm.inspect_previous(&mut Inspector::new()).unwrap();

*/

// pub fn inspect_main<
//     DB: Database,
//     CTX: EthContext
//         + JournalExtGetter
//         + DatabaseGetter<Database = DB>
//         + InspectorCtx<IT = EthInterpreter>,
// >(
//     ctx: &mut CTX,
// ) -> Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
//     InspectorHandlerImpl::<_, _, EthFrame<_, _, _, _>, _, _, EthInterpreter>::new(
//         MainnetHandler::<_, _, _, _, InspectorInstructionExecutor<EthInterpreter, CTX>>::default(),
//         make_instruction_table(),
//     )
//     .run(ctx)
// }

// pub fn inspect_main_commit<
//     DB: Database + DatabaseCommit,
//     CTX: EthContext
//         + JournalExtGetter
//         + DatabaseGetter<Database = DB>
//         + InspectorCtx<IT = EthInterpreter>,
// >(
//     ctx: &mut CTX,
// ) -> Result<ExecutionResult<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
//     inspect_main(ctx).map(|res| {
//         ctx.db().commit(res.state);
//         res.result
//     })
// }
