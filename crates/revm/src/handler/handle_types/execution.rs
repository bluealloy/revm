use crate::{
    frame::EOFCreateFrame,
    handler::mainnet,
    interpreter::{CallInputs, CreateInputs, SharedMemory},
    primitives::{EVMResultGeneric, Spec},
    CallFrame, Context, CreateFrame, EvmWiring, Frame, FrameOrResult, FrameResult,
};
use revm_interpreter::{
    opcode::InstructionTables, CallOutcome, CreateOutcome, EOFCreateInputs, InterpreterAction,
    InterpreterResult,
};
use std::{boxed::Box, sync::Arc};

/// Handles first frame return handle.
pub type LastFrameReturnHandle<'a, EvmWiringT> = Arc<
    dyn Fn(&mut Context<EvmWiringT>, &mut FrameResult) -> EVMResultGeneric<(), EvmWiringT> + 'a,
>;

/// Executes a single frame. Errors can be returned in the EVM context.
pub type ExecuteFrameHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Frame,
            &mut SharedMemory,
            &InstructionTables<'_, Context<EvmWiringT>>,
            &mut Context<EvmWiringT>,
        ) -> EVMResultGeneric<InterpreterAction, EvmWiringT>
        + 'a,
>;

/// Handle sub call.
pub type FrameCallHandle<'a, EvmWiringT> = Arc<
    dyn Fn(&mut Context<EvmWiringT>, Box<CallInputs>) -> EVMResultGeneric<FrameOrResult, EvmWiringT>
        + 'a,
>;

/// Handle call return
pub type FrameCallReturnHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            Box<CallFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CallOutcome, EvmWiringT>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCallOutcomeHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            &mut Frame,
            &mut SharedMemory,
            CallOutcome,
        ) -> EVMResultGeneric<(), EvmWiringT>
        + 'a,
>;

/// Handle sub create.
pub type FrameCreateHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            Box<CreateInputs>,
        ) -> EVMResultGeneric<FrameOrResult, EvmWiringT>
        + 'a,
>;

/// Handle create return
pub type FrameCreateReturnHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            Box<CreateFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CreateOutcome, EvmWiringT>
        + 'a,
>;

/// Insert call outcome to the parent
pub type InsertCreateOutcomeHandle<'a, EvmWiringT> = Arc<
    dyn Fn(&mut Context<EvmWiringT>, &mut Frame, CreateOutcome) -> EVMResultGeneric<(), EvmWiringT>
        + 'a,
>;

/// Handle EOF sub create.
pub type FrameEOFCreateHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            Box<EOFCreateInputs>,
        ) -> EVMResultGeneric<FrameOrResult, EvmWiringT>
        + 'a,
>;

/// Handle EOF create return
pub type FrameEOFCreateReturnHandle<'a, EvmWiringT> = Arc<
    dyn Fn(
            &mut Context<EvmWiringT>,
            Box<EOFCreateFrame>,
            InterpreterResult,
        ) -> EVMResultGeneric<CreateOutcome, EvmWiringT>
        + 'a,
>;

/// Insert EOF crate outcome to the parent
pub type InsertEOFCreateOutcomeHandle<'a, EvmWiringT> = Arc<
    dyn Fn(&mut Context<EvmWiringT>, &mut Frame, CreateOutcome) -> EVMResultGeneric<(), EvmWiringT>
        + 'a,
>;

/// Handles related to stack frames.
pub struct ExecutionHandler<'a, EvmWiringT: EvmWiring> {
    /// Handles last frame return, modified gas for refund and
    /// sets tx gas limit.
    pub last_frame_return: LastFrameReturnHandle<'a, EvmWiringT>,
    /// Executes a single frame.
    pub execute_frame: ExecuteFrameHandle<'a, EvmWiringT>,
    /// Frame call
    pub call: FrameCallHandle<'a, EvmWiringT>,
    /// Call return
    pub call_return: FrameCallReturnHandle<'a, EvmWiringT>,
    /// Insert call outcome
    pub insert_call_outcome: InsertCallOutcomeHandle<'a, EvmWiringT>,
    /// Frame crate
    pub create: FrameCreateHandle<'a, EvmWiringT>,
    /// Crate return
    pub create_return: FrameCreateReturnHandle<'a, EvmWiringT>,
    /// Insert create outcome.
    pub insert_create_outcome: InsertCreateOutcomeHandle<'a, EvmWiringT>,
    /// Frame EOFCreate
    pub eofcreate: FrameEOFCreateHandle<'a, EvmWiringT>,
    /// EOFCreate return
    pub eofcreate_return: FrameEOFCreateReturnHandle<'a, EvmWiringT>,
    /// Insert EOFCreate outcome.
    pub insert_eofcreate_outcome: InsertEOFCreateOutcomeHandle<'a, EvmWiringT>,
}

impl<'a, EvmWiringT: EvmWiring + 'a> ExecutionHandler<'a, EvmWiringT> {
    /// Creates mainnet ExecutionHandler.
    pub fn new<SPEC: Spec + 'a>() -> Self {
        Self {
            last_frame_return: Arc::new(mainnet::last_frame_return::<EvmWiringT, SPEC>),
            execute_frame: Arc::new(mainnet::execute_frame::<EvmWiringT, SPEC>),
            call: Arc::new(mainnet::call::<EvmWiringT, SPEC>),
            call_return: Arc::new(mainnet::call_return::<EvmWiringT>),
            insert_call_outcome: Arc::new(mainnet::insert_call_outcome),
            create: Arc::new(mainnet::create::<EvmWiringT, SPEC>),
            create_return: Arc::new(mainnet::create_return::<EvmWiringT, SPEC>),
            insert_create_outcome: Arc::new(mainnet::insert_create_outcome),
            eofcreate: Arc::new(mainnet::eofcreate::<EvmWiringT, SPEC>),
            eofcreate_return: Arc::new(mainnet::eofcreate_return::<EvmWiringT, SPEC>),
            insert_eofcreate_outcome: Arc::new(mainnet::insert_eofcreate_outcome),
        }
    }
}

impl<'a, EvmWiringT: EvmWiring> ExecutionHandler<'a, EvmWiringT> {
    /// Executes single frame.
    #[inline]
    pub fn execute_frame(
        &self,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        instruction_tables: &InstructionTables<'_, Context<EvmWiringT>>,
        context: &mut Context<EvmWiringT>,
    ) -> EVMResultGeneric<InterpreterAction, EvmWiringT> {
        (self.execute_frame)(frame, shared_memory, instruction_tables, context)
    }

    /// Handle call return, depending on instruction result gas will be reimbursed or not.
    #[inline]
    pub fn last_frame_return(
        &self,
        context: &mut Context<EvmWiringT>,
        frame_result: &mut FrameResult,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.last_frame_return)(context, frame_result)
    }

    /// Call frame call handler.
    #[inline]
    pub fn call(
        &self,
        context: &mut Context<EvmWiringT>,
        inputs: Box<CallInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
        (self.call)(context, inputs)
    }

    /// Call registered handler for call return.
    #[inline]
    pub fn call_return(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: Box<CallFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CallOutcome, EvmWiringT> {
        (self.call_return)(context, frame, interpreter_result)
    }

    /// Call registered handler for inserting call outcome.
    #[inline]
    pub fn insert_call_outcome(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        outcome: CallOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.insert_call_outcome)(context, frame, shared_memory, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn create(
        &self,
        context: &mut Context<EvmWiringT>,
        inputs: Box<CreateInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
        (self.create)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn create_return(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: Box<CreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CreateOutcome, EvmWiringT> {
        (self.create_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_create_outcome(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.insert_create_outcome)(context, frame, outcome)
    }

    /// Call Create frame
    #[inline]
    pub fn eofcreate(
        &self,
        context: &mut Context<EvmWiringT>,
        inputs: Box<EOFCreateInputs>,
    ) -> EVMResultGeneric<FrameOrResult, EvmWiringT> {
        (self.eofcreate)(context, inputs)
    }

    /// Call handler for create return.
    #[inline]
    pub fn eofcreate_return(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: Box<EOFCreateFrame>,
        interpreter_result: InterpreterResult,
    ) -> EVMResultGeneric<CreateOutcome, EvmWiringT> {
        (self.eofcreate_return)(context, frame, interpreter_result)
    }

    /// Call handler for inserting create outcome.
    #[inline]
    pub fn insert_eofcreate_outcome(
        &self,
        context: &mut Context<EvmWiringT>,
        frame: &mut Frame,
        outcome: CreateOutcome,
    ) -> EVMResultGeneric<(), EvmWiringT> {
        (self.insert_eofcreate_outcome)(context, frame, outcome)
    }
}
