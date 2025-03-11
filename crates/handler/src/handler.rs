use crate::EvmTr;
use crate::{
    execution, post_execution, pre_execution, validation, Frame, FrameInitOrResult, FrameOrResult,
    FrameResult, ItemOrResult,
};
use context::JournalOutput;
use context_interface::ContextTr;
use context_interface::{
    result::{HaltReasonTr, InvalidHeader, InvalidTransaction, ResultAndState},
    Cfg, Database, JournalTr, Transaction,
};
use core::mem;
use interpreter::{FrameInput, Gas, InitialAndFloorGas};
use precompile::PrecompileError;
use std::{vec, vec::Vec};

pub trait EvmTrError<EVM: EvmTr>:
    From<InvalidTransaction>
    + From<InvalidHeader>
    + From<<<EVM::Context as ContextTr>::Db as Database>::Error>
    + From<PrecompileError>
{
}

impl<
        EVM: EvmTr,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<<<EVM::Context as ContextTr>::Db as Database>::Error>
            + From<PrecompileError>,
    > EvmTrError<EVM> for T
{
}

/// Main logic of Ethereum Mainnet execution.
///
/// The starting point for execution is the [`Handler::run`] method. And when implemented
/// out of box gives you the ability to execute Ethereum mainnet transactions.
///
/// It is made as a trait so that EVM variants can override of execution logic
/// by implementing their own method logic.
///
/// Handler logic is split in four parts:
///   * Verification - loads caller account checks initial gas requirement.
///   * Pre execution - loads and warms rest of accounts and deducts initial gas.
///   * Execution - Executed the main frame loop. It calls [`Frame`] for sub call logic.
///   * Post execution - Calculates the final refund, checks gas floor, reimburses caller and
///     rewards beneficiary.
///
/// [`Handler::catch_error`] method is used for cleanup of intermediate state if there is error
/// during execution.
pub trait Handler {
    /// The EVM type that contains Context, Instruction, Precompiles.
    type Evm: EvmTr<Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>>;
    /// Error that is going to be returned.
    type Error: EvmTrError<Self::Evm>;
    /// Frame type contains data for frame execution. EthFrame currently supports Call, Create and EofCreate frames.
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    type Frame: Frame<
        Evm = Self::Evm,
        Error = Self::Error,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
    >;
    /// Halt reason type is part of the output
    ///  TODO `HaltReason` should be part of the output.
    type HaltReason: HaltReasonTr;

    /// Main entry point for execution.
    ///
    /// This method will call [`Handler::run_without_catch_error`] and if it returns an error
    /// it will call [`Handler::catch_error`] to handle the error.
    ///
    /// Catching error method clears the intermediate state.
    #[inline]
    fn run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        // run inner handler and catch all errors to handle cleanup.
        match self.run_without_catch_error(evm) {
            Ok(output) => Ok(output),
            Err(e) => self.catch_error(evm, e),
        }
    }

    /// Called by [`Handler::run`] to execute the handler logic.
    ///
    /// This method will call the four parts of execution. [Handler::validate],
    /// [Handler::pre_execution], [Handler::execution], [Handler::post_execution].
    ///
    /// If any of the methods return an error the error will be returned, this method does not
    /// catch or call [`Handler::catch_error`] method.
    #[inline]
    fn run_without_catch_error(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        let exec_result = self.execution(evm, &init_and_floor_gas)?;
        self.post_execution(evm, exec_result, init_and_floor_gas, eip7702_refund)
    }

    /// Call all validation functions that validated the environment (tx, block, config).
    ///
    /// Next step is calculating initial and floor gas and checking if it is covered by gas_limit
    ///
    /// Last step loads caller account and validated transaction fields agains state.
    /// Nonce is checked and if there is balance to cover max amount of gas that can be spend.
    #[inline]
    fn validate(&self, evm: &mut Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        self.validate_env(evm)?;
        let initial_and_floor_gas = self.validate_initial_tx_gas(evm)?;
        self.validate_tx_against_state(evm)?;
        Ok(initial_and_floor_gas)
    }

    /// This method prepares the evm for execution.
    ///
    /// It load beneficiary account (EIP-3651: Warm COINBASE) and all accounts and storages from access list.
    /// (EIP-2929)
    ///
    /// Deducts the caller balance with max amount of fee that it can spend
    ///
    /// If transaction is EIP-7702 type, it will apply the authorization list and delegate successfull authorizations.
    /// It returns the amount of gas refund from EIP-7702. Auhorizations are applied before execution.
    #[inline]
    fn pre_execution(&self, evm: &mut Self::Evm) -> Result<u64, Self::Error> {
        self.load_accounts(evm)?;
        self.deduct_caller(evm)?;
        let gas = self.apply_eip7702_auth_list(evm)?;
        Ok(gas)
    }

    /// Execution creates first frame input and initializes first frame and calls the exec loop.
    ///
    /// In the end it will always call [Handler::last_frame_result] to handle returned gas from the call.
    #[inline]
    fn execution(
        &mut self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = evm.ctx().tx().gas_limit() - init_and_floor_gas.initial_gas;

        // Create first frame action
        let first_frame_input = self.first_frame_input(evm, gas_limit)?;
        let first_frame = self.first_frame_init(evm, first_frame_input)?;
        let mut frame_result = match first_frame {
            ItemOrResult::Item(frame) => self.run_exec_loop(evm, frame)?,
            ItemOrResult::Result(result) => result,
        };

        self.last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

    /// Post execution handles final steps of transaction execution.
    ///
    /// It calculates the final refund, checks gas floor (EIP-7623) and decides to use floor gas or returned call gas.
    /// After EIP-7623 at least floor gas should be spend.
    ///
    /// It reimburses the caller with the balance that was not spend and rewards the beneficiary with transaction rewards.
    /// Transaction rewards is calculated as a effective gas price, and base fee amount is thrown away (burned).
    ///
    /// The last step in execution is output finalization, where journal state is returned as result of execution.
    /// And inner state is cleared and prepared for next execution.
    #[inline]
    fn post_execution(
        &self,
        evm: &mut Self::Evm,
        mut exec_result: FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
        eip7702_gas_refund: i64,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        // Calculate final refund and add EIP-7702 refund to gas.
        self.refund(evm, &mut exec_result, eip7702_gas_refund);
        // Check if gas floor is met and spent at least a floor gas.
        self.eip7623_check_gas_floor(evm, &mut exec_result, init_and_floor_gas);
        // Reimburse the caller
        self.reimburse_caller(evm, &mut exec_result)?;
        // Reward beneficiary
        self.reward_beneficiary(evm, &mut exec_result)?;
        // Prepare output of transaction.
        self.output(evm, exec_result)
    }

    /* VALIDATION */

    /// Validate block, transaction and config fields.
    ///
    /// Every check that can be done without loading/touching the state
    /// is done here. We check obvious things as if tx gas limit is less than block gas limit.
    #[inline]
    fn validate_env(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        validation::validate_env(evm.ctx())
    }

    /// Initial gas depends on data input type of transaction and its kind, is it create or a call.
    ///
    /// Additional initial cost depends on access list and authorization list.
    ///
    /// The main check is done if initial cost is less them transaction gas limit.
    #[inline]
    fn validate_initial_tx_gas(&self, evm: &Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        let ctx = evm.ctx_ref();
        validation::validate_initial_tx_gas(ctx.tx(), ctx.cfg().spec().into()).map_err(From::from)
    }

    /// In this method caller is loaded and we get access to its nonce and balance.
    ///
    /// It calculates maximum fee that this tx can spend and checks if caller can pay it.
    #[inline]
    fn validate_tx_against_state(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        validation::validate_tx_against_state(evm.ctx())
    }

    /* PRE EXECUTION */

    /// Loads access list and beneficiary account. And marks them as warm inside [`context::Journal`].
    #[inline]
    fn load_accounts(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        pre_execution::load_accounts(evm.ctx())
    }

    /// Iterates over authorization list checks if authority signature, nonce and chain ids are correct
    /// and applies authorization to the accounts.
    ///
    /// Returns the amount of gas refund from EIP-7702.
    #[inline]
    fn apply_eip7702_auth_list(&self, evm: &mut Self::Evm) -> Result<u64, Self::Error> {
        pre_execution::apply_eip7702_auth_list(evm.ctx())
    }

    /// Deducts the caller balance with max amount of fee that it can spend and the balance he is sending.
    ///
    /// After execution unspent balance is returned to the caller.
    #[inline]
    fn deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        pre_execution::deduct_caller(evm.ctx()).map_err(From::from)
    }

    /* EXECUTION */

    /// Creates first frame input from transaction, gas limit and config.
    #[inline]
    fn first_frame_input(
        &mut self,
        evm: &mut Self::Evm,
        gas_limit: u64,
    ) -> Result<FrameInput, Self::Error> {
        let ctx: &<<Self as Handler>::Evm as EvmTr>::Context = evm.ctx_ref();
        Ok(execution::create_init_frame(
            ctx.tx(),
            ctx.cfg().spec().into(),
            gas_limit,
        ))
    }

    /// Received the output of the first call and handles returned gas.
    #[inline]
    fn last_frame_result(
        &self,
        evm: &mut Self::Evm,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        let instruction_result = frame_result.interpreter_result().result;
        let gas = frame_result.gas_mut();
        let remaining = gas.remaining();
        let refunded = gas.refunded();

        // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
        *gas = Gas::new_spent(evm.ctx().tx().gas_limit());

        if instruction_result.is_ok_or_revert() {
            gas.erase_cost(remaining);
        }

        if instruction_result.is_ok() {
            gas.record_refund(refunded);
        }
        Ok(())
    }

    /* FRAMES */

    #[inline]
    fn first_frame_init(
        &mut self,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        Self::Frame::init_first(evm, frame_input)
    }

    #[inline]
    fn frame_init(
        &mut self,
        frame: &Self::Frame,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        Frame::init(frame, evm, frame_input)
    }

    #[inline]
    fn frame_call(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
    ) -> Result<FrameInitOrResult<Self::Frame>, Self::Error> {
        Frame::run(frame, evm)
    }

    #[inline]
    fn frame_return_result(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        Self::Frame::return_result(frame, evm, result)
    }

    #[inline]
    fn run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        frame: Self::Frame,
    ) -> Result<FrameResult, Self::Error> {
        let mut frame_stack: Vec<Self::Frame> = vec![frame];
        loop {
            let frame = frame_stack.last_mut().unwrap();
            let call_or_result = self.frame_call(frame, evm)?;

            let result = match call_or_result {
                ItemOrResult::Item(init) => {
                    match self.frame_init(frame, evm, init)? {
                        ItemOrResult::Item(new_frame) => {
                            frame_stack.push(new_frame);
                            continue;
                        }
                        // Dont pop the frame as new frame was not created.
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => {
                    // Pop frame that returned result
                    frame_stack.pop();
                    result
                }
            };

            let Some(frame) = frame_stack.last_mut() else {
                return Ok(result);
            };
            self.frame_return_result(frame, evm, result)?;
        }
    }

    /* POST EXECUTION */

    /// Calculate final refund.
    #[inline]
    fn eip7623_check_gas_floor(
        &self,
        _evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        post_execution::eip7623_check_gas_floor(exec_result.gas_mut(), init_and_floor_gas)
    }

    /// Calculate final refund.
    #[inline]
    fn refund(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        eip7702_refund: i64,
    ) {
        let spec = evm.ctx().cfg().spec().into();
        post_execution::refund(spec, exec_result.gas_mut(), eip7702_refund)
    }

    /// Reimburse the caller with balance it didn't spent.
    #[inline]
    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reimburse_caller(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    /// Reward beneficiary with transaction rewards.
    #[inline]
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reward_beneficiary(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    /// Main return handle, takes state from journal and transforms internal result to output.
    #[inline]
    fn output(
        &self,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let ctx = evm.ctx();
        mem::replace(ctx.error(), Ok(()))?;
        let output = post_execution::output(ctx, result);

        // clear journal
        evm.ctx().journal().clear();
        Ok(output)
    }

    /// Called every time at the end of execution. Used for clearing the journal.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    #[inline]
    fn catch_error(
        &self,
        evm: &mut Self::Evm,
        error: Self::Error,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        // do the cleanup of journal if error is caught
        evm.ctx().journal().clear();
        Err(error)
    }
}
