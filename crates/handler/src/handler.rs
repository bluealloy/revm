pub mod types;

use context::{ContextTrait, EvmTypesTrait};
use precompile::PrecompileErrors;
use primitives::Log;
use state::EvmState;
pub use types::{EthContext, EthError, MainnetHandler};

use crate::{execution, post_execution, pre_execution, validation, FrameResult};
use context_interface::{
    result::{HaltReasonTrait, InvalidHeader, InvalidTransaction, ResultAndState},
    Cfg, Database, Journal, Transaction,
};
use core::mem;
use handler_interface::{Frame, FrameInitOrResult, FrameOrResult, ItemOrResult};
use interpreter::{FrameInput, InitialAndFloorGas};
use std::{vec, vec::Vec};

pub trait EthTraitError<EVM: EvmTypesTrait>:
    From<InvalidTransaction>
    + From<InvalidHeader>
    + From<<<EVM::Context as ContextTrait>::Db as Database>::Error>
    + From<PrecompileErrors>
{
}

impl<
        EVM: EvmTypesTrait,
        T: From<InvalidTransaction>
            + From<InvalidHeader>
            + From<<<EVM::Context as ContextTrait>::Db as Database>::Error>
            + From<PrecompileErrors>,
    > EthTraitError<EVM> for T
{
}

/*

EVM {
ctx
frame
inspector
}

trait EthHandler {

    fn get_ctx_frame_mut(&mut) -> (&mut Self::Context, &mut Self::FrameContext);
}

trait InsectorEthHandler: EthHandler {
    fn components(&mut) -> (&mut Self::Context,&mut FrameCtx, &mut Self::Inspector);

    fn run_precompile

*/

// pub trait InspectorEthHandler: EthHandler<Context: Clone> {
//     type Inspector; // Inspector<Context = Self::Context, EthInterpreter>;

//     fn get_inspector_ctx(&mut self) -> (&mut Self::Inspector, &mut Self::Context);
// }

pub trait EthHandler {
    type Evm: EvmTypesTrait<
        Context: ContextTrait<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
    >;
    type Error: EthTraitError<Self::Evm>;
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    type Frame: Frame<
        Context = Self::Evm,
        Error = Self::Error,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
    >;
    // TODO `HaltReason` should be part of the output.
    type HaltReason: HaltReasonTrait;

    fn run(
        &mut self,
        evm: &mut Self::Evm,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let init_and_floor_gas = self.validate(evm)?;
        let eip7702_refund = self.pre_execution(evm)? as i64;
        let exec_result = self.execution(evm, &init_and_floor_gas)?;
        self.post_execution(evm, exec_result, init_and_floor_gas, eip7702_refund)
    }
    /// Call all validation functions
    fn validate(&self, evm: &mut Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        self.validate_env(evm)?;
        self.validate_tx_against_state(evm)?;
        self.validate_initial_tx_gas(evm)
    }

    /// Call all Pre execution functions.
    fn pre_execution(&self, evm: &mut Self::Evm) -> Result<u64, Self::Error> {
        self.load_accounts(evm)?;
        self.deduct_caller(evm)?;
        let gas = self.apply_eip7702_auth_list(evm)?;
        Ok(gas)
    }

    fn execution(
        &mut self,
        evm: &mut Self::Evm,
        init_and_floor_gas: &InitialAndFloorGas,
    ) -> Result<FrameResult, Self::Error> {
        let gas_limit = evm.ctx().tx().gas_limit() - init_and_floor_gas.initial_gas;

        // Create first frame action
        let first_frame = self.create_first_frame(evm, gas_limit)?;
        let mut frame_result = match first_frame {
            ItemOrResult::Item(frame) => self.run_exec_loop(evm, frame)?,
            ItemOrResult::Result(result) => result,
        };

        self.last_frame_result(evm, &mut frame_result)?;
        Ok(frame_result)
    }

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
        // Returns output of transaction.
        self.output(evm, exec_result)
    }

    /* VALIDATION */

    /// Validate env.
    fn validate_env(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        validation::validate_env(evm.ctx())
    }

    /// Validate transactions against state.
    fn validate_tx_against_state(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        validation::validate_tx_against_state(evm.ctx())
    }

    /// Validate initial gas.
    fn validate_initial_tx_gas(&self, evm: &Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        let ctx = evm.ctx_ref();
        validation::validate_initial_tx_gas(ctx.tx(), ctx.cfg().spec().into()).map_err(From::from)
    }

    /* PRE EXECUTION */

    fn load_accounts(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        pre_execution::load_accounts(evm.ctx())
    }

    fn apply_eip7702_auth_list(&self, evm: &mut Self::Evm) -> Result<u64, Self::Error> {
        pre_execution::apply_eip7702_auth_list(evm.ctx())
    }

    fn deduct_caller(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        pre_execution::deduct_caller(evm.ctx()).map_err(From::from)
    }

    /* EXECUTION */
    fn create_first_frame(
        &mut self,
        evm: &mut Self::Evm,
        gas_limit: u64,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        let ctx = evm.ctx_ref();
        let init_frame = execution::create_init_frame(ctx.tx(), ctx.cfg().spec().into(), gas_limit);
        self.frame_init_first(evm, init_frame)
    }

    fn last_frame_result(
        &self,
        evm: &mut Self::Evm,
        frame_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        execution::last_frame_result(evm.ctx(), frame_result);
        Ok(())
    }

    /* FRAMES */

    fn frame_init_first(
        &mut self,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        Self::Frame::init_first(evm, frame_input)
    }

    fn frame_init(
        &mut self,
        frame: &Self::Frame,
        evm: &mut Self::Evm,
        frame_input: <Self::Frame as Frame>::FrameInit,
    ) -> Result<FrameOrResult<Self::Frame>, Self::Error> {
        Frame::init(frame, evm, frame_input)
    }

    fn frame_call(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
    ) -> Result<FrameInitOrResult<Self::Frame>, Self::Error> {
        Frame::run(frame, evm)
    }

    fn frame_return_result(
        &mut self,
        frame: &mut Self::Frame,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        Self::Frame::return_result(frame, evm, result)
    }

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
    fn eip7623_check_gas_floor(
        &self,
        _evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        post_execution::eip7623_check_gas_floor(exec_result.gas_mut(), init_and_floor_gas)
    }

    /// Calculate final refund.
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
    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reimburse_caller(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    /// Reward beneficiary with transaction rewards.
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reward_beneficiary(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    /// Main return handle, takes state from journal and transforms internal result to output.
    fn output(
        &self,
        evm: &mut Self::Evm,
        result: <Self::Frame as Frame>::FrameResult,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let ctx = evm.ctx();
        mem::replace(ctx.error(), Ok(()))?;
        Ok(post_execution::output(ctx, result))
    }

    /// Called when execution ends.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    ///
    /// While output will be omitted in case of the error.
    fn end(
        &self,
        _evm: &mut Self::Evm,
        end_output: Result<ResultAndState<Self::HaltReason>, Self::Error>,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        end_output
    }

    /// Clean handler. It resets internal Journal state to default one.
    ///
    /// This handle is called every time regardless of the result of the transaction.
    fn clear(&self, evm: &mut Self::Evm) {
        evm.ctx().journal().clear();
    }
}
