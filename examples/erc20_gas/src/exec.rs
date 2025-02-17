use crate::handler::Erc20MainetHandler;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
        ContextTr, Journal,
    },
    database_interface::DatabaseCommit,
    handler::{
        instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, Handler,
        PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, InterpreterAction, InterpreterResult},
    primitives::Log,
    state::EvmState,
};

pub fn transact_erc20evm<EVM>(
    evm: &mut EVM,
) -> Result<ResultAndState<HaltReason>, EVMError<ContextTrDbError<EVM::Context>, InvalidTransaction>>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
        Precompiles: PrecompileProvider<Context = EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
            Output = InterpreterAction,
        >,
    >,
{
    Erc20MainetHandler::<EVM, _, EthFrame<EVM, _, EthInterpreter>>::new().run(evm)
}

pub fn transact_erc20evm_commit<EVM>(
    evm: &mut EVM,
) -> Result<ExecutionResult<HaltReason>, EVMError<ContextTrDbError<EVM::Context>, InvalidTransaction>>
where
    EVM: EvmTr<
        Context: ContextTr<
            Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>,
            Db: DatabaseCommit,
        >,
        Precompiles: PrecompileProvider<Context = EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
            Output = InterpreterAction,
        >,
    >,
{
    transact_erc20evm(evm).map(|r| {
        evm.ctx().db().commit(r.state);
        r.result
    })
}
