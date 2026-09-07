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

type Erc20Error<CTX> = EVMError<ContextTrDbError<CTX>, InvalidTransaction>;

/// Executes a transaction using ERC20 tokens for gas payment.
/// Returns the execution result and the finalized state changes.
/// This function does not commit the state to the database.
pub fn transact_erc20evm<EVM>(
    evm: &mut EVM,
) -> Result<(ExecutionResult<HaltReason>, EvmState), Erc20Error<EVM::Context>>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
        Frame = EthFrame<EthInterpreter>,
    >,
{
    Erc20MainnetHandler::new().run(evm).map(|r| {
        let state = evm.ctx().journal_mut().finalize();
        (r, state)
    })
}

/// Executes a transaction using ERC20 tokens for gas payment and commits the state.
/// This is a convenience function that runs the transaction and immediately
/// commits the resulting state changes to the database.
pub fn transact_erc20evm_commit<EVM>(
    evm: &mut EVM,
) -> Result<ExecutionResult<HaltReason>, Erc20Error<EVM::Context>>
where
    EVM: EvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit>,
        Precompiles: PrecompileProvider<EVM::Context, Output = InterpreterResult>,
        Instructions: InstructionProvider<
            Context = EVM::Context,
            InterpreterTypes = EthInterpreter,
        >,
        Frame = EthFrame<EthInterpreter>,
    >,
{
    transact_erc20evm(evm).map(|(result, state)| {
        evm.ctx().db_mut().commit(state);
        result
    })
}
