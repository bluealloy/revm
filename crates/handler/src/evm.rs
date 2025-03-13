use crate::{
    instructions::{EthInstructions, InstructionProvider},
    EthFrame, Handler, MainnetHandler, PrecompileProvider,
};
use auto_impl::auto_impl;
use context::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, ContextSetters, ContextTr, Database, Evm, JournalOutput, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{
    interpreter::EthInterpreter, Interpreter, InterpreterAction, InterpreterResult,
    InterpreterTypes,
};

/// Main trait that combines the context, instructions and precompiles and allows execution of interpreter.
/// 
/// 
#[auto_impl(&mut, Box)]
pub trait EvmTr {
    type Context: ContextTr;
    type Instructions: InstructionProvider;
    type Precompiles;

    /// Run the interpreter loop and returns the output that can be a return or a next action.
    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output;

    /// Get the context.
    fn ctx(&mut self) -> &mut Self::Context;

    /// Get the context reference.
    fn ctx_ref(&self) -> &Self::Context;

    /// Get the context and instructions.
    ///
    /// This allows us to obtains mutable references to both context and instructions.
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions);

    /// Get the context and precompiles.
    ///
    /// This allows us to obtains mutable references to both context and precompiles.
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles);
}

/// Execute EVM transactions. Main trait for transaction execution.
pub trait ExecuteEvm {
    /// Output of transaction execution.
    type Output;
    /// Transaction type.
    type Tx: Transaction;
    /// Block type.
    type Block: Block;

    /// Set the transaction.
    fn set_tx(&mut self, tx: Self::Tx);

    /// Set the block.
    fn set_block(&mut self, block: Self::Block);

    /// Transact the transaction that is set in the context.
    fn replay(&mut self) -> Self::Output;

    /// Transact the given transaction.
    ///
    /// Internally sets transaction in context and use `replay` to execute the transaction.
    fn transact(&mut self, tx: Self::Tx) -> Self::Output {
        self.set_tx(tx);
        self.replay()
    }
}

/// Extension of the [`ExecuteEvm`] trait that adds a method that commits the state after execution.
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit output of transaction execution.
    type CommitOutput;

    /// Transact the transaction and commit to the state.
    fn replay_commit(&mut self) -> Self::CommitOutput;

    /// Transact the transaction and commit to the state.
    fn transact_commit(&mut self, tx: Self::Tx) -> Self::CommitOutput {
        self.set_tx(tx);
        self.replay_commit()
    }
}

impl<CTX, INSP, I, P> EvmTr for Evm<CTX, INSP, I, P>
where
    CTX: ContextTr,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;

    #[inline]
    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.data.ctx;
        let instructions = &mut self.instruction;
        interpreter.run_plain(instructions.instruction_table(), context)
    }
    #[inline]
    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.data.ctx
    }

    #[inline]
    fn ctx_ref(&self) -> &Self::Context {
        &self.data.ctx
    }

    #[inline]
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.data.ctx, &mut self.instruction)
    }

    #[inline]
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.data.ctx, &mut self.precompiles)
    }
}

impl<CTX, INSP, PRECOMPILES> ExecuteEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>> + ContextSetters,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Output = Result<
        ResultAndState<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn replay(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self)
    }

    fn set_tx(&mut self, tx: Self::Tx) {
        self.data.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.data.ctx.set_block(block);
    }
}

impl<CTX, INSP, PRECOMPILES> ExecuteCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>, Db: DatabaseCommit>
        + ContextSetters,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type CommitOutput = Result<
        ExecutionResult<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}
