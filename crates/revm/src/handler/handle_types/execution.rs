use crate::{
    handler::mainnet,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{db::Database, Spec},
    CallFrame, Context, CreateFrame, Frame, FrameOrResult, FrameResult,
};
use alloc::{boxed::Box, sync::Arc};
use core::ops::Range;
use revm_interpreter::{CallOutcome, CreateOutcome, InterpreterResult};

/// Handles first frame return handle.
pub type LastFrameReturnHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &mut FrameResult) + 'a>;

/// Handle sub call.
pub type FrameCallHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Box<CallInputs>, Range<usize>) -> FrameOrResult + 'a>;

/// Handle call return
pub type FrameCallReturnHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Box<CallFrame>, InterpreterResult) -> CallOutcome + 'a>;

/// Insert call outcome to the parent
pub type InsertCallOutcomeHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &mut Frame, &mut SharedMemory, CallOutcome) + 'a>;

/// Handle sub create.
pub type FrameCreateHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Box<CreateInputs>) -> FrameOrResult + 'a>;

/// Handle create return
pub type FrameCreateReturnHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, Box<CreateFrame>, InterpreterResult) -> CreateOutcome + 'a>;

/// Insert call outcome to the parent
pub type InsertCreateOutcomeHandle<'a, EXT, DB> =
    Arc<dyn Fn(&mut Context<EXT, DB>, &mut Frame, CreateOutcome) + 'a>;

/// Handles related to stack frames.
pub struct ExecutionHandler<'a, EXT, DB: Database> {
    /// Validate Transaction against the state.
    /// Uses env, call result and returned gas from the call to determine the gas
    /// that is returned from transaction execution..
    pub last_frame_return: LastFrameReturnHandle<'a, EXT, DB>,
    /// Frame call
    pub call: FrameCallHandle<'a, EXT, DB>,
    /// Call return
    pub call_return: FrameCallReturnHandle<'a, EXT, DB>,
    /// Insert call outcome
    pub insert_call_outcome: InsertCallOutcomeHandle<'a, EXT, DB>,
    /// Frame crate
    pub create: FrameCreateHandle<'a, EXT, DB>,
    /// Crate return
    pub create_return: FrameCreateReturnHandle<'a, EXT, DB>,
    /// Insert create outcome.
    pub insert_create_outcome: InsertCreateOutcomeHandle<'a, EXT, DB>,
}

impl<'a, EXT: 'a, DB: Database + 'a> ExecutionHandler<'a, EXT, DB> {
    /// Creates mainnet ExecutionHandler..
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            last_frame_return: Arc::new(mainnet::last_frame_return::<SPEC, EXT, DB>),
            //frame_return: Arc::new(mainnet::frame_return::<SPEC, EXT, DB>),
            call: Arc::new(mainnet::call::<SPEC, EXT, DB>),
            call_return: Arc::new(mainnet::call_return::<EXT, DB>),
            insert_call_outcome: Arc::new(mainnet::insert_call_outcome),
            create: Arc::new(mainnet::create::<SPEC, EXT, DB>),
            create_return: Arc::new(mainnet::create_return::<SPEC, EXT, DB>),
            insert_create_outcome: Arc::new(mainnet::insert_create_outcome),
        }
    }
}

impl<'a, EXT, DB: Database> ExecutionHandler<'a, EXT, DB> {
    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    pub fn last_frame_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame_result: &mut FrameResult,
    ) {
        (self.last_frame_return)(context, frame_result)
    }

    /// Call frame call handler.
    pub fn call(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CallInputs>,
        return_memory_offset: Range<usize>,
    ) -> FrameOrResult {
        (self.call)(context, inputs, return_memory_offset)
    }

    /// Call registered handler for call return.
    pub fn call_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> CallOutcome {
        (self.call_return)(context, frame, interpreter_result)
    }

    /// Call registered handler for inserting call outcome.
    pub fn insert_call_outcome(
        &self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) {
        (self.insert_call_outcome)(context, frame, shared_memory, outcome)
    }

    /// Call Create frame
    pub fn create(
        &self,
        context: &mut Context<EXT, DB>,
        inputs: Box<CreateInputs>,
    ) -> FrameOrResult {
        (self.create)(context, inputs)
    }

    /// Call handler for create return.
    pub fn create_return(
        &self,
        context: &mut Context<EXT, DB>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> CreateOutcome {
        (self.create_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    pub fn insert_create_outcome(
        &self,
        context: &mut Context<EXT, DB>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) {
        (self.insert_create_outcome)(context, frame, outcome)
    }

    // /// Frame return
    // pub fn frame_return(
    //     &self,
    //     context: &mut Context<EXT, DB>,
    //     child_stack_frame: Box<CallStackFrame>,
    //     parent_stack_frame: Option<&mut Box<CallStackFrame>>,
    //     shared_memory: &mut SharedMemory,
    //     result: InterpreterResult,
    // ) -> Option<InterpreterResult> {
    //     (self.frame_return)(
    //         context,
    //         child_stack_frame,
    //         parent_stack_frame,
    //         shared_memory,
    //         result,
    //     )
    // }
}
