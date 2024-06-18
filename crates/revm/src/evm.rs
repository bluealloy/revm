use revm_interpreter::Host as _;

use crate::{
    builder::{EvmBuilder, HandlerStage, SetGenericStage},
    db::{Database, DatabaseCommit, EmptyDB},
    handler::{EnvWithChainSpec, Handler},
    interpreter::{CallInputs, CreateInputs, EOFCreateInputs, InterpreterAction, SharedMemory},
    primitives::{
        CfgEnv, ChainSpec, EVMError, EVMResult, EthChainSpec, ExecutionResult, ResultAndState,
        SpecId, Transaction as _, TransactionValidation, TxKind,
    },
    Context, ContextWithChainSpec, Frame, FrameOrResult, FrameResult,
};
use core::fmt::{self, Debug};
use std::{boxed::Box, vec::Vec};

/// EVM call stack limit.
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hardfork specification).
pub struct Evm<'a, ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<ChainSpecT, EXT, DB>,
    /// Handler is a component of the of EVM that contains all the logic. Handler contains specification id
    /// and it different depending on the specified fork.
    pub handler: Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>,
}

impl<ChainSpecT, EXT, DB> Debug for Evm<'_, ChainSpecT, EXT, DB>
where
    ChainSpecT: ChainSpec<Block: Debug, Transaction: Debug>,
    EXT: Debug,
    DB: Database<Error: Debug> + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EXT, ChainSpecT: ChainSpec, DB: Database + DatabaseCommit> Evm<'_, ChainSpecT, EXT, DB> {
    /// Commit the changes to the database.
    #[allow(clippy::type_complexity)]
    pub fn transact_commit(
        &mut self,
    ) -> Result<
        ExecutionResult<ChainSpecT>,
        EVMError<DB::Error, <ChainSpecT::Transaction as TransactionValidation>::ValidationError>,
    > {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.db.commit(state);
        Ok(result)
    }
}

impl<'a> Evm<'a, EthChainSpec, (), EmptyDB> {
    /// Returns evm builder with the mainnet chain spec, empty database, and empty external context.
    pub fn builder() -> EvmBuilder<'a, SetGenericStage, EthChainSpec, (), EmptyDB> {
        EvmBuilder::default()
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> Evm<'a, ChainSpecT, EXT, DB> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<ChainSpecT, EXT, DB>,
        handler: Handler<'a, ChainSpecT, Context<ChainSpecT, EXT, DB>, EXT, DB>,
    ) -> Evm<'a, ChainSpecT, EXT, DB> {
        context
            .evm
            .journaled_state
            .set_spec_id(handler.spec_id.into());
        Evm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> EvmBuilder<'a, HandlerStage, ChainSpecT, EXT, DB> {
        EvmBuilder::new(self)
    }

    /// Runs main call loop.
    #[inline]
    pub fn run_the_loop(
        &mut self,
        first_frame: Frame,
    ) -> Result<
        FrameResult,
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        let mut call_stack: Vec<Frame> = Vec::with_capacity(1025);
        call_stack.push(first_frame);

        #[cfg(feature = "memory_limit")]
        let mut shared_memory =
            SharedMemory::new_with_memory_limit(self.context.evm.env.cfg.memory_limit);
        #[cfg(not(feature = "memory_limit"))]
        let mut shared_memory = SharedMemory::new();

        shared_memory.new_context();

        // Peek the last stack frame.
        let mut stack_frame = call_stack.last_mut().unwrap();

        loop {
            // Execute the frame.
            let next_action =
                self.handler
                    .execute_frame(stack_frame, &mut shared_memory, &mut self.context)?;

            // Take error and break the loop, if any.
            // This error can be set in the Interpreter when it interacts with the context.
            self.context.evm.take_error().map_err(EVMError::Database)?;

            let exec = &mut self.handler.execution;
            let frame_or_result = match next_action {
                InterpreterAction::Call { inputs } => exec.call(&mut self.context, inputs)?,
                InterpreterAction::Create { inputs } => exec.create(&mut self.context, inputs)?,
                InterpreterAction::EOFCreate { inputs } => {
                    exec.eofcreate(&mut self.context, inputs)?
                }
                InterpreterAction::Return { result } => {
                    // free memory context.
                    shared_memory.free_context();

                    // pop last frame from the stack and consume it to create FrameResult.
                    let returned_frame = call_stack
                        .pop()
                        .expect("We just returned from Interpreter frame");

                    let ctx = &mut self.context;
                    FrameOrResult::Result(match returned_frame {
                        Frame::Call(frame) => {
                            // return_call
                            FrameResult::Call(exec.call_return(ctx, frame, result)?)
                        }
                        Frame::Create(frame) => {
                            // return_create
                            FrameResult::Create(exec.create_return(ctx, frame, result)?)
                        }
                        Frame::EOFCreate(frame) => {
                            // return_eofcreate
                            FrameResult::EOFCreate(exec.eofcreate_return(ctx, frame, result)?)
                        }
                    })
                }
                InterpreterAction::None => unreachable!("InterpreterAction::None is not expected"),
            };
            // handle result
            match frame_or_result {
                FrameOrResult::Frame(frame) => {
                    shared_memory.new_context();
                    call_stack.push(frame);
                    stack_frame = call_stack.last_mut().unwrap();
                }
                FrameOrResult::Result(result) => {
                    let Some(top_frame) = call_stack.last_mut() else {
                        // Break the loop if there are no more frames.
                        return Ok(result);
                    };
                    stack_frame = top_frame;
                    let ctx = &mut self.context;
                    // Insert result to the top frame.
                    match result {
                        FrameResult::Call(outcome) => {
                            // return_call
                            exec.insert_call_outcome(ctx, stack_frame, &mut shared_memory, outcome)?
                        }
                        FrameResult::Create(outcome) => {
                            // return_create
                            exec.insert_create_outcome(ctx, stack_frame, outcome)?
                        }
                        FrameResult::EOFCreate(outcome) => {
                            // return_eofcreate
                            exec.insert_eofcreate_outcome(ctx, stack_frame, outcome)?
                        }
                    }
                }
            }
        }
    }
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> Evm<'_, ChainSpecT, EXT, DB> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> ChainSpecT::Hardfork {
        self.handler.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balance to pay for the gas.
    #[inline]
    pub fn preverify_transaction(
        &mut self,
    ) -> Result<
        (),
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<ChainSpecT, DB::Error> {
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)
            .map_err(|e| {
                self.clear();
                e
            })?;
        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Pre verify transaction inner.
    #[inline]
    fn preverify_transaction_inner(
        &mut self,
    ) -> Result<
        u64,
        EVMError<
            DB::Error,
            <<ChainSpecT as ChainSpec>::Transaction as TransactionValidation>::ValidationError,
        >,
    > {
        self.handler.validation().env(&self.context.evm.env)?;
        let initial_gas_spend = self
            .handler
            .validation()
            .initial_tx_gas(&self.context.evm.env)?;
        self.handler
            .validation()
            .tx_against_state(&mut self.context)?;
        Ok(initial_gas_spend)
    }

    /// Transact transaction
    ///
    /// This function will validate the transaction.
    #[inline]
    pub fn transact(&mut self) -> EVMResult<ChainSpecT, DB::Error> {
        let initial_gas_spend = self.preverify_transaction_inner().map_err(|e| {
            self.clear();
            e
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of Env configuration
    #[inline]
    pub fn cfg(&self) -> &CfgEnv {
        &self.context.env().cfg
    }

    /// Returns the mutable reference of Env configuration
    #[inline]
    pub fn cfg_mut(&mut self) -> &mut CfgEnv {
        &mut self.context.evm.env.cfg
    }

    /// Returns the reference of transaction
    #[inline]
    pub fn tx(&self) -> &ChainSpecT::Transaction {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut ChainSpecT::Transaction {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &DB {
        &self.context.evm.db
    }

    /// Returns the mutable reference of database
    #[inline]
    pub fn db_mut(&mut self) -> &mut DB {
        &mut self.context.evm.db
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &ChainSpecT::Block {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut ChainSpecT::Block {
        &mut self.context.evm.env.block
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<ChainSpecT, EXT, DB> {
        self.context
    }

    /// Returns database and [`EnvWithChainSpec`].
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(self) -> (DB, EnvWithChainSpec<ChainSpecT>) {
        (
            self.context.evm.inner.db,
            EnvWithChainSpec {
                env: self.context.evm.inner.env,
                spec_id: self.handler.spec_id,
            },
        )
    }

    /// Returns [Context] and hardfork.
    #[inline]
    pub fn into_context_with_spec_id(self) -> ContextWithChainSpec<ChainSpecT, EXT, DB> {
        ContextWithChainSpec::new(self.context, self.handler.spec_id)
    }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(
        &mut self,
        initial_gas_spend: u64,
    ) -> EVMResult<ChainSpecT, DB::Error> {
        let spec_id = self.spec_id();
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // load precompiles
        let precompiles = pre_exec.load_precompiles();
        ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.gas_limit() - initial_gas_spend;

        let exec = self.handler.execution();
        // call inner handling of call/create
        let first_frame_or_result = match ctx.evm.env.tx.kind() {
            TxKind::Call(_) => exec.call(
                ctx,
                CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
            )?,
            TxKind::Create => {
                // if first byte of data is magic 0xEF00, then it is EOFCreate.
                if Into::<SpecId>::into(spec_id).is_enabled_in(SpecId::PRAGUE_EOF)
                    && ctx
                        .env()
                        .tx
                        .data()
                        .get(0..2)
                        .filter(|&t| t == [0xEF, 00])
                        .is_some()
                {
                    exec.eofcreate(
                        ctx,
                        Box::new(EOFCreateInputs::new_tx::<ChainSpecT>(
                            &ctx.evm.env.tx,
                            gas_limit,
                        )),
                    )?
                } else {
                    // Safe to unwrap because we are sure that it is create tx.
                    exec.create(
                        ctx,
                        CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
                    )?
                }
            }
        };

        // Starts the main running loop.
        let mut result = match first_frame_or_result {
            FrameOrResult::Frame(first_frame) => self.run_the_loop(first_frame)?,
            FrameOrResult::Result(result) => result,
        };

        let ctx = &mut self.context;

        // handle output of call/create calls.
        self.handler
            .execution()
            .last_frame_return(ctx, &mut result)?;

        let post_exec = self.handler.post_execution();
        // Reimburse the caller
        post_exec.reimburse_caller(ctx, result.gas())?;
        // Reward beneficiary
        post_exec.reward_beneficiary(ctx, result.gas())?;
        // Returns output of transaction.
        post_exec.output(ctx, result)
    }
}
