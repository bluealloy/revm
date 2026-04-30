use crate::{
    evm::FrameTr,
    execution,
    post_execution::{self, build_result_gas},
    pre_execution::{self, apply_eip7702_auth_list},
    validation, EvmTr, FrameResult, ItemOrResult,
};
use context::{
    result::{ExecutionResult, FromStringError},
    LocalContextTr,
};
use context_interface::{
    context::{take_error, ContextError},
    result::{HaltReasonTr, InvalidHeader, InvalidTransaction, ResultGas},
    Cfg, ContextTr, Database, JournalTr, Transaction,
};
use interpreter::{interpreter_action::FrameInit, Gas, InitialAndFloorGas, SharedMemory};
use primitives::U256;

/// Trait for errors that can occur during EVM execution.
///
/// This trait represents the minimal error requirements for EVM execution,
/// ensuring that all necessary error types can be converted into the handler's error type.
pub trait EvmTrError<EVM: EvmTr>:
    From<InvalidTransaction>
    + From<InvalidHeader>
    + From<<<EVM::Context as ContextTr>::Db as Database>::Error>
    + From<ContextError<<<EVM::Context as ContextTr>::Db as Database>::Error>>
    + FromStringError
{
}

impl<
        EVM: EvmTr,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<<<EVM::Context as ContextTr>::Db as Database>::Error>
            + From<ContextError<<<EVM::Context as ContextTr>::Db as Database>::Error>>
            + FromStringError,
    > EvmTrError<EVM> for T
{
}

/// Caches the EIP-8037 `cost_per_state_byte` on the local context for the
/// current transaction, honoring `cfg.cpsb_override`.
///
/// Called at the start of every top-level execution entry point so that
/// `Host::cpsb` becomes a single field read instead of a recomputation.
#[inline]
pub fn cache_cpsb_on_local<CTX: ContextTr>(ctx: &mut CTX) {
    let cpsb = ctx.cfg().cpsb();
    ctx.local_mut().set_cpsb(cpsb);
}

/// The main implementation of Ethereum Mainnet transaction execution.
///
/// The [`Handler::run`] method serves as the entry point for execution and provides
/// out-of-the-box support for executing Ethereum mainnet transactions.
///
/// This trait allows EVM variants to customize execution logic by implementing
/// their own method implementations.
///
/// The handler logic consists of four phases:
///   * Validation - Validates tx/block/config fields and loads caller account and validates initial gas requirements and
///     balance checks.
///   * Pre-execution - Loads and warms accounts, deducts initial gas
///   * Execution - Executes the main frame loop, delegating to [`EvmTr`] for creating and running call frames.
///   * Post-execution - Calculates final refunds, validates gas floor, reimburses caller,
///     and rewards beneficiary
///
///
/// The [`Handler::catch_error`] method handles cleanup of intermediate state if an error
/// occurs during execution.
///
/// # Returns
///
/// Returns execution status, error, gas spend and logs. State change is not returned and it is
/// contained inside Context Journal. This setup allows multiple transactions to be chain executed.
///
/// To finalize the execution and obtain changed state, call [`JournalTr::finalize`] function.
pub trait Handler {
    /// The EVM type containing Context, Instruction, and Precompiles implementations.
    type Evm: EvmTr<
        Context: ContextTr<Journal: JournalTr, Local: LocalContextTr>,
        Frame: FrameTr<FrameInit = FrameInit, FrameResult = FrameResult>,
    >;
    /// The error type returned by this handler.
    type Error: EvmTrError<Self::Evm>;
    /// The halt reason type included in the output
    type HaltReason: HaltReasonTr;

    /// The main entry point for transaction execution.
    ///
    /// This method calls [`Handler::run_without_catch_error`] and if it returns an error,
    /// calls [`Handler::catch_error`] to handle the error and cleanup.
    ///
    /// The [`Handler::catch_error`] method ensures intermediate state is properly cleared.
    ///
    /// # Error handling
    ///
    /// In case of error, the journal can be in an inconsistent state and should be cleared by calling
    /// [`JournalTr::discard_tx`] method or dropped.
    ///
    /// # Returns
    ///
    /// Returns execution result, error, gas spend and logs.
    #[inline]
    fn run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        // Cache EIP-8037 cost_per_state_byte on the local context so the hot-path
        // Host::cpsb is a single field read. Honors cfg.cpsb_override.
        cache_cpsb_on_local(evm.ctx_mut());
        // Run inner handler and catch all errors to handle cleanup.
        match self.run_without_catch_error(evm) {
            Ok(output) => Ok(output),
            Err(e) => self.catch_error(evm, e),
        }
    }

    /// Runs the system call.
    ///
    /// System call is a special transaction where caller is a [`crate::SYSTEM_ADDRESS`]
    ///
    /// It is used to call a system contracts and it skips all the `validation` and `pre-execution` and most of `post-execution` phases.
    /// For example it will not deduct the caller or reward the beneficiary.
    ///
    /// State changs can be obtained by calling [`JournalTr::finalize`] method from the [`EvmTr::Context`].
    ///
    /// # Error handling
    ///
    /// By design system call should not fail and should always succeed.
    /// In case of an error (If fetching account/storage on rpc fails), the journal can be in an inconsistent
    /// state and should be cleared by calling [`JournalTr::discard_tx`] method or dropped.
    #[inline]
    fn run_system_call(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        // Cache EIP-8037 cost_per_state_byte on the local context. System calls
        // skip validation/pre-execution but still execute interpreter code that
        // reads Host::cpsb, so this must be populated here too.
        cache_cpsb_on_local(evm.ctx_mut());
        // dummy values that are not used.
        let init_and_floor_gas = InitialAndFloorGas::new(0, 0);
        // call execution and than output.
        match self
            .execution(evm, &init_and_floor_gas)
            .and_then(|exec_result| {
                // System calls have no intrinsic gas; build ResultGas from frame result.
                let gas = exec_result.gas();
                let result_gas = build_result_gas(gas, init_and_floor_gas);
                self.execution_result(evm, exec_result, result_gas)
            }) {
            out @ Ok(_) => out,
            Err(e) => self.catch_error(evm, e),
        }
    }

    /// Called by [`Handler::run`] to execute the core handler logic.
    ///
    /// Executes the four phases in sequence: [Handler::validate],
    /// [Handler::pre_execution], [Handler::execution], [Handler::post_execution].
    ///
    /// Returns any errors without catching them or calling [`Handler::catch_error`].
    #[inline]
    fn run_without_catch_error(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        let mut init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm, &mut init_and_floor_gas)?;
        // Regular refund is returned from pre_execution after state gas split is applied
        let eip7702_regular_refund = eip7702_refund as i64;

        let mut exec_result = self.execution(evm, &init_and_floor_gas)?;
        let result_gas = self.post_execution(
            evm,
            &mut exec_result,
            init_and_floor_gas,
            eip7702_regular_refund,
        )?;

        // Prepare the output
        self.execution_result(evm, exec_result, result_gas)
    }

    /// Validates the execution environment and transaction parameters.
    ///
    /// Calculates initial and floor gas requirements and verifies they are covered by the gas limit.
    ///
    /// Validation against state is done later in pre-execution phase in deduct_caller function.
    #[inline]
    fn validate(&self, evm: &mut Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        self.validate_env(evm)?;
        self.validate_initial_tx_gas(evm)
    }

    /// Prepares the EVM state for execution.
    ///
    /// Loads the beneficiary account (EIP-3651: Warm COINBASE) and all accounts/storage from the access list (EIP-2929).
    ///
    /// Deducts the maximum possible fee from the caller's balance.
    ///
    /// For EIP-7702 transactions, applies the authorization list and delegates successful authorizations.
    /// Returns the gas refund amount from EIP-7702. Authorizations are applied before execution begins.
    #[inline]
    fn pre_execution(
        &self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &mut InitialAndFloorGas,
    ) -> Result<u64, Self::Error> {
        self.validate_against_state_and_deduct_caller(evm, init_and_floor_gas)?;
        self.load_accounts(evm)?;

        let gas = self.apply_eip7702_auth_list(evm, init_and_floor_gas)?;
        Ok(gas)
    }

    /// Creates and executes the initial frame, then processes the execution loop.
    ///
    /// Always calls [Handler::last_frame_result] to handle returned gas from the call.
    #[inline]
    fn execution(
        &mut self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        // Compute the regular gas budget and EIP-8037 reservoir for the first frame.
        let (gas_limit, reservoir) = init_and_floor_gas.initial_gas_and_reservoir(
            evm.ctx().tx().gas_limit(),
            evm.ctx().cfg().tx_gas_limit_cap(),
            evm.ctx().cfg().is_amsterdam_eip8037_enabled(),
        );

        // Create first frame action
        // Note: first_frame_input now handles state gas deduction from the reservoir
        let first_frame_input = self.first_frame_input(evm, gas_limit, reservoir)?;

        // Run execution loop
        let mut frame_result = self.run_exec_loop(evm, first_frame_input)?;

        // Handle last frame result
        self.last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

    /// Handles the final steps of transaction execution.
    ///
    /// Calculates final refunds and validates the gas floor (EIP-7623) to ensure minimum gas is spent.
    /// After EIP-7623, at least floor gas must be consumed.
    ///
    /// Reimburses unused gas to the caller and rewards the beneficiary with transaction fees.
    /// The effective gas price determines rewards, with the base fee being burned.
    ///
    /// Finally, finalizes output by returning the journal state and clearing internal state
    /// for the next execution.
    #[inline]
    fn post_execution(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
        eip7702_gas_refund: i64,
    ) -> Result<ResultGas, Self::Error> {

        //println!("init_and_floor_gas: {:?}", exec_result.gas());
        // EIP-8037: Refund reservoir for accounts that were created and then
        // self-destructed in this tx (EIP-6780 erasure). Runs first so the
        // updated reservoir feeds into refund, reimbursement, and beneficiary
        // reward accounting below.
        self.eip8037_selfdestruct_refund(evm, exec_result);

        // Calculate final refund and add EIP-7702 refund to gas.
        self.refund(evm, exec_result, eip7702_gas_refund);

        // Build ResultGas from the final gas state
        // This includes all necessary fields and gas values.
        let result_gas = post_execution::build_result_gas(exec_result.gas(), init_and_floor_gas);

        // Ensure gas floor is met and minimum floor gas is spent.
        // if `cfg.is_eip7623_disabled` is true, floor gas will be set to zero
        self.eip7623_check_gas_floor(evm, exec_result, init_and_floor_gas);
        // Return unused gas to caller
        self.reimburse_caller(evm, exec_result)?;
        // Pay transaction fees to beneficiary
        self.reward_beneficiary(evm, exec_result)?;
        // Build ResultGas from the final gas state
        Ok(result_gas)
    }

    /* VALIDATION */

    /// Validates block, transaction and configuration fields.
    ///
    /// Performs all validation checks that can be done without loading state.
    /// For example, verifies transaction gas limit is below block gas limit.
    #[inline]
    fn validate_env(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        validation::validate_env(evm.ctx())
    }

    /// Calculates initial gas costs based on transaction type and input data.
    ///
    /// Includes additional costs for access list and authorization list.
    ///
    /// Verifies the initial cost does not exceed the transaction gas limit.
    #[inline]
    fn validate_initial_tx_gas(
        &self,
        evm: &mut Self::Evm,
    ) -> Result<InitialAndFloorGas, Self::Error> {
        let ctx = evm.ctx_ref();
        let gas = validation::validate_initial_tx_gas(
            ctx.tx(),
            ctx.cfg().spec().into(),
            ctx.cfg().is_eip7623_disabled(),
            ctx.cfg().is_amsterdam_eip8037_enabled(),
            ctx.cfg().tx_gas_limit_cap(),
            ctx.cfg().cpsb(),
        )?;

        Ok(gas)
    }

    /* PRE EXECUTION */

    /// Loads access list and beneficiary account, marking them as warm in the [`context::Journal`].
    #[inline]
    fn load_accounts(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        pre_execution::load_accounts(evm)
    }

    /// Processes the authorization list, validating authority signatures, nonces and chain IDs.
    /// Applies valid authorizations to accounts.
    ///
    /// Returns the gas refund amount specified by EIP-7702.
    #[inline]
    fn apply_eip7702_auth_list(
        &self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &mut InitialAndFloorGas,
    ) -> Result<u64, Self::Error> {
        apply_eip7702_auth_list(evm.ctx_mut(), init_and_floor_gas)
    }

    /// Deducts the maximum possible fee from caller's balance.
    ///
    /// If cfg.is_balance_check_disabled, this method will add back enough funds to ensure that
    /// the caller's balance is at least tx.value() before returning. Note that the amount of funds
    /// added back in this case may exceed the maximum fee.
    ///
    /// Unused fees are returned to caller after execution completes.
    #[inline]
    fn validate_against_state_and_deduct_caller(
        &self,
        evm: &mut Self::Evm,
        _init_and_floor_gas: &mut InitialAndFloorGas,
    ) -> Result<(), Self::Error> {
        pre_execution::validate_against_state_and_deduct_caller(evm.ctx())
    }

    /* EXECUTION */

    /// Creates initial frame input using transaction parameters, gas limit and configuration.
    #[inline]
    fn first_frame_input(
        &mut self,
        evm: &mut Self::Evm,
        gas_limit: u64,
        reservoir: u64,
    ) -> Result<FrameInit, Self::Error> {
        let ctx = evm.ctx_mut();
        let mut memory = SharedMemory::new_with_buffer(ctx.local().shared_memory_buffer().clone());
        memory.set_memory_limit(ctx.cfg().memory_limit());

        let frame_input = execution::create_init_frame(ctx, gas_limit, reservoir)?;

        Ok(FrameInit {
            depth: 0,
            memory,
            frame_input,
        })
    }

    /// Processes the result of the initial call and handles returned gas.
    #[inline]
    fn last_frame_result(
        &mut self,
        evm: &mut Self::Evm,
        frame_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        let instruction_result = frame_result.interpreter_result().result;

        // // Detect a failed top-level CREATE for the EIP-8037 state-gas refund
        // // applied below. Mirrors the `create_failed` condition used in
        // // `EthFrame::return_result` for nested creates, with one twist for the
        // // top-level case: a `SelfDestruct` result counts as failure too. Per
        // // EIP-6780, a contract that self-destructs in the same transaction it
        // // was created in is erased at tx end, so the intrinsic
        // // `create_state_gas` (which `eip8037_selfdestruct_state_gas_refund`
        // // skips for the CREATE-tx target) must be unwound here.
        // let create_failed = match frame_result {
        //     FrameResult::Create(outcome) => {
        //         outcome.address.is_none() || !instruction_result.is_ok_without_selfdestruct()
        //     }
        //     FrameResult::Call(_) => false,
        // };

        let gas = frame_result.gas_mut();
        let remaining = gas.remaining();
        let refunded = gas.refunded();
        let reservoir = gas.reservoir();
        let state_gas_spent = gas.state_gas_spent();

        // Spend the gas limit. Gas is reimbursed when the tx returns successfully.
        *gas = Gas::new_spent_with_reservoir(evm.ctx().tx().gas_limit(), reservoir);

        if instruction_result.is_ok_or_revert() {
            // Return unused regular gas. Reservoir is handled separately via state_gas_spent.
            gas.erase_cost(remaining);
        }

        if instruction_result.is_ok() {
            gas.record_refund(refunded);
        }

        if instruction_result.is_ok() {
            gas.set_state_gas_spent(state_gas_spent);
        } else {
            // State changes rolled back, so no execution state gas was consumed.
            // `state_gas_spent` can be negative (EIP-8037 issue #2) if the top
            // frame refilled more than it charged; clamp to zero for reservoir
            // recovery since the combined value cannot go below zero.
            gas.set_state_gas_spent(0);
            let combined = state_gas_spent.saturating_add_unsigned(reservoir).max(0) as u64;
            gas.set_reservoir(combined);
        }

        // // EIP-8037: for a failed top-level CREATE (or one that self-destructs
        // // in init code, see EIP-6780), refund the intrinsic `create_state_gas`
        // // to the reservoir. The nested-create equivalent is
        // // `EthFrame::return_result`'s `refill_reservoir(create_state_gas)`; at
        // // the top level the same charge is deducted in
        // // `initial_gas_and_reservoir` rather than via `record_state_cost`, so
        // // it would otherwise stay consumed when the deployment is rolled back
        // // or erased.
        // if create_failed && evm.ctx().cfg().is_amsterdam_eip8037_enabled() {
        //     let ctx = evm.ctx();
        //     let state_gas_charged = ctx.cfg().gas_params().create_state_gas(ctx.local().cpsb());
        //     gas.refill_reservoir(state_gas_charged);
        // }

        Ok(())
    }

    /* FRAMES */

    /// Executes the main frame processing loop.
    ///
    /// This loop manages the frame stack, processing each frame until execution completes.
    /// For each iteration:
    /// 1. Calls the current frame
    /// 2. Handles the returned frame input or result
    /// 3. Creates new frames or propagates results as needed
    #[inline]
    fn run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        first_frame_input: <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameResult, Self::Error> {
        let res = evm.frame_init(first_frame_input)?;

        if let ItemOrResult::Result(frame_result) = res {
            return Ok(frame_result);
        }

        loop {
            let call_or_result = evm.frame_run()?;

            let result = match call_or_result {
                ItemOrResult::Item(init) => {
                    match evm.frame_init(init)? {
                        ItemOrResult::Item(_) => {
                            continue;
                        }
                        // Do not pop the frame since no new frame was created
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => result,
            };

            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }

    /* POST EXECUTION */

    /// Validates that the minimum gas floor requirements are satisfied.
    ///
    /// Ensures that at least the floor gas amount has been consumed during execution.
    #[inline]
    fn eip7623_check_gas_floor(
        &self,
        _evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        post_execution::eip7623_check_gas_floor(exec_result.gas_mut(), init_and_floor_gas)
    }

    /// EIP-8037: Refunds state gas for accounts that were both created and
    /// self-destructed in this transaction (EIP-6780 erasure).
    ///
    /// Iterates over destroyed accounts in the journal, sums the state gas that
    /// was charged for creating each account, depositing its code, and setting
    /// its storage slots, and refills the reservoir with the total. Refilling
    /// the reservoir (rather than recording a refund) bypasses the 1/5 refund
    /// cap because this state never actually persists.
    #[inline]
    fn eip8037_selfdestruct_refund(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) {
        post_execution::eip8037_selfdestruct_state_gas_refund(evm.ctx(), exec_result.gas_mut())
    }

    /// Calculates the final gas refund amount, including any EIP-7702 refunds.
    #[inline]
    fn refund(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
        eip7702_refund: i64,
    ) {
        let spec = evm.ctx().cfg().spec().into();
        post_execution::refund(spec, exec_result.gas_mut(), eip7702_refund)
    }

    /// Returns unused gas costs to the transaction sender's account.
    #[inline]
    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reimburse_caller(evm.ctx(), exec_result.gas(), U256::ZERO)
            .map_err(From::from)
    }

    /// Transfers transaction fees to the block beneficiary's account.
    #[inline]
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reward_beneficiary(evm.ctx(), exec_result.gas()).map_err(From::from)
    }

    /// Processes the final execution output.
    ///
    /// This method, retrieves the final state from the journal, converts internal results to the external output format.
    /// Internal state is cleared and EVM is prepared for the next transaction.
    #[inline]
    fn execution_result(
        &mut self,
        evm: &mut Self::Evm,
        result: <<Self::Evm as EvmTr>::Frame as FrameTr>::FrameResult,
        result_gas: ResultGas,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        take_error::<Self::Error, _>(evm.ctx().error())?;

        let exec_result = post_execution::output(evm.ctx(), result, result_gas);

        // commit transaction
        evm.ctx().journal_mut().commit_tx();
        evm.ctx().local_mut().clear();
        evm.frame_stack().clear();

        Ok(exec_result)
    }

    /// Handles cleanup when an error occurs during execution.
    ///
    /// Ensures the journal state is properly cleared before propagating the error.
    /// On happy path journal is cleared in [`Handler::execution_result`] method.
    #[inline]
    fn catch_error(
        &self,
        evm: &mut Self::Evm,
        error: Self::Error,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        // clean up local context. Initcode cache needs to be discarded.
        evm.ctx().local_mut().clear();
        evm.ctx().journal_mut().discard_tx();
        evm.frame_stack().clear();
        Err(error)
    }
}
