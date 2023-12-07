//! Handler contains all the logic that is specific to the Evm.
//! It is used to define different behavior depending on the chain (Optimism,Mainnet) or
//! hardfork (Berlin, London, ..).

// Modules.
pub mod mainnet;
#[cfg(feature = "optimism")]
pub mod optimism;
pub mod register;

// Exports.
pub use register::{inspector_handle_register, HandleRegister};

// Includes.
use crate::{
    interpreter::{
        opcode::{make_instruction_table, InstructionTables},
        CallInputs, CreateInputs, Gas, Host, InstructionResult, InterpreterResult,
        SelfDestructResult, SharedMemory,
    },
    precompile::{Address, Bytes, B256},
    primitives::{
        db::Database, EVMError, EVMResultGeneric, Env, Output, ResultAndState, Spec, SpecId,
    },
    CallStackFrame, Context, FrameOrResult,
};
use alloc::sync::Arc;
use core::ops::Range;

/// Handle call return and return final gas value.
pub type CallReturnHandle<'a> = Arc<dyn Fn(&Env, InstructionResult, Gas) -> Gas + 'a>;

/// Load access list account, precompiles and beneficiary.
/// There is not need to load Caller as it is assumed that
/// it will be loaded in DeductCallerHandle.
pub type MainLoadHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> Result<(), EVMError<<DB as Database>::Error>> + 'a>;

/// Deduct the caller to its limit.
pub type DeductCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Reimburse the caller with ethereum it didn't spent.
pub type ReimburseCallerHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &Gas) -> EVMResultGeneric<(), <DB as Database>::Error> + 'a>;

/// Reward beneficiary with transaction rewards.
pub type RewardBeneficiaryHandle<'a, EXT, DB> = ReimburseCallerHandle<'a, EXT, DB>;

/// Main return handle, takes state from journal and transforms internal result to external.
pub type MainReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            InstructionResult,
            Output,
            &Gas,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// After subcall is finished, call this function to handle return result.
///
/// Return Some if we want to halt execution. This can be done on any stack frame.
pub type FrameReturnHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            // context
            &mut Context<EXT, DB>,
            // returned frame
            Box<CallStackFrame>,
            // parent frame if it exist.
            Option<&mut Box<CallStackFrame>>,
            // shared memory to insert output of the call.
            &mut SharedMemory,
            // output of frame execution.
            InterpreterResult,
        ) -> Option<InterpreterResult>
        + 'a,
>;

/// Create first frame.
pub type CreateFirstFrame<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, u64) -> FrameOrResult + 'a>;

/// Call to the host from Interpreter to save the log.
pub type HostLogHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Address, Vec<B256>, Bytes) + 'a>;

/// Call to the host from Interpreter to selfdestruct account.
///
/// After CANCUN hardfork original contract will stay the same but the value will
/// be transfered to the target.
pub type HostSelfdestructHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Address, Address) -> Option<SelfDestructResult> + 'a>;

/// End handle, takes result and state and returns final result.
/// This will be called after all the other handlers.
///
/// It is useful for catching errors and returning them in a different way.
pub type EndHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Result<ResultAndState, EVMError<<DB as Database>::Error>>,
        ) -> Result<ResultAndState, EVMError<<DB as Database>::Error>>
        + 'a,
>;

/// Handle sub call.
pub type FrameSubCallHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            Box<CallInputs>,
            &mut CallStackFrame,
            &mut SharedMemory,
            Range<usize>,
        ) -> Option<Box<CallStackFrame>>
        + 'a,
>;

/// Handle sub create.
pub type FrameSubCreateHandle<'a, EXT, DB> = Arc<
    dyn Fn(
            &mut Context<EXT, DB>,
            &mut CallStackFrame,
            Box<CreateInputs>,
        ) -> Option<Box<CallStackFrame>>
        + 'a,
>;

/// Handle that validates env.
pub type ValidateEnvHandle<'a, DB> =
    Arc<dyn Fn(&Env) -> Result<(), EVMError<<DB as Database>::Error>> + 'a>;

/// Handle that validates transaction environment against the state.
/// Second parametar is initial gas.
pub type ValidateTxEnvAgainstState<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>) -> Result<(), EVMError<<DB as Database>::Error>> + 'a>;

/// Initial gas calculation handle
pub type ValidateInitialTxGasHandle<'a, DB> =
    Arc<dyn Fn(&Env) -> Result<u64, EVMError<<DB as Database>::Error>> + 'a>;

pub struct ValidationHandles<'a, EXT, DB: Database> {
    /// Initial tx gas.
    pub validate_initial_tx_gas: ValidateInitialTxGasHandle<'a, DB>,
    /// Validate transactions against state data.
    pub validate_tx_against_state: ValidateTxEnvAgainstState<'a, EXT, DB>,
    /// Validate Env
    pub validate_env: ValidateEnvHandle<'a, DB>,
}

pub struct MainHandles<'a, EXT, DB: Database> {
    /// Validate Transaction against the state.
    /// Uses env, call result and returned gas from the call to determine the gas
    /// that is returned from transaction execution..
    pub call_return: CallReturnHandle<'a>,
    /// Main load handle
    pub main_load_handle: MainLoadHandle<'a, EXT, DB>,
    /// Deduct max value from the caller.
    pub deduct_caller: DeductCallerHandle<'a, EXT, DB>,
    /// Reimburse the caller with ethereum it didn't spent.
    pub reimburse_caller: ReimburseCallerHandle<'a, EXT, DB>,
    /// Reward the beneficiary with caller fee.
    pub reward_beneficiary: RewardBeneficiaryHandle<'a, EXT, DB>,
    /// Main return handle, returns the output of the transact.
    pub main_return: MainReturnHandle<'a, EXT, DB>,
    /// End handle.
    pub end: EndHandle<'a, EXT, DB>,
}

pub struct FrameHandles<'a, EXT, DB: Database> {
    /// Create Main frame
    pub create_first_frame: CreateFirstFrame<'a, EXT, DB>,
    /// Frame return
    pub frame_return: FrameReturnHandle<'a, EXT, DB>,
    /// Frame sub call
    pub frame_sub_call: FrameSubCallHandle<'a, EXT, DB>,
    /// Frame sub crate
    pub frame_sub_create: FrameSubCreateHandle<'a, EXT, DB>,
}

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<'a, H: Host + 'a, EXT, DB: Database> {
    /// Specification ID.
    pub spec_id: SpecId,
    /// Instruction table type.
    pub instruction_table: Option<InstructionTables<'a, H>>,
    /// Validity handles.
    pub validation: ValidationHandles<'a, EXT, DB>,
    /// Main handles.
    pub main: MainHandles<'a, EXT, DB>,
    /// Frame handles.
    pub frame: FrameHandles<'a, EXT, DB>,
    /// Host log handle.
    pub host_log: HostLogHandle<'a, EXT, DB>,
    /// Host selfdestruct handle.
    pub host_selfdestruct: HostSelfdestructHandle<'a, EXT, DB>,
}

impl<'a, H: Host, EXT: 'a, DB: Database + 'a> Handler<'a, H, EXT, DB> {
    /// Handler for the mainnet
    pub fn mainnet<SPEC: Spec + 'static>() -> Self {
        Self {
            spec_id: SPEC::SPEC_ID,
            instruction_table: Some(InstructionTables::Plain(make_instruction_table::<H, SPEC>())),
            validation: ValidationHandles {
                validate_initial_tx_gas: Arc::new(
                    mainnet::preexecution::validate_initial_tx_gas::<SPEC, DB>,
                ),
                validate_env: Arc::new(mainnet::preexecution::validate_env::<SPEC, DB>),
                validate_tx_against_state: Arc::new(
                    mainnet::preexecution::validate_tx_against_state::<SPEC, EXT, DB>,
                ),
            },
            main: MainHandles {
                call_return: Arc::new(mainnet::handle_call_return::<SPEC>),
                main_load_handle: Arc::new(mainnet::main_load::<SPEC, EXT, DB>),
                deduct_caller: Arc::new(mainnet::deduct_caller::<SPEC, EXT, DB>),
                reimburse_caller: Arc::new(mainnet::handle_reimburse_caller::<SPEC, EXT, DB>),
                reward_beneficiary: Arc::new(mainnet::reward_beneficiary::<SPEC, EXT, DB>),
                main_return: Arc::new(mainnet::main::main_return::<EXT, DB>),
                end: Arc::new(mainnet::main::end_handle::<EXT, DB>),
            },
            frame: FrameHandles {
                create_first_frame: Arc::new(mainnet::frames::create_first_frame::<SPEC, EXT, DB>),
                frame_return: Arc::new(mainnet::frames::handle_frame_return::<SPEC, EXT, DB>),
                frame_sub_call: Arc::new(mainnet::frames::handle_frame_sub_call::<SPEC, EXT, DB>),
                frame_sub_create: Arc::new(
                    mainnet::frames::handle_frame_sub_create::<SPEC, EXT, DB>,
                ),
            },
            host_log: Arc::new(mainnet::host::handle_host_log::<SPEC, EXT, DB>),
            host_selfdestruct: Arc::new(mainnet::host::handle_selfdestruct::<SPEC, EXT, DB>),
        }
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    pub fn call_return(&self, env: &Env, call_result: InstructionResult, returned_gas: Gas) -> Gas {
        (self.main.call_return)(env, call_result, returned_gas)
    }

    /// Reimburse the caller with gas that were not spend.
    pub fn reimburse_caller(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.main.reimburse_caller)(context, gas)
    }

    /// Deduct caller to its limit.
    pub fn deduct_caller(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.main.deduct_caller)(context)
    }

    /// Reward beneficiary
    pub fn reward_beneficiary(
        &self,
        context: &mut Context<EXT, DB>,
        gas: &Gas,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.main.reward_beneficiary)(context, gas)
    }

    /// Main return.
    pub fn main_return(
        &self,
        context: &mut Context<EXT, DB>,
        call_result: InstructionResult,
        output: Output,
        gas: &Gas,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.main.main_return)(context, call_result, output, gas)
    }

    /// End handler.
    pub fn end(
        &self,
        context: &mut Context<EXT, DB>,
        end_output: Result<ResultAndState, EVMError<DB::Error>>,
    ) -> Result<ResultAndState, EVMError<DB::Error>> {
        (self.main.end)(context, end_output)
    }

    /// Call frame sub call handler.
    pub fn frame_sub_call(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
        curent_stack_frame: &mut CallStackFrame,
        shared_memory: &mut SharedMemory,
        return_memory_offset: Range<usize>,
    ) -> Option<Box<CallStackFrame>> {
        (self.frame.frame_sub_call)(
            context,
            inputs,
            curent_stack_frame,
            shared_memory,
            return_memory_offset,
        )
    }

    pub fn frame_sub_create(
        &self,
        context: &mut Context<EXT, DB>,
        curent_stack_frame: &mut CallStackFrame,
        inputs: Box<CreateInputs>,
    ) -> Option<Box<CallStackFrame>> {
        (self.frame.frame_sub_create)(context, curent_stack_frame, inputs)
    }

    /// Frame return
    pub fn frame_return(
        &self,
        context: &mut Context<EXT, DB>,
        child_stack_frame: Box<CallStackFrame>,
        parent_stack_frame: Option<&mut Box<CallStackFrame>>,
        shared_memory: &mut SharedMemory,
        result: InterpreterResult,
    ) -> Option<InterpreterResult> {
        (self.frame.frame_return)(
            context,
            child_stack_frame,
            parent_stack_frame,
            shared_memory,
            result,
        )
    }

    /// Call host log handle.
    pub fn host_log(
        &self,
        context: &mut Context<EXT, DB>,
        address: Address,
        topics: Vec<B256>,
        data: Bytes,
    ) {
        (self.host_log)(context, address, topics, data)
    }

    /// Call host selfdestruct handle.
    pub fn host_selfdestruct(
        &self,
        context: &mut Context<EXT, DB>,
        address: Address,
        target: Address,
    ) -> Option<SelfDestructResult> {
        (self.host_selfdestruct)(context, address, target)
    }

    /// Validate env.
    pub fn validate_env(&self, env: &Env) -> Result<(), EVMError<DB::Error>> {
        (self.validation.validate_env)(env)
    }

    /// Initial gas
    pub fn validate_initial_tx_gas(&self, env: &Env) -> Result<u64, EVMError<DB::Error>> {
        (self.validation.validate_initial_tx_gas)(env)
    }

    /// Validate ttansaction against the state.
    pub fn validate_tx_against_state(
        &self,
        context: &mut Context<EXT, DB>,
    ) -> Result<(), EVMError<DB::Error>> {
        (self.validation.validate_tx_against_state)(context)
    }

    /// Create first call frame.
    pub fn create_first_frame(
        &self,
        context: &mut Context<EXT, DB>,
        gas_limit: u64,
    ) -> FrameOrResult {
        (self.frame.create_first_frame)(context, gas_limit)
    }

    /// Main load
    pub fn main_load(&self, context: &mut Context<EXT, DB>) -> Result<(), EVMError<DB::Error>> {
        (self.main.main_load_handle)(context)
    }
}
