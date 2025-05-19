use crate::handler::Erc20MainnetHandler;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction},
        ContextTr, JournalTr,
    },
    database_interface::DatabaseCommit,
    handler::{
        instructions::InstructionProvider, ContextTrDbError, EthFrame, EvmTr, Handler,
        PrecompileProvider,
    },
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    state::EvmState,
};

pub fn transact_erc20evm<EVM>(
    evm: &mut EVM,
) -> Result<
    (ExecutionResult<HaltReason>, EvmState),
    EVMError<ContextTrDbError<EVM::Context>, InvalidTransaction>,
>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    let mut handler = Erc20MainnetHandler::<EVM, _, EthFrame<EVM, _, EthInterpreter>>::new();
    handler.run(evm).and_then(|r| {
        let state = evm.ctx().journal().finalize();
        Ok((r, state))
    })
}

pub fn transact_erc20evm_commit<EVM>(
    evm: &mut EVM,
) -> Result<ExecutionResult<HaltReason>, EVMError<ContextTrDbError<EVM::Context>, InvalidTransaction>>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
    >,
{
    transact_erc20evm(evm).map(|(result, state)| {
        evm.ctx().db().commit(state);
        result
    })
}
