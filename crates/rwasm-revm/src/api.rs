use crate::{frame::RwasmFrame, RwasmEvm, RwasmSpecId};
use fluentbase_sdk::{Address, Bytes};
use revm::{
    context::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
        Block,
        BlockEnv,
        Cfg,
        CfgEnv,
        ContextSetters,
        ContextTr,
        JournalOutput,
        JournalTr,
        Transaction,
        TxEnv,
    },
    database::EmptyDB,
    handler::{
        instructions::{EthInstructions, InstructionProvider},
        EvmTr,
        Handler,
        MainnetHandler,
        PrecompileProvider,
        SystemCallTx,
    },
    inspector::{InspectorHandler, JournalExt},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    primitives::hardfork::SpecId,
    Context,
    Database,
    DatabaseCommit,
    ExecuteCommitEvm,
    ExecuteEvm,
    InspectCommitEvm,
    InspectEvm,
    Inspector,
    Journal,
    MainContext,
    SystemCallEvm,
};

pub trait RwasmContextTr:
    ContextTr<
    Journal: JournalTr<FinalOutput = JournalOutput>,
    Tx: Transaction,
    Cfg: Cfg<Spec = RwasmSpecId>,
>
{
}

impl<T> RwasmContextTr for T where
    T: ContextTr<
        Journal: JournalTr<FinalOutput = JournalOutput>,
        Tx: Transaction,
        Cfg: Cfg<Spec = RwasmSpecId>,
    >
{
}

impl<CTX, INSP, INST, PRECOMPILE> ExecuteEvm for RwasmEvm<CTX, INSP, INST, PRECOMPILE>
where
    CTX: RwasmContextTr + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Output = Result<
        ResultAndState<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn replay(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, RwasmFrame<_, _, _>>::default();
        t.run(self)
    }

    fn set_tx(&mut self, tx: Self::Tx) {
        self.0.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.0.set_block(block);
    }
}

impl<CTX, INSP, PRECOMPILE> ExecuteCommitEvm
    for RwasmEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: RwasmContextTr<Db: DatabaseCommit> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type CommitOutput = Result<
        ExecutionResult<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<CTX, INSP, PRECOMPILE> InspectEvm
    for RwasmEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: RwasmContextTr<Journal: JournalExt> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_replay(&mut self) -> Self::Output {
        let mut h = MainnetHandler::<_, _, RwasmFrame<_, _, _>>::default();
        h.inspect_run(self)
    }
}

impl<CTX, INSP, PRECOMPILE> InspectCommitEvm
    for RwasmEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: RwasmContextTr<Journal: JournalExt, Db: DatabaseCommit> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput {
        self.inspect_replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<CTX, INSP, PRECOMPILE> SystemCallEvm
    for RwasmEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: RwasmContextTr<Tx: SystemCallTx> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self::Output {
        self.set_tx(CTX::Tx::new_system_tx(data, system_contract_address));
        let mut h = MainnetHandler::<_, _, RwasmFrame<_, _, _>>::default();
        h.run_system_call(self)
    }
}

/// Trait that allows for optimism OpEvm to be built.
pub trait RwasmBuilder: Sized {
    /// Type of the context.
    type Context;

    /// Build the op.
    fn build_rwasm(
        self,
    ) -> RwasmEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>>;

    /// Build the op with an inspector.
    fn build_rwasm_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> RwasmEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> RwasmBuilder
    for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg<Spec = RwasmSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput>,
{
    type Context = Self;

    fn build_rwasm(
        self,
    ) -> RwasmEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>> {
        RwasmEvm::new(self, ())
    }

    fn build_rwasm_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> RwasmEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>> {
        RwasmEvm::new(self, inspector)
    }
}

/// Type alias for the default context type of the RwasmEvm.
pub type RwasmContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB, Journal<DB>, ()>;

/// Trait that allows for a default context to be created.
pub trait DefaultOp {
    /// Create a default context.
    fn rwasm() -> RwasmContext<EmptyDB>;
}

impl DefaultOp for RwasmContext<EmptyDB> {
    fn rwasm() -> Self {
        Context::mainnet().with_cfg(CfgEnv::new_with_spec(SpecId::PRAGUE))
    }
}
