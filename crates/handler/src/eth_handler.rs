use crate::{
    execution, post_execution, pre_execution, validation, EthPrecompileProvider, FrameContext,
    FrameResult,
};
use context::Context;
use context_interface::{
    result::{HaltReason, InvalidHeader, InvalidTransaction, ResultAndState},
    Block, BlockGetter, Cfg, CfgGetter, Database, DatabaseGetter, ErrorGetter, Journal,
    JournalDBError, JournalGetter, PerformantContextAccess, Transaction, TransactionGetter,
};
use handler_interface::{
    util::FrameOrFrameResult, Frame, FrameOrResult, InitialAndFloorGas, PrecompileProvider,
};
use interpreter::{
    interpreter::{EthInstructionProvider, EthInterpreter, InstructionProvider},
    FrameInput, Host,
};
use precompile::PrecompileErrors;
use primitives::Log;
use specification::hardfork::SpecId;
use state::EvmState;
use std::{vec, vec::Vec};

pub struct EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> {
    pub precompiles: PRECOMPILES,
    pub instructions: INSTRUCTIONS,
    pub _phantom: core::marker::PhantomData<(CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS)>,
}

impl<CTX: Host, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
    EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
where
    PRECOMPILES: PrecompileProvider<Context = CTX, Error = ERROR>,
    INSTRUCTIONS: InstructionProvider<WIRE = EthInterpreter, Host = CTX>,
{
    pub fn crete_frame_context(&self) -> FrameContext<PRECOMPILES, INSTRUCTIONS> {
        FrameContext {
            precompiles: self.precompiles.clone(),
            instructions: self.instructions.clone(),
        }
    }
}

impl<CTX: Host, ERROR, FRAME> Default
    for EthHandlerImpl<
        CTX,
        ERROR,
        FRAME,
        EthPrecompileProvider<CTX, ERROR>,
        EthInstructionProvider<EthInterpreter, CTX>,
    >
{
    fn default() -> Self {
        Self {
            precompiles: EthPrecompileProvider::new(SpecId::LATEST),
            instructions: EthInstructionProvider::new(),
            _phantom: core::marker::PhantomData,
        }
    }
}

pub trait EthContext:
    TransactionGetter
    + BlockGetter
    + DatabaseGetter
    + CfgGetter
    + PerformantContextAccess<Error = JournalDBError<Self>>
    + ErrorGetter<Error = JournalDBError<Self>>
    + JournalGetter<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>
    + Host
{
}

pub trait EthError<CTX: JournalGetter, FRAME: Frame>:
    From<InvalidTransaction>
    + From<InvalidHeader>
    + From<JournalDBError<CTX>>
    + From<<FRAME as Frame>::Error>
    + From<PrecompileErrors>
{
}

impl<
        CTX: JournalGetter,
        FRAME: Frame,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<JournalDBError<CTX>>
            + From<<FRAME as Frame>::Error>
            + From<PrecompileErrors>,
    > EthError<CTX, FRAME> for T
{
}

impl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS> EthHandler
    for EthHandlerImpl<CTX, ERROR, FRAME, PRECOMPILES, INSTRUCTIONS>
where
    CTX: EthContext,
    ERROR: EthError<CTX, FRAME>,
    PRECOMPILES: PrecompileProvider<Context = CTX, Error = ERROR>,
    INSTRUCTIONS: InstructionProvider<WIRE = EthInterpreter, Host = CTX>,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<
        Context = CTX,
        Error = ERROR,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
        FrameContext = FrameContext<PRECOMPILES, INSTRUCTIONS>,
    >,
{
    type Context = CTX;
    type Error = ERROR;
    type Frame = FRAME;
    type Precompiles = PRECOMPILES;
    type Instructions = INSTRUCTIONS;

    fn frame_context(
        &mut self,
        context: &mut Self::Context,
    ) -> <Self::Frame as Frame>::FrameContext {
        self.precompiles.set_spec(context.cfg().spec().into());
        self.crete_frame_context()
    }
}

pub trait EthHandler {
    type Context: EthContext;
    type Error: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalDBError<Self::Context>>
        + From<InvalidTransaction>
        + From<<Self::Frame as Frame>::Error>;
    type Precompiles: PrecompileProvider<Context = Self::Context, Error = Self::Error>;
    type Instructions: InstructionProvider<Host = Self::Context>;
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    type Frame: Frame<
        Context = Self::Context,
        Error = Self::Error,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
        FrameContext = FrameContext<Self::Precompiles, Self::Instructions>,
    >;

    fn run(
        &mut self,
        context: &mut Self::Context,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(context)?;
        let eip7702_refund = self.pre_execution(context)? as i64;
        let exec_result = self.execution(context, &init_and_floor_gas)?;
        let post_execution_gas = (init_and_floor_gas, eip7702_refund);
        self.post_execution(context, exec_result, post_execution_gas)
    }

    fn frame_context(
        &mut self,
        context: &mut Self::Context,
    ) -> <Self::Frame as Frame>::FrameContext;

    /// Call all validation functions
    fn validate(&self, context: &mut Self::Context) -> Result<InitialAndFloorGas, Self::Error> {
        self.validate_env(context)?;
        self.validate_tx_against_state(context)?;
        self.validate_initial_tx_gas(context)
    }

    /// Call all Pre execution functions.
    fn pre_execution(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        self.load_accounts(context)?;
        let gas = self.apply_eip7702_auth_list(context)?;
        self.deduct_caller(context)?;
        Ok(gas)
    }

    fn execution(
        &mut self,
        context: &mut Self::Context,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = context.tx().gas_limit() - init_and_floor_gas.initial_gas;

        // Make a context!
        let mut frame_context = self.frame_context(context);
        // Create first frame action
        let first_frame = self.create_first_frame(context, &mut frame_context, gas_limit)?;
        let mut frame_result = match first_frame {
            FrameOrResult::Frame(frame) => {
                self.run_exec_loop(context, &mut frame_context, frame)?
            }
            FrameOrResult::Result(result) => result,
        };

        self.last_frame_result(context, &mut frame_context, &mut frame_result)?;
        Ok(frame_result)
    }

    fn post_execution(
        &self,
        context: &mut Self::Context,
        mut exec_result: FrameResult,
        post_execution_gas: (InitialAndFloorGas, i64),
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        let init_and_floor_gas = post_execution_gas.0;
        let eip7702_gas_refund = post_execution_gas.1;

        // Calculate final refund and add EIP-7702 refund to gas.
        self.refund(context, &mut exec_result, eip7702_gas_refund);
        // Check if gas floor is met and spent at least a floor gas.
        self.eip7623_check_gas_floor(context, &mut exec_result, init_and_floor_gas);
        // Reimburse the caller
        self.reimburse_caller(context, &mut exec_result)?;
        // Reward beneficiary
        self.reward_beneficiary(context, &mut exec_result)?;
        // Returns output of transaction.
        self.output(context, exec_result)
    }

    /* VALIDATION */

    /// Validate env.
    fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
        validation::validate_env(context)
    }

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        validation::validate_tx_against_state(context)
    }

    /// Validate initial gas.
    fn validate_initial_tx_gas(
        &self,
        context: &Self::Context,
    ) -> Result<InitialAndFloorGas, Self::Error> {
        validation::validate_initial_tx_gas(context).map_err(From::from)
    }

    /* PRE EXECUTION */

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        pre_execution::load_accounts(context)
    }

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error> {
        pre_execution::apply_eip7702_auth_list(context)
    }

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error> {
        pre_execution::deduct_caller(context).map_err(From::from)
    }

    /* EXECUTION */
    fn create_first_frame(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        gas_limit: u64,
    ) -> Result<FrameOrFrameResult<Self::Frame>, Self::Error> {
        let init_frame =
            execution::create_init_frame(context.tx(), context.cfg().spec().into(), gas_limit);
        self.frame_init_first(context, frame_context, init_frame)
    }

    fn last_frame_result(
        &self,
        context: &mut Self::Context,
        _frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        execution::last_frame_result(context, frame_result);
        Ok(())
    }

    /* FRAMES */

    fn frame_init_first(
        &mut self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        frame_input: <<Self as EthHandler>::Frame as Frame>::FrameInit,
    ) -> Result<
        FrameOrResult<
            <Self as EthHandler>::Frame,
            <<Self as EthHandler>::Frame as Frame>::FrameResult,
        >,
        Self::Error,
    > {
        Self::Frame::init_first(context, frame_context, frame_input)
    }

    fn frame_init(
        &self,
        frame: &Self::Frame,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        frame_input: <<Self as EthHandler>::Frame as Frame>::FrameInit,
    ) -> Result<
        FrameOrResult<
            <Self as EthHandler>::Frame,
            <<Self as EthHandler>::Frame as Frame>::FrameResult,
        >,
        Self::Error,
    > {
        Frame::init(frame, context, frame_context, frame_input)
    }

    fn frame_call(
        &mut self,
        frame: &mut Self::Frame,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
    ) -> Result<
        FrameOrResult<
            <<Self as EthHandler>::Frame as Frame>::FrameInit,
            <<Self as EthHandler>::Frame as Frame>::FrameResult,
        >,
        Self::Error,
    > {
        Frame::run(frame, context, frame_context)
    }

    fn frame_return_result(
        &mut self,
        frame: &mut Self::Frame,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        result: <<Self as EthHandler>::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        Self::Frame::return_result(frame, context, frame_context, result)
    }

    fn frame_final_return(
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        result: &mut <<Self as EthHandler>::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        Self::Frame::final_return(context, frame_context, result)?;
        Ok(())
    }

    fn run_exec_loop(
        &self,
        context: &mut Self::Context,
        frame_context: &mut <<Self as EthHandler>::Frame as Frame>::FrameContext,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = frame.run(context, frame_context)?;

            let mut result = match call_or_result {
                FrameOrResult::Frame(init) => match frame.init(context, frame_context, init)? {
                    FrameOrResult::Frame(new_frame) => {
                        frame_stack.push(new_frame);
                        continue;
                    }
                    // Dont pop the frame as new frame was not created.
                    FrameOrResult::Result(result) => result,
                },
                FrameOrResult::Result(result) => {
                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                Self::Frame::final_return(context, frame_context, &mut result)?;
                return Ok(result);
            };
            frame.return_result(context, frame_context, result)?;
        }
    }

    /* POST EXECUTION */

    /// Calculate final refund.
    fn eip7623_check_gas_floor(
        &self,
        _context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        post_execution::eip7623_check_gas_floor(exec_result.gas_mut(), init_and_floor_gas)
    }

    /// Calculate final refund.
    fn refund(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        eip7702_refund: i64,
    ) {
        let spec = context.cfg().spec().into();
        post_execution::refund(spec, exec_result.gas_mut(), eip7702_refund)
    }

    /// Reimburse the caller with balance it didn't spent.
    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reimburse_caller(context, exec_result.gas_mut()).map_err(From::from)
    }

    /// Reward beneficiary with transaction rewards.
    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reward_beneficiary(context, exec_result.gas_mut()).map_err(From::from)
    }

    /// Main return handle, takes state from journal and transforms internal result to [`Output`][PostExecutionHandler::Output].
    fn output(
        &self,
        context: &mut Self::Context,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        context.take_error()?;
        Ok(post_execution::output(context, result))
    }

    /// Called when execution ends.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    ///
    /// While [`output`][PostExecutionHandler::output] will be omitted in case of the error.
    fn end(
        &self,
        _context: &mut Self::Context,
        end_output: Result<ResultAndState<HaltReason>, Self::Error>,
    ) -> Result<ResultAndState<HaltReason>, Self::Error> {
        end_output
    }

    /// Clean handler. It resets internal Journal state to default one.
    ///
    /// This handle is called every time regardless of the result of the transaction.
    fn clear(&self, context: &mut Self::Context) {
        context.journal().clear();
    }
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
        CHAIN,
    > EthContext for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
        CHAIN,
    > EthContext for &mut Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
}
