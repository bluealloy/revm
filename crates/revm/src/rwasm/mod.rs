use crate::{
    builder::{HandlerStage, RwasmBuilder, SetGenericStage},
    db::EmptyDB,
    interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome, Host},
    primitives::{
        BlockEnv,
        CfgEnv,
        EVMError,
        EVMResult,
        EnvWithHandlerCfg,
        ExecutionResult,
        HandlerCfg,
        ResultAndState,
        SpecId,
        TransactTo,
        TxEnv,
        U256,
    },
    rwasm::sdk_adapter::RwasmSdkAdapter,
    Context,
    ContextWithHandlerCfg,
    Database,
    DatabaseCommit,
    FrameResult,
    Handler,
};
use core::fmt;
use fluentbase_core::blended::BlendedRuntime;
use fluentbase_sdk::runtime::TestingContext;

pub mod context_reader;
pub mod sdk_adapter;

/// EVM instance containing both internal EVM context and external context
/// and the handler that dictates the logic of EVM (or hard fork specification).
pub struct Rwasm<'a, EXT, DB: Database> {
    /// Context of execution, containing both EVM and external context.
    pub context: Context<EXT, DB>,
    /// Handler is a component of the of EVM that contains all the logic.
    /// The handler contains
    /// specification id, and it is different depending on the specified fork.
    pub handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
}

impl<EXT, DB> fmt::Debug for Rwasm<'_, EXT, DB>
where
    EXT: fmt::Debug,
    DB: Database + fmt::Debug,
    DB::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Evm")
            .field("evm context", &self.context.evm)
            .finish_non_exhaustive()
    }
}

impl<EXT, DB: Database + DatabaseCommit> Rwasm<'_, EXT, DB> {
    /// Commit the changes to the database.
    pub fn transact_commit(&mut self) -> Result<ExecutionResult, EVMError<DB::Error>> {
        let ResultAndState { result, state } = self.transact()?;
        self.context.evm.db.commit(state);
        Ok(result)
    }
}

impl<'a> Rwasm<'a, (), EmptyDB> {
    /// Returns evm builder with an empty database and empty external context.
    pub fn builder() -> RwasmBuilder<'a, SetGenericStage, (), EmptyDB> {
        RwasmBuilder::default()
    }
}

impl<'a, EXT, DB: Database> Rwasm<'a, EXT, DB> {
    /// Create new EVM.
    pub fn new(
        mut context: Context<EXT, DB>,
        handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    ) -> Rwasm<'a, EXT, DB> {
        context.evm.journaled_state.set_spec_id(handler.cfg.spec_id);
        Rwasm { context, handler }
    }

    /// Allow for evm setting to be modified by feeding current evm
    /// into the builder for modifications.
    pub fn modify(self) -> RwasmBuilder<'a, HandlerStage, EXT, DB> {
        RwasmBuilder::<'a, HandlerStage, EXT, DB>::new(self)
    }
}

impl<EXT, DB: Database> Rwasm<'_, EXT, DB> {
    /// Returns specification (hardfork) that the EVM is instanced with.
    ///
    /// SpecId depends on the handler.
    pub fn spec_id(&self) -> SpecId {
        self.handler.cfg.spec_id
    }

    /// Pre verify transaction by checking Environment, initial gas spend and if caller
    /// has enough balances to pay for the gas.
    #[inline]
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<DB::Error>> {
        let output = self.preverify_transaction_inner().map(|_| ());
        self.clear();
        output
    }

    /// Calls clear handle of post-execution to clear the state for next execution.
    fn clear(&mut self) {
        self.handler.post_execution().clear(&mut self.context);
    }

    /// Transact pre-verified transaction
    ///
    /// This function will not validate the transaction.
    #[inline]
    pub fn transact_preverified(&mut self) -> EVMResult<DB::Error> {
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
    fn preverify_transaction_inner(&mut self) -> Result<u64, EVMError<DB::Error>> {
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
    pub fn transact(&mut self) -> EVMResult<DB::Error> {
        let initial_gas_spend = self.preverify_transaction_inner().map_err(|e| {
            self.clear();
            e
        })?;

        let output = self.transact_preverified_inner(initial_gas_spend);
        let output = self.handler.post_execution().end(&mut self.context, output);
        self.clear();
        output
    }

    /// Returns the reference of handler configuration
    #[inline]
    pub fn handler_cfg(&self) -> &HandlerCfg {
        &self.handler.cfg
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
    pub fn tx(&self) -> &TxEnv {
        &self.context.evm.env.tx
    }

    /// Returns the mutable reference of transaction
    #[inline]
    pub fn tx_mut(&mut self) -> &mut TxEnv {
        &mut self.context.evm.env.tx
    }

    /// Returns the reference of database
    #[inline]
    pub fn db(&self) -> &DB {
        &self.context.evm.db
    }

    /// Returns the mutable reference of a database
    #[inline]
    pub fn db_mut(&mut self) -> &mut DB {
        &mut self.context.evm.db
    }

    /// Returns the reference of block
    #[inline]
    pub fn block(&self) -> &BlockEnv {
        &self.context.evm.env.block
    }

    /// Returns the mutable reference of block
    #[inline]
    pub fn block_mut(&mut self) -> &mut BlockEnv {
        &mut self.context.evm.env.block
    }

    /// Modify spec id, this will create new EVM that matches this spec id.
    pub fn modify_spec_id(&mut self, spec_id: SpecId) {
        self.handler.modify_spec_id(spec_id);
    }

    /// Returns internal database and external struct.
    #[inline]
    pub fn into_context(self) -> Context<EXT, DB> {
        self.context
    }

    /// Returns database and [`EnvWithHandlerCfg`].
    #[inline]
    pub fn into_db_and_env_with_handler_cfg(self) -> (DB, EnvWithHandlerCfg) {
        (
            self.context.evm.inner.db,
            EnvWithHandlerCfg {
                env: self.context.evm.inner.env,
                handler_cfg: self.handler.cfg,
            },
        )
    }

    #[inline]
    pub fn into_db(self) -> DB {
        self.context.evm.inner.db
    }

    /// Returns [Context] and [HandlerCfg].
    #[inline]
    pub fn into_context_with_handler_cfg(self) -> ContextWithHandlerCfg<EXT, DB> {
        ContextWithHandlerCfg::new(self.context, self.handler.cfg)
    }

    /// Transact pre-verified transaction.
    // fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
    //     let spec_id = self.spec_id();
    //     let ctx = &mut self.context;
    //     let pre_exec = self.handler.pre_execution();
    //
    //     // load access list and beneficiary if needed.
    //     pre_exec.load_accounts(ctx)?;
    //
    //     // load precompiles
    //     let precompiles = pre_exec.load_precompiles();
    //     ctx.evm.set_precompiles(precompiles);
    //
    //     // deduce caller balance with its limit.
    //     pre_exec.deduct_caller(ctx)?;
    //
    //     let gas_limit = ctx.evm.env.tx.gas_limit - initial_gas_spend;
    //
    //     // apply EIP-7702 auth list.
    //     let eip7702_gas_refund = pre_exec.apply_eip7702_auth_list(ctx)? as i64;
    //
    //     let exec = self.handler.execution();
    //     // call inner handling of call/create
    //     let first_frame_or_result = match ctx.evm.env.tx.transact_to {
    //         TxKind::Call(_) => exec.call(
    //             ctx,
    //             CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
    //         )?,
    //         TxKind::Create => {
    //             // if the first byte of data is magic 0xEF00, then it is EOFCreate.
    //             if spec_id.is_enabled_in(SpecId::PRAGUE_EOF)
    //                 && ctx.env().tx.data.starts_with(&EOF_MAGIC_BYTES)
    //             {
    //                 exec.eofcreate(
    //                     ctx,
    //                     Box::new(EOFCreateInputs::new_tx(&ctx.evm.env.tx, gas_limit)),
    //                 )?
    //             } else {
    //                 // Safe to unwrap because we are sure that it is creating tx.
    //                 exec.create(
    //                     ctx,
    //                     CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap(),
    //                 )?
    //             }
    //         }
    //     };
    //
    //     // Starts the main running loop.
    //     let mut result = match first_frame_or_result {
    //         FrameOrResult::Frame(first_frame) => self.run_the_loop(first_frame)?,
    //         FrameOrResult::Result(result) => result,
    //     };
    //
    //     let ctx = &mut self.context;
    //
    //     // handle output of call/create calls.
    //     self.handler
    //         .execution()
    //         .last_frame_return(ctx, &mut result)?;
    //
    //     let post_exec = self.handler.post_execution();
    //     // calculate final refund and add EIP-7702 refund to gas.
    //     post_exec.refund(ctx, result.gas_mut(), eip7702_gas_refund);
    //     // Reimburse the caller
    //     post_exec.reimburse_caller(ctx, result.gas())?;
    //     // Reward beneficiary
    //     post_exec.reward_beneficiary(ctx, result.gas())?;
    //     // Returns output of transaction.
    //     post_exec.output(ctx, result)
    // }

    /// Transact pre-verified transaction.
    fn transact_preverified_inner(&mut self, initial_gas_spend: u64) -> EVMResult<DB::Error> {
        let ctx = &mut self.context;
        let pre_exec = self.handler.pre_execution();

        // load access list and beneficiary if needed.
        pre_exec.load_accounts(ctx)?;

        // load precompiles
        let precompiles = pre_exec.load_precompiles();
        ctx.evm.set_precompiles(precompiles);

        // deduce caller balance with its limit.
        pre_exec.deduct_caller(ctx)?;

        let gas_limit = ctx.evm.env.tx.gas_limit - initial_gas_spend;

        // load an EVM loader account to access storage slots
        // let (evm_storage, _) = ctx.evm.load_account(PRECOMPILE_EVM)?;
        // evm_storage.info.nonce = 1;
        // ctx.evm.touch(&PRECOMPILE_EVM);

        let mut result = match ctx.evm.env.tx.transact_to.clone() {
            TransactTo::Call(_) => {
                let inputs = CallInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap();
                let result = self.call_inner(inputs)?;
                FrameResult::Call(result)
            }
            TransactTo::Create => {
                let inputs = CreateInputs::new_boxed(&ctx.evm.env.tx, gas_limit).unwrap();
                let result = self.create_inner(inputs)?;
                FrameResult::Create(result)
            }
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

    /// EVM create opcode for both initial CREATE and CREATE2 opcodes.
    fn create_inner(
        &mut self,
        create_inputs: Box<CreateInputs>,
    ) -> Result<CreateOutcome, EVMError<DB::Error>> {
        let sdk = RwasmSdkAdapter::<TestingContext, DB>::new(&mut self.context.evm);
        let result = BlendedRuntime::new(sdk).create(create_inputs);
        Ok(result)
    }

    /// Main contract call of the EVM.
    fn call_inner(
        &mut self,
        call_inputs: Box<CallInputs>,
    ) -> Result<CallOutcome, EVMError<DB::Error>> {
        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if call_inputs.call_value() == U256::ZERO {
            self.context.evm.load_account(call_inputs.target_address)?;
            self.context
                .evm
                .journaled_state
                .touch(&call_inputs.target_address);
        }

        let sdk = RwasmSdkAdapter::<TestingContext, DB>::new(&mut self.context.evm);
        let result = BlendedRuntime::new(sdk).call(call_inputs);
        Ok(result)
    }
}
