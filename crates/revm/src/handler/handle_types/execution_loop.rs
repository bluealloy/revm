use crate::{
    handler::mainnet,
    interpreter::{
        CallInputs, CreateInputs, Gas, InstructionResult, InterpreterResult, SharedMemory,
    },
    primitives::{db::Database, Env, Spec},
    CallStackFrame, Context, FrameOrResult,
};
use alloc::{boxed::Box, sync::Arc};
use core::ops::Range;

/// Creates the first frame.
pub type CreateFirstFrameHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, u64) -> FrameOrResult + 'a>;

/// Handles first frame return handle.
pub type FirstFrameReturnHandle<'a> = Arc<dyn Fn(&Env, InstructionResult, Gas) -> Gas + 'a>;

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

/// Handles related to stack frames.
pub struct ExecutionLoopHandler<'a, EXT, DB: Database> {
    /// Create Main frame
    pub create_first_frame: CreateFirstFrameHandle<'a, EXT, DB>,
    /// Validate Transaction against the state.
    /// Uses env, call result and returned gas from the call to determine the gas
    /// that is returned from transaction execution..
    pub first_frame_return: FirstFrameReturnHandle<'a>,
    /// Frame return
    pub frame_return: FrameReturnHandle<'a, EXT, DB>,
    /// Frame sub call
    pub sub_call: FrameSubCallHandle<'a, EXT, DB>,
    /// Frame sub crate
    pub sub_create: FrameSubCreateHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> ExecutionLoopHandler<'a, EXT, DB> {
    /// Creates mainnet ExecutionLoopHandler..
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            create_first_frame: Arc::new(mainnet::create_first_frame::<SPEC, EXT, DB>),
            first_frame_return: Arc::new(mainnet::first_frame_return::<SPEC>),
            frame_return: Arc::new(mainnet::frame_return::<SPEC, EXT, DB>),
            sub_call: Arc::new(mainnet::sub_call::<SPEC, EXT, DB>),
            sub_create: Arc::new(mainnet::sub_create::<SPEC, EXT, DB>),
        }
    }
}

impl<'a, EXT, DB: Database> ExecutionLoopHandler<'a, EXT, DB> {
    /// Create first call frame.
    pub fn create_first_frame(
        &self,
        context: &mut Context<EXT, DB>,
        gas_limit: u64,
    ) -> FrameOrResult {
        (self.create_first_frame)(context, gas_limit)
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    pub fn first_frame_return(
        &self,
        env: &Env,
        call_result: InstructionResult,
        returned_gas: Gas,
    ) -> Gas {
        (self.first_frame_return)(env, call_result, returned_gas)
    }

    /// Call frame sub call handler.
    pub fn sub_call(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
        curent_stack_frame: &mut CallStackFrame,
        shared_memory: &mut SharedMemory,
        return_memory_offset: Range<usize>,
    ) -> Option<Box<CallStackFrame>> {
        (self.sub_call)(
            context,
            inputs,
            curent_stack_frame,
            shared_memory,
            return_memory_offset,
        )
    }

    /// Create sub frame
    pub fn sub_create(
        &self,
        context: &mut Context<EXT, DB>,
        curent_stack_frame: &mut CallStackFrame,
        inputs: Box<CreateInputs>,
    ) -> Option<Box<CallStackFrame>> {
        (self.sub_create)(context, curent_stack_frame, inputs)
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
        (self.frame_return)(
            context,
            child_stack_frame,
            parent_stack_frame,
            shared_memory,
            result,
        )
    }
}
