use crate::handler::Erc20MainetHandler;
use revm::{
    context::JournalOutput,
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
        ContextTr, JournalTr,
    },
    database_interface::DatabaseCommit,
    handler::{
        instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, Handler,
        PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    precompile::PrecompileError,
};

pub type EVMErrorForContext<CTX, PRECOMPILE, TransactionError> =
    EVMError<ContextTrDbError<CTX>, PRECOMPILE, TransactionError>;

pub fn transact_erc20evm<EVM>(
    evm: &mut EVM,
) -> Result<
    ResultAndState<HaltReason>,
    EVMErrorForContext<EVM::Context, PrecompileError, InvalidTransaction>,
>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    Erc20MainetHandler::<EVM, _, EthFrame<EVM, _, EthInterpreter>>::new().run(evm)
}

pub fn transact_erc20evm_commit<EVM>(
    evm: &mut EVM,
) -> Result<
    ExecutionResult<HaltReason>,
    EVMErrorForContext<EVM::Context, PrecompileError, InvalidTransaction>,
>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>, Db: DatabaseCommit>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    transact_erc20evm(evm).map(|r| {
        evm.ctx().db().commit(r.state);
        r.result
    })
}
